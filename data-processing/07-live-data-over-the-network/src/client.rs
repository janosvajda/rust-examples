//! The client: connects, proves who it is, processes readings as they arrive,
//! and survives everything the network and the server can throw at it.

use crate::protocol::{Bad, LineRead, MAX_LINE_BYTES, Message, Rejection, parse_message, read_line_limited};
use std::collections::{BTreeMap, VecDeque};
use std::fmt;
use std::io::{self, BufReader, Write};
use std::net::{SocketAddr, TcpStream};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub struct ClientConfig {
    pub address: SocketAddr,
    pub token: String,
    pub sensor: String,
    /// Give up connecting after this long (an unreachable host can hang for minutes otherwise).
    pub connect_timeout: Duration,
    /// If nothing arrives for this long, the server is considered stalled.
    pub read_timeout: Duration,
    /// Retries in a row before giving up. Reset after every successful connection.
    pub max_retries: u32,
    /// The first retry waits this long; each further retry waits twice as long…
    pub base_delay: Duration,
    /// …but never longer than this.
    pub max_delay: Duration,
    /// Stop after this sequence number (None: run until stopped).
    pub stop_after_seq: Option<u64>,
}

/// Everything that can happen, reported to the caller as it happens.
#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Connected { resume_from: u64 },
    Reading { seq: u64, time: String, celsius: f64, moving_average: f64 },
    Alert { seq: u64, time: String, celsius: f64 },
    Rejected(Rejection),
    Duplicate { seq: u64 },
    Lost { from: u64, to: u64 },
    Disconnected(String),
    Retrying { attempt: u32, after: Duration },
}

/// Failures that end the whole run.
#[derive(Debug, Clone, PartialEq)]
pub enum ClientError {
    /// The server doesn't know our token. Retrying won't help.
    Unauthenticated,
    /// The server knows us, but we may not read this sensor. Retrying won't help.
    Forbidden,
    /// The server didn't understand our request: a bug, so retrying won't help.
    BadRequest,
    /// Still failing after every retry.
    GaveUp { attempts: u32, last_error: String },
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClientError::Unauthenticated => write!(f, "the server rejected our token (not retrying)"),
            ClientError::Forbidden => write!(f, "this token may not read that sensor (not retrying)"),
            ClientError::BadRequest => write!(f, "the server didn't understand the request (not retrying)"),
            ClientError::GaveUp { attempts, last_error } => {
                write!(f, "gave up after {attempts} attempts; last error: {last_error}")
            }
        }
    }
}

// ---- The client's memory: fixed size, however long it runs ------------------------------

const WINDOW: usize = 5;
const ALERT_ABOVE: f64 = 30.0;

/// Everything the client remembers. Every field has a fixed size: nothing here
/// grows while the stream runs, so the client can run for months without its
/// memory growing.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    /// The next sequence number we expect. This one number is enough to
    /// resume, to recognise duplicates and to notice losses.
    pub next_seq: u64,
    pub accepted: u64,
    pub alerts: u64,
    pub duplicates: u64,
    pub lost: u64,
    pub reconnects: u64,
    /// One counter per kind of rejection: at most 8 entries, ever.
    pub rejected: BTreeMap<Rejection, u64>,
    pub min: f64,
    pub max: f64,
    /// The last WINDOW normal readings, for the moving average.
    window: VecDeque<f64>,
}

impl Default for State {
    fn default() -> Self {
        State {
            next_seq: 0,
            accepted: 0,
            alerts: 0,
            duplicates: 0,
            lost: 0,
            reconnects: 0,
            rejected: BTreeMap::new(),
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
            window: VecDeque::with_capacity(WINDOW),
        }
    }
}

impl State {
    pub fn window_len(&self) -> usize {
        self.window.len()
    }

    /// Moves `next_seq` past `seq`, reporting losses before it. Returns false
    /// if `seq` was already seen: a duplicate, which must change nothing.
    fn arrived(&mut self, seq: u64, on_event: &mut impl FnMut(Event)) -> bool {
        if seq < self.next_seq {
            self.duplicates += 1;
            on_event(Event::Duplicate { seq });
            return false;
        }
        if seq > self.next_seq {
            self.lost += seq - self.next_seq;
            on_event(Event::Lost { from: self.next_seq, to: seq - 1 });
        }
        self.next_seq = seq + 1;
        true
    }

