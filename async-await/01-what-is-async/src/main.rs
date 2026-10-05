// Lesson 1: what async is.
//
// An `async fn` doesn't run when you call it. It returns a FUTURE: a value
// describing work that can be done later, and that can pause while waiting
// (for a timer or the network) without blocking the polling thread. `.await`
// polls a future; Pending can suspend this task, while Ready continues inline.
// A runtime (here Tokio) schedules tasks and provides timer/I/O support.

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use tokio::time::Instant;

// Delay used by the simulated coffee operation.
const COFFEE_DELAY: Duration = Duration::from_millis(300);

// ---- 1. async fn returns a future, and futures are lazy ----------------------------

async fn make_coffee() -> &'static str {
    println!("    (making coffee…)");
    tokio::time::sleep(COFFEE_DELAY).await; // waits WITHOUT blocking the thread
    "coffee"
}

// ---- 2. What a future really is: something you can poll ---------------------------

/// Measures elapsed Tokio time from the first poll until the inner future finishes.
/// Includes time spent waiting and scheduling delays, rather than CPU time alone.
struct Timed<F> {
    // The inner future stays pinned even when this wrapper moves.
    inner: Pin<Box<F>>,
    started_at: Option<Instant>,
}

impl<F: Future> Timed<F> {
    fn new(future: F) -> Self {
        Self {
            inner: Box::pin(future),
            started_at: None,
        }
    }
}

impl<F: Future> Future for Timed<F> {
    type Output = (F::Output, Duration);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let started_at = *this.started_at.get_or_insert_with(Instant::now);

        // Forward the current context so the inner future can register wakeups.
        // It decides when it is ready; this wrapper never forces another poll.
        match this.inner.as_mut().poll(cx) {
            Poll::Ready(output) => Poll::Ready((output, started_at.elapsed())),
            Poll::Pending => Poll::Pending,
        }
    }
}

// ---- 3. Why bother: waiting for several things at once ----------------------------

/// A timer-based simulation; this does not make a network request.
async fn download(name: &str, millis: u64) -> String {
    tokio::time::sleep(Duration::from_millis(millis)).await;
    format!("{name} ({millis} ms)")
}

/// `#[tokio::main]` starts a tokio runtime and runs `main`'s future on it.
/// A plain `async fn main` without a wrapper doesn't compile:
///     error[E0752]: `main` function is not allowed to be `async`
#[tokio::main(flavor = "current_thread")]
async fn main() {
    println!("1. Calling an async fn creates a future; polling runs its body");
    let future = make_coffee(); // nothing printed yet: the future is just created
    println!("    future created, coffee not started yet");
    let coffee = future.await; // NOW it runs
    println!("    got {coffee}");
    // make_coffee();   // without .await:
    //     warning: unused implementer of `Future` that must be used
    //     = note: futures do nothing unless you `.await` or poll them

    println!("\n2. A custom future measures an async operation");
    let (coffee, elapsed) = Timed::new(make_coffee()).await;
    println!("    {coffee} ready after {} ms", elapsed.as_millis());

    println!("\n3. Sequential waits vs overlapping waits");
    let start = Instant::now();
    let a = download("page", 300).await;
    let b = download("image", 300).await;
    println!(
        "    sequential: {a}, {b} took {} ms",
        start.elapsed().as_millis()
    );

    let start = Instant::now();
    let (a, b) = tokio::join!(download("page", 300), download("image", 300));
    println!(
        "    concurrent: {a}, {b} took {} ms",
        start.elapsed().as_millis()
    );
    // The current-thread runtime polls this task. Inside it, join! polls both
    // downloads, overlapping their timer waits. Lesson 2 goes deeper.
}

#[cfg(test)]
mod tests {
    use super::*;

    // start_paused controls Tokio time, not std::time::Instant or thread::sleep.
    // When no task can progress, the runtime can jump to the next timer.

    #[tokio::test(start_paused = true)]
    async fn timed_future_measures_from_first_poll_until_completion() {
        let operation_duration = Duration::from_millis(250);
        let future = Timed::new(async {
            tokio::time::sleep(operation_duration).await;
            String::from("operation finished")
        });

        // Time before the wrapper's first poll must not enter its measurement.
        tokio::time::sleep(Duration::from_secs(1)).await;
        let (output, elapsed) = future.await;

        assert_eq!(output, "operation finished");
        assert_eq!(elapsed, operation_duration);
    }

    #[tokio::test(start_paused = true)]
    async fn timed_future_preserves_an_immediate_error_result() {
        let future = std::future::ready(Err::<(), _>("service unavailable"));
        let (output, elapsed) = Timed::new(future).await;

        assert_eq!(output, Err("service unavailable"));
        assert_eq!(elapsed, Duration::ZERO);
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
        let started = AtomicBool::new(false);
        let future = async { started.store(true, Ordering::SeqCst) };
        assert!(!started.load(Ordering::SeqCst)); // created, not run
        future.await;
        assert!(started.load(Ordering::SeqCst)); // run by .await
    }
}
