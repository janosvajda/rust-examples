//! Lesson 7: live data over the network.
//!
//! A server streams sensor readings over TCP; a client processes them as they
//! arrive. The point of the lesson is everything that goes wrong in real life,
//! and how each problem is handled:
//!
//!   protocol.rs   the message format, validation, and memory-safe line reading
//!   server.rs     authentication, authorization, limits, and deliberate faults
//!   client.rs     timeouts, retries with backoff, resuming, duplicates, losses
//!
//! Three programs use this library: `cargo run` (a full demo), and the
//! separate `server` and `client` binaries in `src/bin/`.

pub mod client;
pub mod protocol;
pub mod server;

use std::collections::HashMap;
use std::net::{SocketAddr, TcpListener};
use std::thread;
use std::time::Duration;

/// Starts a server on a free port in a background thread, for the demo and tests.
/// Token "demo-token" may read A12 and B22; "guest-token" may read only B22.
pub fn start_server(faults: bool, last_seq: Option<u64>, max_clients: usize) -> SocketAddr {
    // Port 0 asks the operating system for any free port, so this never clashes.
    let listener = TcpListener::bind("127.0.0.1:0").expect("can bind to a local port");
    let address = listener.local_addr().expect("has an address");
    let config = server::ServerConfig {
        tokens: HashMap::from([
            (
                String::from("demo-token"),
                vec![String::from("A12"), String::from("B22")],
            ),
            (String::from("guest-token"), vec![String::from("B22")]),
        ]),
        max_clients,
        interval: Duration::from_millis(10),
        last_seq,
        faults,
        log: false,
    };
    thread::spawn(move || server::serve(listener, config));
    address
}

/// Client settings for the demo and tests: short timeouts, so failures show quickly.
pub fn demo_client(
    address: SocketAddr,
    token: &str,
    sensor: &str,
    stop_after_seq: u64,
) -> client::ClientConfig {
    client::ClientConfig {
        address,
        token: token.to_string(),
        sensor: sensor.to_string(),
        connect_timeout: Duration::from_millis(500),
        read_timeout: Duration::from_millis(500),
        max_retries: 4,
        base_delay: Duration::from_millis(50),
        max_delay: Duration::from_millis(400),
        stop_after_seq: Some(stop_after_seq),
    }
}

#[cfg(test)]
mod end_to_end {
    use super::*;
    use crate::client::{ClientError, Event, State, run};
    use crate::protocol::Rejection;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;

    #[test]
    fn a_clean_stream_arrives_completely() {
        let address = start_server(false, None, 4);
        let mut state = State::default();
        run(
            &demo_client(address, "demo-token", "A12", 49),
            &mut state,
            |_| {},
        )
        .unwrap();
        assert_eq!(
            (state.next_seq, state.accepted, state.lost, state.duplicates),
            (50, 50, 0, 0)
        );
    }

    #[test]
    fn every_fault_is_survived_and_accounted_for() {
        let address = start_server(true, None, 4);
        let mut state = State::default();
        let mut events = Vec::new();
        run(
            &demo_client(address, "demo-token", "A12", 39),
            &mut state,
            |e| events.push(e),
        )
        .unwrap();

        // Every sequence number 0..=39 ends up exactly once as accepted, rejected
        // with its number (the NaN at 13, the impossible value at 36) or lost (33).
        assert_eq!(state.next_seq, 40);
        assert_eq!(state.accepted + 2 + state.lost, 40);
        assert_eq!(state.lost, 1);
        assert!(events.contains(&Event::Lost { from: 33, to: 33 }));
        // the messy lines were all recognised
        for why in [
            Rejection::Malformed,
            Rejection::NotUtf8,
            Rejection::TooLong,
            Rejection::NotFinite,
            Rejection::OutOfRange,
        ] {
            assert!(state.rejected.contains_key(&why), "{why:?} wasn't seen");
        }
        assert!(
            state.duplicates >= 3,
            "the double send and the resends after reconnecting"
        );
        assert!(
            state.reconnects >= 2,
            "once for the dropped connection, once for the stall"
        );
        assert!(
            events
                .iter()
                .any(|e| matches!(e, Event::Disconnected(why) if why.contains("stalled")))
        );
    }

    #[test]
    fn a_wrong_token_is_refused_without_retrying() {
        let address = start_server(false, None, 4);
        let mut state = State::default();
        let result = run(
            &demo_client(address, "stolen-token", "A12", 5),
            &mut state,
            |_| {},
        );
        assert_eq!(result, Err(ClientError::Unauthenticated));
        assert_eq!(state.reconnects, 0);
    }

    #[test]
    fn a_valid_token_cannot_read_another_sensor() {
        let address = start_server(false, None, 4);
        let result = run(
            &demo_client(address, "guest-token", "A12", 5),
            &mut State::default(),
            |_| {},
        );
        assert_eq!(result, Err(ClientError::Forbidden));
    }

    #[test]
    fn an_unavailable_server_is_retried_then_given_up() {
        // Bind and immediately drop a listener: now nothing listens on that port.
        let address = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap();
        let mut state = State::default();
        let mut retries = 0;
        let result = run(
            &demo_client(address, "demo-token", "A12", 5),
            &mut state,
            |e| {
                if matches!(e, Event::Retrying { .. }) {
                    retries += 1;
                }
            },
        );
        assert!(matches!(
            result,
            Err(ClientError::GaveUp { attempts: 5, .. })
        ));
        assert_eq!(retries, 4);
    }

    #[test]
    fn too_many_clients_are_turned_away() {
        let address = start_server(false, None, 1);
        let mut first = TcpStream::connect(address).unwrap();
        writeln!(first, "HELLO demo-token A12 0").unwrap();
        let mut answer = String::new();
        BufReader::new(&first).read_line(&mut answer).unwrap(); // the first client is in
        let second = TcpStream::connect(address).unwrap();
        let mut answer = String::new();
        BufReader::new(second).read_line(&mut answer).unwrap();
        assert_eq!(answer.trim(), "ERR busy");
    }

    #[test]
    fn a_silent_client_is_disconnected_by_the_server() {
        let address = start_server(false, None, 4);
        let mut silent = TcpStream::connect(address).unwrap(); // connects, never says HELLO
        let mut line = String::new();
        let n = BufReader::new(&mut silent).read_line(&mut line).unwrap();
        assert_eq!(
            n, 0,
            "the server closed the connection after its 2 s timeout"
        );
    }
}
