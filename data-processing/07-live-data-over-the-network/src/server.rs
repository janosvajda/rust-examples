//! The server: checks who's asking (authentication) and what they may read
//! (authorization), then streams numbered readings. With `faults` switched
//! on, it also misbehaves in every way a real data source can, on purpose,
//! so the client can prove it copes.

use crate::protocol::{LineRead, MAX_LINE_BYTES, read_line_limited};
use std::collections::HashMap;
use std::io::{self, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

pub struct ServerConfig {
    /// token → the sensors that token may read. A real server would keep
    /// hashed tokens in a database; the idea is the same.
    pub tokens: HashMap<String, Vec<String>>,
    /// More clients than this are turned away with `ERR busy`.
    pub max_clients: usize,
    /// Pause between readings.
    pub interval: Duration,
    /// Stop each stream after this sequence number (None: never).
    pub last_seq: Option<u64>,
    /// Misbehave on purpose (see `Faults`).
    pub faults: bool,
    /// Print a line when a client's stream ends.
    pub log: bool,
}

/// One-time faults remember whether they've happened, so a reconnecting
/// client sees them once, not on every attempt.
#[derive(Default)]
struct Faults {
    dropped: AtomicBool,
    stalled: AtomicBool,
    skipped: AtomicBool,
}

/// The reading for a sequence number. The same number always gives the same
/// value, like a real stored measurement, so a resent reading is identical.
pub fn reading(seq: u64) -> f64 {
    let mut x = seq.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    x ^= x >> 31;
    let noise = (x >> 40) as f64 / (1u64 << 24) as f64;
    if seq % 9 == 8 { 35.0 + noise * 5.0 } else { 21.0 + noise * 2.0 } // every 9th: a spike
}

pub fn serve(listener: TcpListener, config: ServerConfig) {
    let config = Arc::new(config);
    let faults = Arc::new(Faults::default());
    let clients = Arc::new(AtomicUsize::new(0));
    for connection in listener.incoming() {
        let Ok(mut stream) = connection else { continue };
        // Too many clients: refuse politely instead of running out of threads.
        if clients.fetch_add(1, Ordering::SeqCst) >= config.max_clients {
            clients.fetch_sub(1, Ordering::SeqCst);
            let _ = writeln!(stream, "ERR busy");
            continue;
        }
        let (config, faults, clients) = (Arc::clone(&config), Arc::clone(&faults), Arc::clone(&clients));
        thread::spawn(move || {
            let peer = stream.peer_addr().map(|a| a.to_string()).unwrap_or_default();
            let result = handle_client(stream, &config, &faults);
            if config.log {
                match result {
                    Ok(()) => println!("[server] {peer}: stream finished"),
                    Err(error) => println!("[server] {peer}: {error}"),
                }
            }
            clients.fetch_sub(1, Ordering::SeqCst); // the slot is free again
        });
    }
}

fn handle_client(mut stream: TcpStream, config: &ServerConfig, faults: &Faults) -> io::Result<()> {
    // A client that connects and says nothing must not hold a thread forever.
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    // A client that stops reading must not block us forever either.
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;

    let mut reader = BufReader::new(stream.try_clone()?);
    let mut buf = Vec::new();
    if read_line_limited(&mut reader, &mut buf, MAX_LINE_BYTES)? != LineRead::Line {
        return Err(io::Error::other("no valid HELLO"));
    }
    let hello = String::from_utf8_lossy(&buf).into_owned();
    let parts: Vec<&str> = hello.split_whitespace().collect();
    let ["HELLO", token, sensor, first_seq] = parts[..] else {
        writeln!(stream, "ERR bad-request")?;
        return Err(io::Error::other("bad request"));
    };
    let Some(allowed) = config.tokens.get(token) else {
        writeln!(stream, "ERR unauthenticated")?; // who are you?
        return Err(io::Error::other("unknown token"));
    };
    if !allowed.iter().any(|s| s == sensor) {
        writeln!(stream, "ERR forbidden")?; // I know you, but not this sensor
        return Err(io::Error::other(format!("token may not read {sensor}")));
    }
    let Ok(mut seq) = first_seq.parse::<u64>() else {
        writeln!(stream, "ERR bad-request")?;
        return Err(io::Error::other("bad first-seq"));
    };
    writeln!(stream, "OK")?;

    // At-least-once delivery: after a reconnect, a real server often resends a
    // little of what the client may already have. Here: the last 2 readings.
    if config.faults && seq > 0 {
        seq = seq.saturating_sub(2);
    }

    let mut out = io::BufWriter::new(stream);
    while config.last_seq.is_none_or(|last| seq <= last) {
        if config.faults {
            match seq {
                5 => writeln!(out, "%%% garbage %%%")?,
                8 => out.write_all(b"\xff\xfe not text\n")?, // invalid UTF-8
                10 => writeln!(out, "{}", "x".repeat(100_000))?, // a line far too long
                13 => writeln!(out, "13,{},{sensor},NaN,C", time(13))?, // a broken value
                36 => writeln!(out, "36,{},{sensor},500.0,C", time(36))?, // an impossible value
                33 if !faults.skipped.swap(true, Ordering::SeqCst) => {
                    seq += 1; // a reading lost on the way: never sent
                    continue;
                }
                27 if !faults.stalled.swap(true, Ordering::SeqCst) => {
                    out.flush()?;
                    thread::sleep(Duration::from_millis(1500)); // the server hangs
                }
                _ => {}
            }
        }
        if !(config.faults && (seq == 13 || seq == 36)) {
            writeln!(out, "{seq},{},{sensor},{:.2},C", time(seq), reading(seq))?;
        }
        if config.faults && seq == 16 {
            writeln!(out, "{seq},{},{sensor},{:.2},C", time(seq), reading(seq))?; // sent twice
        }
        out.flush()?;
        if config.faults && seq == 21 && !faults.dropped.swap(true, Ordering::SeqCst) {
            return Err(io::Error::other("connection dropped on purpose after seq 21"));
        }
        seq += 1;
        thread::sleep(config.interval);
    }
    Ok(())
}

fn time(seq: u64) -> String {
    format!("12:{:02}:{:02}", seq / 60 % 60, seq % 60)
}
