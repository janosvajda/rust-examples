// Lesson 1: what async is.
//
// An `async fn` doesn't run when you call it. It returns a FUTURE: a value
// describing work that can be done later, and that can pause while waiting
// (for a timer, the network, a file) without blocking a thread. `.await`
// runs a future to completion. A RUNTIME (here tokio) drives the futures.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

// ---- 1. async fn returns a future, and futures are lazy ----------------------------

async fn make_coffee() -> &'static str {
    println!("    (making coffee…)");
    tokio::time::sleep(Duration::from_millis(300)).await; // waits WITHOUT blocking the thread
    "coffee"
}

// ---- 2. What a future really is: something you can poll ---------------------------

/// A hand-written future, to show what `async` generates for you. Each time
/// the runtime POLLS it, it either finishes (`Ready`) or says "not yet"
/// (`Pending`) and arranges to be woken up and polled again.
struct CountToThree {
    polls: u32,
}

impl Future for CountToThree {
    type Output = u32;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<u32> {
        self.polls += 1;
        println!("    poll #{}", self.polls);
        if self.polls == 3 {
            Poll::Ready(self.polls)
        } else {
            // "Not done yet. Please poll me again." Real futures call the
            // waker later, when the thing they're waiting for has happened.
            cx.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

// ---- 3. Why bother: waiting for several things at once ----------------------------

async fn download(name: &str, millis: u64) -> String {
    tokio::time::sleep(Duration::from_millis(millis)).await;
    format!("{name} ({millis} ms)")
}

/// `#[tokio::main]` starts a tokio runtime and runs `main`'s future on it.
/// Without it, `async fn main` doesn't compile:
///     error[E0752]: `main` function is not allowed to be `async`
#[tokio::main]
async fn main() {
    println!("1. Calling an async fn does nothing until you .await it");
    let future = make_coffee(); // nothing printed yet: the future is just created
    println!("    future created, coffee not started yet");
    let coffee = future.await; // NOW it runs
    println!("    got {coffee}");
    // make_coffee();   // without .await:
    //     warning: unused implementer of `Future` that must be used
    //     = note: futures do nothing unless you `.await` or poll them

    println!("\n2. A future is polled until it's ready");
    let result = CountToThree { polls: 0 }.await;
    println!("    ready with {result}");

    println!("\n3. One after another vs at the same time");
    let start = Instant::now();
    let a = download("page", 300).await;
    let b = download("image", 300).await;
    println!("    sequential: {a}, {b} took {} ms", start.elapsed().as_millis());

    let start = Instant::now();
    let (a, b) = tokio::join!(download("page", 300), download("image", 300));
    println!("    concurrent: {a}, {b} took {} ms", start.elapsed().as_millis());
    // Both waits overlapped on ONE thread: while one future waits, the
    // runtime polls the other. Lesson 2 goes deeper.
}

#[cfg(test)]
mod tests {
    use super::*;

    // `start_paused = true` freezes tokio's clock: sleeps complete instantly
    // in real time but the "virtual" time still advances exactly.

    #[tokio::test]
    async fn hand_written_future_needs_three_polls() {
        assert_eq!(CountToThree { polls: 0 }.await, 3);
    }

    #[tokio::test(start_paused = true)]
    async fn sequential_awaits_add_up() {
        let start = tokio::time::Instant::now();
        download("a", 300).await;
        download("b", 300).await;
        assert_eq!(start.elapsed(), Duration::from_millis(600));
    }

    #[tokio::test(start_paused = true)]
    async fn joined_futures_overlap() {
        let start = tokio::time::Instant::now();
        tokio::join!(download("a", 300), download("b", 300));
        assert_eq!(start.elapsed(), Duration::from_millis(300));
    }

    #[tokio::test]
    async fn futures_are_lazy() {
        use std::sync::atomic::{AtomicBool, Ordering};
        static STARTED: AtomicBool = AtomicBool::new(false);
        let future = async { STARTED.store(true, Ordering::SeqCst) };
        assert!(!STARTED.load(Ordering::SeqCst)); // created, not run
        future.await;
        assert!(STARTED.load(Ordering::SeqCst)); // run by .await
    }
}
