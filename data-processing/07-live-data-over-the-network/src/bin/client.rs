// The client on its own: connects to the server and processes readings until
// stopped with Ctrl+C. Start the server first, in another terminal.
//
//     SENSOR_TOKEN=demo-token cargo run --bin client
//     SENSOR_TOKEN=demo-token cargo run --bin client -- B22
//
// The token comes from an environment variable, not from the code: secrets
// don't belong in source files, where they end up in version control.

use live_data_over_the_network::client::{ClientConfig, State, describe, run};
use std::time::Duration;

fn main() {
    let Ok(token) = std::env::var("SENSOR_TOKEN") else {
        eprintln!(
            "set SENSOR_TOKEN first, for example: SENSOR_TOKEN=demo-token cargo run --bin client"
        );
        std::process::exit(2);
    };
    let sensor = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("A12"));
    let config = ClientConfig {
        address: "127.0.0.1:8080".parse().expect("a valid address"),
        token,
        sensor,
        connect_timeout: Duration::from_secs(3),
        read_timeout: Duration::from_secs(5), // the server sends every second
        max_retries: 5,
        base_delay: Duration::from_millis(500),
        max_delay: Duration::from_secs(10),
        stop_after_seq: None,
    };
    let mut state = State::default();
    let result = run(&config, &mut state, |event| {
        println!("{}", describe(&event))
    });
    match result {
        Ok(()) => println!("finished"),
        Err(error) => {
            eprintln!("stopped: {error}");
            std::process::exit(1);
        }
    }
}
