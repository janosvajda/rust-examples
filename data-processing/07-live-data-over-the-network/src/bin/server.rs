// The server on its own: streams one reading per second to every client that
// connects. Stop it with Ctrl+C.
//
//     cargo run --bin server            a well-behaved server
//     cargo run --bin server -- faults  a server that misbehaves on purpose
//
// Tokens: "demo-token" may read A12 and B22, "guest-token" only B22.

use live_data_over_the_network::server::{ServerConfig, serve};
use std::collections::HashMap;
use std::net::TcpListener;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let faults = std::env::args().any(|arg| arg == "faults");
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("server listening on 127.0.0.1:8080 (faults: {faults}; Ctrl+C to stop)");
    serve(
        listener,
        ServerConfig {
            tokens: HashMap::from([
                (String::from("demo-token"), vec![String::from("A12"), String::from("B22")]),
                (String::from("guest-token"), vec![String::from("B22")]),
            ]),
            max_clients: 8,
            interval: Duration::from_secs(1),
            last_seq: None,
            faults,
            log: true,
        },
    );
    Ok(())
}