    /// Handle one validated message. Applying the same `seq` a second time
    /// changes nothing: that's what makes processing idempotent.
    pub fn apply(&mut self, m: Message, on_event: &mut impl FnMut(Event)) {
        if !self.arrived(m.seq, on_event) {
            return;
        }
        self.accepted += 1;
        self.min = self.min.min(m.celsius);
        self.max = self.max.max(m.celsius);
        if m.celsius > ALERT_ABOVE {
            // Reported, but kept out of the moving average, so one faulty
            // reading doesn't distort the trend.
            self.alerts += 1;
            return on_event(Event::Alert { seq: m.seq, time: m.time, celsius: m.celsius });
        }
        if self.window.len() == WINDOW {
            self.window.pop_front();
        }
        self.window.push_back(m.celsius);
        let moving_average = self.window.iter().sum::<f64>() / self.window.len() as f64;
        on_event(Event::Reading { seq: m.seq, time: m.time, celsius: m.celsius, moving_average });
    }

    /// Handle a refused line. If its sequence number is known, that reading
    /// arrived (unusable, but not lost), and a resent copy is still a duplicate.
    pub fn reject(&mut self, bad: Bad, on_event: &mut impl FnMut(Event)) {
        if let Some(seq) = bad.seq
            && !self.arrived(seq, on_event)
        {
            return;
        }
        *self.rejected.entry(bad.why).or_insert(0) += 1;
        on_event(Event::Rejected(bad.why));
    }

    fn done(&self, config: &ClientConfig) -> bool {
        config.stop_after_seq.is_some_and(|last| self.next_seq > last)
    }
}

// ---- One connection, from HELLO until it ends ----------------------------------------------

/// How one session ended.
enum SessionEnd {
    /// We have everything we wanted.
    Finished,
    /// Something went wrong that a retry may fix: say what.
    Retry(String),
    /// Something went wrong that no retry will fix.
    Fatal(ClientError),
}

fn session(config: &ClientConfig, state: &mut State, on_event: &mut impl FnMut(Event)) -> SessionEnd {
    match try_session(config, state, on_event) {
        Ok(end) => end,
        Err(error) if matches!(error.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {
            SessionEnd::Retry(format!("no data for {:?}: the server seems stalled", config.read_timeout))
        }
        Err(error) => SessionEnd::Retry(error.to_string()),
    }
}

fn try_session(config: &ClientConfig, state: &mut State, on_event: &mut impl FnMut(Event)) -> io::Result<SessionEnd> {
    let mut stream = TcpStream::connect_timeout(&config.address, config.connect_timeout)?;
    stream.set_read_timeout(Some(config.read_timeout))?;
    // Authenticate and say where to resume. A token belongs in a secure
    // channel (TLS) in real life: on plain TCP anyone on the network can read it.
    writeln!(stream, "HELLO {} {} {}", config.token, config.sensor, state.next_seq)?;

    let mut reader = BufReader::new(stream);
    let mut buf = Vec::with_capacity(MAX_LINE_BYTES + 1);
    if read_line_limited(&mut reader, &mut buf, MAX_LINE_BYTES)? != LineRead::Line {
        return Ok(SessionEnd::Retry(String::from("the server closed the connection before answering")));
    }
    match &buf[..] {
        b"OK" => on_event(Event::Connected { resume_from: state.next_seq }),
        b"ERR unauthenticated" => return Ok(SessionEnd::Fatal(ClientError::Unauthenticated)),
        b"ERR forbidden" => return Ok(SessionEnd::Fatal(ClientError::Forbidden)),
        b"ERR bad-request" => return Ok(SessionEnd::Fatal(ClientError::BadRequest)),
        other => return Ok(SessionEnd::Retry(format!("server said {:?}", String::from_utf8_lossy(other)))),
    }

    loop {
        if state.done(config) {
            return Ok(SessionEnd::Finished);
        }
        match read_line_limited(&mut reader, &mut buf, MAX_LINE_BYTES)? {
            LineRead::Closed => return Ok(SessionEnd::Retry(String::from("the server closed the connection"))),
            LineRead::TooLong => state.reject(Bad { seq: None, why: Rejection::TooLong }, on_event),
            LineRead::Line => match std::str::from_utf8(&buf) {
                Err(_) => state.reject(Bad { seq: None, why: Rejection::NotUtf8 }, on_event),
                Ok(line) => match parse_message(line, &config.sensor) {
                    Ok(message) => state.apply(message, on_event),
                    Err(bad) => state.reject(bad, on_event),
                },
            },
        }
    }
}

// ---- The whole run: reconnect, back off, give up -------------------------------------------

/// Runs until `stop_after_seq` is reached, or a fatal error, or the retries run out.
/// `state` is kept across reconnects, so nothing is counted twice.
pub fn run(config: &ClientConfig, state: &mut State, mut on_event: impl FnMut(Event)) -> Result<(), ClientError> {
    let mut failures_in_a_row = 0;
    loop {
        let seq_before = state.next_seq;
        match session(config, state, &mut on_event) {
            SessionEnd::Finished => return Ok(()),
            SessionEnd::Fatal(error) => return Err(error),
            SessionEnd::Retry(why) => {
                on_event(Event::Disconnected(why.clone()));
                if state.next_seq > seq_before {
                    failures_in_a_row = 0; // we made progress: the connection was fine for a while
                }
                failures_in_a_row += 1;
                if failures_in_a_row > config.max_retries {
                    return Err(ClientError::GaveUp { attempts: failures_in_a_row, last_error: why });
                }
                let delay = backoff(config, failures_in_a_row);
                on_event(Event::Retrying { attempt: failures_in_a_row, after: delay });
                thread::sleep(delay);
                state.reconnects += 1;
            }
        }
    }
}

/// Exponential backoff with jitter: base × 2^(attempt − 1), capped, then a
/// random 50–100% of that. Doubling stops a client from hammering a struggling
/// server; the randomness stops a thousand clients from all retrying at the
/// same instant after an outage.
pub fn backoff(config: &ClientConfig, attempt: u32) -> Duration {
    let doubled = config.base_delay.saturating_mul(2u32.saturating_pow(attempt.saturating_sub(1)));
    let capped = doubled.min(config.max_delay);
    // A little randomness without a crate: scramble the clock's nanoseconds.
    // (Some clocks only tick in microseconds, so the raw low digits are always 0.)
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_nanos() as u64);
    let random = (nanos.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 11) as f64 / (1u64 << 53) as f64; // 0.0 to 1.0
    capped.mul_f64(0.5 + random / 2.0)
}

