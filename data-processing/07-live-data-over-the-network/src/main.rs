// Lesson 7: live data over the network, the full demo.
//
// Starts a server that misbehaves on purpose, and a client that copes with
// all of it. Then four short scenarios: a wrong token, a forbidden sensor,
// and a server that isn't there at all.

use live_data_over_the_network::client::{Event, State, describe, run};
use live_data_over_the_network::{demo_client, start_server};
use std::net::TcpListener;

fn main() {
    println!("1. A stream with every kind of trouble (readings #0–#39)\n");
    let address = start_server(true, None, 4);
    let mut state = State::default();
    let result = run(&demo_client(address, "demo-token", "A12", 39), &mut state, |event| {
        // Print only the interesting events; plain readings would scroll by.
        if !matches!(event, Event::Reading { .. }) || matches!(event, Event::Reading { seq, .. } if seq % 10 == 0) {
            println!("    {}", describe(&event));
        }
    });
    println!("\n    result: {result:?}");
    println!(
        "    accepted {}, alerts {}, duplicates ignored {}, lost {}, reconnects {}",
        state.accepted, state.alerts, state.duplicates, state.lost, state.reconnects
    );
    println!("    rejected: {:?}", state.rejected);
    println!("    every reading accounted for: next expected #{}", state.next_seq);

    println!("\n2. A wrong token (authentication)");
    let outcome = run(&demo_client(address, "stolen-token", "A12", 5), &mut State::default(), |_| {});
    println!("    {}", outcome.map_or_else(|e| e.to_string(), |_| String::from("unexpectedly allowed")));

    println!("\n3. A valid token, but a sensor it may not read (authorization)");
    let outcome = run(&demo_client(address, "guest-token", "A12", 5), &mut State::default(), |_| {});
    println!("    {}", outcome.map_or_else(|e| e.to_string(), |_| String::from("unexpectedly allowed")));

    println!("\n4. No server at all");
    let nowhere = TcpListener::bind("127.0.0.1:0").and_then(|l| l.local_addr()).expect("a free port");
    let outcome = run(&demo_client(nowhere, "demo-token", "A12", 5), &mut State::default(), |event| {
        println!("    {}", describe(&event));
    });
    println!("    {}", outcome.map_or_else(|e| e.to_string(), |_| String::from("unexpectedly connected")));
}
