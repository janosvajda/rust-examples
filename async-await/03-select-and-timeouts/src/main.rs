// Lesson 3: select and timeouts.
//
// Sometimes you don't want ALL futures to finish, just the FIRST one:
//   tokio::select!   race several futures, continue with the winner
//   timeout          give a future a deadline
// The losers are DROPPED, and dropping a future cancels it.

use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{Instant, interval, sleep, timeout};

async fn ask_server(name: &str, millis: u64) -> String {
    sleep(Duration::from_millis(millis)).await;
    format!("answer from {name}")
}

/// Asks two mirrors and takes whichever answers first.
async fn fastest_answer() -> String {
    tokio::select! {
        answer = ask_server("Europe mirror", 150) => answer,
        answer = ask_server("US mirror", 400) => answer,
    }
}

/// Gives up if the server takes longer than `limit`.
async fn ask_with_deadline(millis: u64, limit: Duration) -> Result<String, String> {
    timeout(limit, ask_server("slow server", millis))
        .await
        .map_err(|_| format!("no answer within {} ms", limit.as_millis()))
}

/// A worker that does periodic work until it's told to stop.
async fn worker(mut stop: oneshot::Receiver<()>) -> u32 {
    let mut ticker = interval(Duration::from_millis(100));
    let mut ticks = 0;
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                ticks += 1;
                println!("    tick {ticks}");
            }
            _ = &mut stop => {
                println!("    stop signal received, cleaning up");
                return ticks;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("1. select!: the first future to finish wins");
    let start = Instant::now();
    println!("    {} after {} ms", fastest_answer().await, start.elapsed().as_millis());
    // The US mirror's future was dropped (cancelled) when Europe answered.

    println!("\n2. timeout: give up after a deadline");
    println!("    {:?}", ask_with_deadline(100, Duration::from_millis(300)).await);
    println!("    {:?}", ask_with_deadline(900, Duration::from_millis(300)).await);

    println!("\n3. select! in a loop: periodic work with a stop signal");
    let (stop_sender, stop_receiver) = oneshot::channel();
    let task = tokio::spawn(worker(stop_receiver));
    sleep(Duration::from_millis(350)).await;
    stop_sender.send(()).unwrap();
    println!("    worker ran {} ticks", task.await.unwrap());

    println!("\n4. Cancellation is just dropping the future");
    let slow = ask_server("never awaited", 10_000);
    drop(slow); // nothing ran, nothing to clean up: the future is simply gone
    println!("    a dropped future never runs again");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn select_returns_the_faster_one() {
        let start = Instant::now();
        assert_eq!(fastest_answer().await, "answer from Europe mirror");
        assert_eq!(start.elapsed(), Duration::from_millis(150));
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_passes_through_a_fast_result() {
        let result = ask_with_deadline(100, Duration::from_millis(200)).await;
        assert_eq!(result, Ok(String::from("answer from slow server")));
    }

    #[tokio::test(start_paused = true)]
    async fn timeout_gives_up_at_the_deadline() {
        let start = Instant::now();
        let result = ask_with_deadline(5_000, Duration::from_millis(200)).await;
        assert_eq!(result, Err(String::from("no answer within 200 ms")));
        assert_eq!(start.elapsed(), Duration::from_millis(200)); // didn't wait 5 s
    }

    #[tokio::test(start_paused = true)]
    async fn worker_stops_when_told() {
        let (stop, receiver) = oneshot::channel();
        let task = tokio::spawn(worker(receiver));
        sleep(Duration::from_millis(250)).await; // ticks at 0, 100, 200
        stop.send(()).unwrap();
        assert_eq!(task.await.unwrap(), 3);
    }
}