/// How the example programs print an event.
pub fn describe(event: &Event) -> String {
    match event {
        Event::Connected { resume_from } => format!("connected, resuming from seq {resume_from}"),
        Event::Reading { seq, time, celsius, moving_average } => {
            format!("#{seq:<3} {time}  {celsius:>6.2} °C   moving average {moving_average:.2}")
        }
        Event::Alert { seq, time, celsius } => format!("#{seq:<3} {time}  {celsius:>6.2} °C   ⚠ ALERT: above {ALERT_ABOVE}"),
        Event::Rejected(why) => format!("     rejected: {why}"),
        Event::Duplicate { seq } => format!("     duplicate #{seq} ignored"),
        Event::Lost { from, to } if from == to => format!("     ⚠ reading #{from} was lost"),
        Event::Lost { from, to } => format!("     ⚠ readings #{from}–#{to} were lost"),
        Event::Disconnected(why) => format!("disconnected: {why}"),
        Event::Retrying { attempt, after } => format!("retry {attempt} in {after:.2?}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn message(seq: u64, celsius: f64) -> Message {
        Message { seq, time: String::from("t"), sensor: String::from("A12"), celsius }
    }

    #[test]
    fn duplicates_change_nothing() {
        let mut state = State::default();
        let mut events = Vec::new();
        state.apply(message(0, 21.0), &mut |e| events.push(e));
        let after_first = state.clone();
        state.apply(message(0, 21.0), &mut |e| events.push(e)); // the same reading again
        assert_eq!(state.duplicates, 1);
        state.duplicates = 0;
        assert_eq!(state, after_first, "processing a duplicate is idempotent");
    }

    #[test]
    fn a_skipped_sequence_number_is_reported_as_lost() {
        let mut state = State::default();
        let mut events = Vec::new();
        for seq in [0, 1, 4] {
            state.apply(message(seq, 21.0), &mut |e| events.push(e));
        }
        assert_eq!(state.lost, 2);
        assert!(events.contains(&Event::Lost { from: 2, to: 3 }));
        assert_eq!(state.next_seq, 5);
    }

    #[test]
    fn memory_stays_bounded_over_a_long_run() {
        let mut state = State::default();
        for seq in 0..1_000_000 {
            state.apply(message(seq, 20.0 + (seq % 7) as f64), &mut |_| {});
            state.reject(Bad { seq: None, why: Rejection::Malformed }, &mut |_| {});
        }
        assert_eq!(state.accepted, 1_000_000);
        assert_eq!(state.window_len(), WINDOW); // never more than 5 values
        assert_eq!(state.rejected.len(), 1); // one counter, not a million entries
    }

    #[test]
    fn backoff_doubles_up_to_the_cap() {
        let config = ClientConfig {
            address: "127.0.0.1:1".parse().unwrap(),
            token: String::new(),
            sensor: String::new(),
            connect_timeout: Duration::from_millis(100),
            read_timeout: Duration::from_millis(100),
            max_retries: 10,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            stop_after_seq: None,
        };
        for (attempt, full) in [(1, 100), (2, 200), (3, 400), (4, 800), (5, 1000), (9, 1000)] {
            let delay = backoff(&config, attempt).as_millis();
            assert!(delay >= full / 2 && delay <= full, "attempt {attempt}: {delay} ms");
        }
    }
}
