// Lesson 5: async pitfalls.
//
// The mistakes almost everyone makes when starting with async Rust, what
// the compiler (or the clock) tells you, and the fix for each.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---- 1. Blocking the runtime ------------------------------------------------------------
//
// An async task only gives the thread back at an `.await`. Code that blocks,
// like std::thread::sleep, a heavy calculation or a synchronous file read,
// holds the thread, and every other task on it has to wait.

/// Simulates CPU-heavy or blocking work.
fn heavy_calculation() -> u64 {
    std::thread::sleep(Duration::from_millis(200)); // stands in for real work
    42
}

/// WRONG: calls blocking code directly inside async code.
async fn blocking_inside_async() -> u64 {
    heavy_calculation()
}

/// RIGHT: moves blocking work to a separate pool of threads made for it.
async fn blocking_moved_out() -> u64 {
    tokio::task::spawn_blocking(heavy_calculation).await.unwrap()
}

/// Runs four of `job` concurrently on a SINGLE-threaded runtime and times it.
fn time_four<F, Fut>(job: F) -> u128
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = u64>,
{
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let start = Instant::now();
    runtime.block_on(async {
        tokio::join!(job(), job(), job(), job());
    });
    start.elapsed().as_millis()
}

// ---- 2. Holding a std Mutex lock across .await ---------------------------------------
//
//     let mut guard = counter.lock().unwrap();   // std::sync::Mutex
//     some_async_call().await;                     // ← the task may move threads here
//     *guard += 1;
//
// Inside tokio::spawn this doesn't compile:
//     error: future cannot be sent between threads safely
// Fix: finish with the lock before awaiting, by putting it in its own block
// (or use tokio::sync::Mutex, which is designed to be held across .await).

async fn increment_correctly(counter: Arc<Mutex<u32>>) {
    {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
    } // guard dropped: lock released BEFORE the await
    tokio::task::yield_now().await;
}

// ---- 3. Rc (and other non-Send types) in spawned tasks -------------------------------
//
//     let shared = Rc::new(5);
//     tokio::spawn(async move { …await…; println!("{shared}") });
//     error: future cannot be sent between threads safely
//
// tokio::spawn may move a task to another thread between awaits, so everything
// the task holds across an .await must be Send. Use Arc instead of Rc.

// ---- 4. Recursion needs a Box -----------------------------------------------------------
//
//     async fn countdown(n: u32) { if n > 0 { countdown(n - 1).await } }
//     error[E0733]: recursion in an async fn requires boxing
//
// A future contains everything it needs to remember across awaits, so a
// future that contains itself would be infinitely large. Box::pin puts the
// inner call on the heap.

async fn countdown(n: u32) -> Vec<u32> {
    if n == 0 {
        return Vec::new();
    }
    let mut rest = Box::pin(countdown(n - 1)).await;
    rest.insert(0, n);
    rest
}

// ---- 5. Forgetting .await ----------------------------------------------------------------
//
//     save_to_disk();         // looks like a call, does nothing
//     warning: unused implementer of `Future` that must be used
//     = note: futures do nothing unless you `.await` or poll them

fn main() {
    println!("Pitfalls 3 and 5 are compile errors and warnings: see the comments in the code.\n");
    println!("Pitfall 1. Blocking the runtime (4 jobs of 200 ms on ONE thread)");
    println!("    blocking inside async:   {} ms", time_four(blocking_inside_async));
    println!("    moved to spawn_blocking: {} ms", time_four(blocking_moved_out));

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        println!("\nPitfall 2. Release a std Mutex before .await");
        let counter = Arc::new(Mutex::new(0));
        let handles: Vec<_> = (0..5)
            .map(|_| tokio::spawn(increment_correctly(Arc::clone(&counter))))
            .collect();
        for handle in handles {
            handle.await.unwrap();
        }
        println!("    counter = {}", counter.lock().unwrap());

        println!("\nPitfall 4. Recursion with Box::pin");
        println!("    countdown: {:?}", countdown(5).await);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocking_serialises_the_tasks() {
        // On one thread, four blocking 200 ms jobs can't overlap: ~800 ms.
        assert!(time_four(blocking_inside_async) >= 800);
    }

    #[test]
    fn spawn_blocking_lets_them_overlap() {
        // Moved to the blocking pool, they run in parallel: well under 800 ms.
        assert!(time_four(blocking_moved_out) < 600);
    }

    #[tokio::test]
    async fn mutex_released_before_await() {
        let counter = Arc::new(Mutex::new(0));
        let tasks: Vec<_> = (0..10)
            .map(|_| tokio::spawn(increment_correctly(Arc::clone(&counter))))
            .collect();
        for task in tasks {
            task.await.unwrap();
        }
        assert_eq!(*counter.lock().unwrap(), 10);
    }

    #[tokio::test]
    async fn boxed_recursion() {
        assert_eq!(countdown(3).await, vec![3, 2, 1]);
        assert!(countdown(0).await.is_empty());
    }
}
