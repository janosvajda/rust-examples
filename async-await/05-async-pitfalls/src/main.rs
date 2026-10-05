// Lesson 5: async pitfalls.
//
// The mistakes almost everyone makes when starting with async Rust, what
// the compiler (or the clock) tells you, and the fix for each.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

// ---- 1. Blocking the runtime ------------------------------------------------------------
//
// An await suspends the task only when the awaited operation returns Pending.
// Blocking or long CPU work inside a poll prevents that thread from polling
// other tasks, even if the function is declared async.

/// A blocking 200 ms wait standing in for a synchronous operation.
/// This sleeps an OS thread; it does not perform CPU-heavy calculation.
fn heavy_calculation() -> u64 {
    std::thread::sleep(Duration::from_millis(200)); // stands in for real work
    42
}

/// WRONG: calls blocking code directly inside async code.
async fn blocking_inside_async() -> u64 {
    heavy_calculation()
}

/// Moves finite blocking work to Tokio's separate blocking pool.
/// Once started, this closure cannot be stopped by aborting its JoinHandle.
async fn blocking_moved_out() -> u64 {
    tokio::task::spawn_blocking(heavy_calculation)
        .await
        .unwrap()
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
// async fn increment_wrong(counter: Arc<Mutex<u32>>) {
//     let mut guard = counter.lock().unwrap();
//     tokio::task::yield_now().await;
//     *guard += 1;
// }
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
// See the complete commented spawn example inside main below.
// tokio::spawn requires its whole future and output to be Send + 'static,
// even on a current-thread runtime. A locally created Rc may be used within
// one poll if it is dropped before awaiting. LocalSet supports non-Send tasks.

// ---- 4. Recursion needs a Box -----------------------------------------------------------
//
// Boxing gives the recursive future a finite size. It does not remove stack
// growth when nested futures are polled immediately, so this example bounds depth.
const MAX_COUNTDOWN_DEPTH: u32 = 256;

async fn countdown(n: u32) -> Result<Vec<u32>, &'static str> {
    if n > MAX_COUNTDOWN_DEPTH {
        return Err("recursive countdown exceeds 256 steps");
    }
    async fn append(n: u32, output: &mut Vec<u32>) {
        if n == 0 {
            return;
        }
        output.push(n);
        Box::pin(append(n - 1, output)).await;
    }
    let mut output = Vec::with_capacity(n as usize);
    append(n, &mut output).await;
    Ok(output)
}

// ---- 5. Forgetting .await ----------------------------------------------------------------
//
//     save_note();            // creates and drops an unpolled future
//     warning: unused implementer of `Future` that must be used
//     = note: futures do nothing unless you `.await` or poll them

/// Simulates saving a note by printing; it does not write a file.
async fn save_note() {
    println!("    simulated save: the async body ran");
}

fn main() {
    println!("Comments show deliberate compiler errors; the active demo runs successfully.\n");
    println!("Pitfall 1. Blocking the runtime (4 jobs of 200 ms on ONE thread)");
    println!(
        "    blocking inside async:   {} ms",
        time_four(blocking_inside_async)
    );
    println!(
        "    moved to spawn_blocking: {} ms",
        time_four(blocking_moved_out)
    );

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        println!("\nPitfall 2. Release a std Mutex before .await");
        let counter = Arc::new(Mutex::new(0));
        // Uncomment increment_wrong above and this call together:
        // tokio::spawn(increment_wrong(Arc::clone(&counter)));
        // error: future cannot be sent between threads safely
        let handles: Vec<_> = (0..5)
            .map(|_| tokio::spawn(increment_correctly(Arc::clone(&counter))))
            .collect();
        for handle in handles {
            handle.await.unwrap();
        }
        println!("    counter = {}", counter.lock().unwrap());

        println!("\nPitfall 3. Use LocalSet for tasks that keep Rc across an await");
        // tokio::spawn(async {
        //     let shared = std::rc::Rc::new(5);
        //     tokio::task::yield_now().await;
        //     println!("{shared}");
        // });
        // error: future cannot be sent between threads safely
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async {
                let shared = std::rc::Rc::new(5);
                let task = tokio::task::spawn_local(async move {
                    tokio::task::yield_now().await;
                    *shared
                });
                println!(
                    "    local task kept Rc and returned {}",
                    task.await.unwrap()
                );
            })
            .await;

        println!("\nPitfall 4. Recursion with Box::pin");
        println!("    countdown: {:?}", countdown(5).await.unwrap());

        println!("\nPitfall 5. Await the operation so its body runs");
        // save_note(); // deliberate unused-Future warning
        save_note().await;
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn blocking_work_keeps_its_result_when_offloaded() {
        assert_eq!(blocking_moved_out().await, 42);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn a_blocking_job_does_not_occupy_the_async_scheduler_thread() {
        let (started, ready) = tokio::sync::oneshot::channel();
        let (release, wait) = std::sync::mpsc::channel();
        let job = tokio::task::spawn_blocking(move || {
            started.send(()).unwrap();
            wait.recv().unwrap(); // waits until this async task releases it
            42
        });
        ready.await.unwrap(); // this task must progress while the job is blocked
        let other = tokio::spawn(async { 7 });
        assert_eq!(other.await.unwrap(), 7);
        release.send(()).unwrap();
        assert_eq!(job.await.unwrap(), 42);
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
        assert_eq!(countdown(3).await.unwrap(), vec![3, 2, 1]);
        assert!(countdown(0).await.unwrap().is_empty());
        assert!(countdown(MAX_COUNTDOWN_DEPTH + 1).await.is_err());
    }
}
