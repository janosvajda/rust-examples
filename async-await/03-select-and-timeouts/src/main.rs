// Lesson 3: select and timeouts.
//
// Sometimes you want the first ready, matching branch rather than every result:
//   tokio::select!   race several futures, continue with the winner
//   timeout          give a future a deadline
// Owned losing futures are dropped. Borrowed futures can remain alive outside
// select!, and dropping a JoinHandle detaches its task rather than stopping it.

use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::{Instant, interval, sleep, timeout};

/// Simulates a reply with a timer; no network request is made.
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

/// Gives up waiting after `limit`, as long as polling the operation yields.
async fn ask_with_deadline(millis: u64, limit: Duration) -> Result<String, String> {
    timeout(limit, ask_server("slow server", millis))
        .await
        .map_err(|_| format!("no answer within {} ms", limit.as_millis()))
}

/// A worker that stops on a message or when the stop sender is dropped.
/// interval's first tick is immediate; the count saturates at u32::MAX.
async fn worker(mut stop: oneshot::Receiver<()>) -> u32 {
    let mut ticker = interval(Duration::from_millis(100));
    let mut ticks = 0_u32;
    loop {
        tokio::select! {
            // Prefer shutdown if both the stop signal and a tick are ready.
            biased;
            signal = &mut stop => {
                match signal {
                    Ok(()) => println!("    stop signal received, cleaning up"),
                    Err(_) => println!("    stop sender dropped, cleaning up"),
                }
                return ticks;
            }
            _ = ticker.tick() => {
                ticks = ticks.saturating_add(1);
                println!("    tick {ticks}");
            }
        }
    }
}

#[tokio::main]
async fn main() {
    println!("1. select!: take the first ready, matching reply");
    let start = Instant::now();
    println!(
        "    {} after {} ms",
        fastest_answer().await,
        start.elapsed().as_millis()
    );
    // The US mirror's future was dropped (cancelled) when Europe answered.

    println!("\n2. timeout: give up after a deadline");
    println!(
        "    {:?}",
        ask_with_deadline(100, Duration::from_millis(300)).await
    );
    println!(
        "    {:?}",
        ask_with_deadline(900, Duration::from_millis(300)).await
    );

    println!("\n3. select! in a loop: periodic work with a stop signal");
    let (stop_sender, stop_receiver) = oneshot::channel();
    let task = tokio::spawn(worker(stop_receiver));
    sleep(Duration::from_millis(350)).await;
    stop_sender.send(()).unwrap();
    println!("    worker ran {} ticks", task.await.unwrap());

    println!("\n4. Drop an unpolled operation: its async body never starts");
    let slow = ask_server("never awaited", 10_000);
    drop(slow); // the body's timer wasn't created; captured arguments are dropped
    println!("    a dropped future never runs again");

    println!("\n5. A task handle needs explicit cancellation");
    let mut task = tokio::spawn(async {
        sleep(Duration::from_millis(200)).await;
        "background result"
    });
    if timeout(Duration::from_millis(20), &mut task).await.is_err() {
        println!("    stopped waiting; request cancellation and join the task");
        task.abort();
        match task.await {
            Err(error) if error.is_cancelled() => println!("    task cancelled"),
            Ok(result) => println!("    task had already completed: {result}"),
            Err(error) => println!("    task failed: {error}"),
        }
    }
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

    #[tokio::test(start_paused = true)]
    async fn dropping_the_stop_sender_also_stops_the_worker() {
        let (stop, receiver) = oneshot::channel();
        drop(stop);
        assert_eq!(worker(receiver).await, 0); // shutdown takes priority over first tick
    }

    #[tokio::test(start_paused = true)]
    async fn timing_out_a_borrowed_handle_leaves_the_task_running() {
        let mut task = tokio::spawn(async {
            sleep(Duration::from_millis(100)).await;
            42
        });
        assert!(timeout(Duration::from_millis(10), &mut task).await.is_err());
        assert_eq!(task.await.unwrap(), 42); // still running after the timeout
    }

    #[tokio::test(start_paused = true)]
    async fn timing_out_an_owned_operation_drops_its_live_resources() {
        use std::sync::{
            Arc,
            atomic::{AtomicBool, Ordering},
        };
        struct Notice(Arc<AtomicBool>);
        impl Drop for Notice {
            fn drop(&mut self) {
                self.0.store(true, Ordering::SeqCst);
            }
        }
        let dropped = Arc::new(AtomicBool::new(false));
        let result = timeout(Duration::from_millis(10), async {
            let _notice = Notice(Arc::clone(&dropped));
            sleep(Duration::from_millis(100)).await;
        })
        .await;
        assert!(result.is_err());
        assert!(dropped.load(Ordering::SeqCst));
    }
}
