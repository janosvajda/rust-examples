// Lesson 4: atomics.
//
// An atomic type (AtomicUsize, AtomicBool, …) is a single number or flag
// that many threads can update at once WITHOUT a lock. The CPU guarantees
// each operation happens all at once: no other thread can see it half-done.
// Every operation takes an `Ordering`, which says how it lines up with the
// thread's OTHER memory operations.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

// ---- 1. A counter without a lock ---------------------------------------------------------

/// `fetch_add` reads, adds and writes back as ONE indivisible step.
/// `Relaxed` is enough for a counter: we only care about the final total,
/// not about ordering relative to other data.
fn count_with_atomic(threads: usize, increments: usize) -> usize {
    let counter = Arc::new(AtomicUsize::new(0));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..increments {
                    counter.fetch_add(1, Ordering::Relaxed);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    counter.load(Ordering::Relaxed)
}

// ---- 2. A stop flag ------------------------------------------------------------------------

/// Workers check a shared flag and stop when it's set.
fn run_until_stopped() -> u64 {
    let stop = Arc::new(AtomicBool::new(false));
    let work_done = Arc::new(AtomicU64::new(0));
    let workers: Vec<_> = (0..3)
        .map(|_| {
            let stop = Arc::clone(&stop);
            let work_done = Arc::clone(&work_done);
            thread::spawn(move || {
                while !stop.load(Ordering::Relaxed) {
                    work_done.fetch_add(1, Ordering::Relaxed);
                    thread::sleep(Duration::from_millis(1));
                }
            })
        })
        .collect();
    thread::sleep(Duration::from_millis(50));
    stop.store(true, Ordering::Relaxed); // every worker sees this soon after
    for w in workers {
        w.join().unwrap();
    }
    work_done.load(Ordering::Relaxed)
}

// ---- 3. compare_exchange: update only if nobody else did -------------------------------------

/// Records the highest value seen by any thread. `fetch_max` exists, but
/// writing it by hand shows compare_exchange, the building block of every
/// lock-free algorithm: "set it to NEW, but only if it's still OLD".
fn record_max(highest: &AtomicU64, value: u64) {
    let mut current = highest.load(Ordering::Relaxed);
    while value > current {
        match highest.compare_exchange(current, value, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => return,                 // we won: it was still `current`
            Err(actual) => current = actual, // someone changed it: try again
        }
    }
}

fn highest_reading() -> u64 {
    let highest = Arc::new(AtomicU64::new(0));
    let handles: Vec<_> = (0..4u64)
        .map(|t| {
            let highest = Arc::clone(&highest);
            thread::spawn(move || {
                for i in 0..1000u64 {
                    record_max(&highest, (i * 7 + t * 13) % 997);
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    highest.load(Ordering::Relaxed)
}

// ---- 4. Release / Acquire: publishing data safely ----------------------------------------------
//
// One thread writes some data, then sets a "ready" flag. Another thread
// waits for the flag, then reads the data. For the reader to be guaranteed
// to see the data, the flag must be stored with Release and loaded with
// Acquire: everything written BEFORE the Release store is visible to the
// thread AFTER its Acquire load sees the flag.

fn publish_and_read() -> u64 {
    let data = Arc::new(AtomicU64::new(0));
    let ready = Arc::new(AtomicBool::new(false));

    let writer = {
        let (data, ready) = (Arc::clone(&data), Arc::clone(&ready));
        thread::spawn(move || {
            data.store(42, Ordering::Relaxed); // 1. write the data
            ready.store(true, Ordering::Release); // 2. publish: "data is ready"
        })
    };

    let reader = thread::spawn(move || {
        while !ready.load(Ordering::Acquire) {
            // 3. wait for the flag…
            std::hint::spin_loop();
        }
        data.load(Ordering::Relaxed) // 4. …then the data is guaranteed to be 42
    });

    writer.join().unwrap();
    reader.join().unwrap()
}

fn main() {
    println!("1. A lock-free counter");
    println!(
        "    8 threads × 100,000 increments = {}",
        count_with_atomic(8, 100_000)
    );

    println!("\n2. A stop flag");
    println!(
        "    3 workers did {} units of work before being stopped",
        run_until_stopped()
    );

    println!("\n3. compare_exchange");
    println!(
        "    highest reading across 4 threads: {}",
        highest_reading()
    );

    println!("\n4. Release / Acquire");
    println!("    the reader saw {}", publish_and_read());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_counter_loses_nothing() {
        assert_eq!(count_with_atomic(8, 50_000), 400_000);
    }

    #[test]
    fn stop_flag_stops_the_workers() {
        assert!(run_until_stopped() > 0); // returning at all means they stopped
    }

    #[test]
    fn compare_exchange_keeps_the_maximum() {
        let highest = AtomicU64::new(0);
        for v in [5, 3, 9, 1] {
            record_max(&highest, v);
        }
        assert_eq!(highest.load(Ordering::Relaxed), 9);
        assert_eq!(highest_reading(), 996);
    }

    #[test]
    fn acquire_sees_what_release_published() {
        for _ in 0..100 {
            assert_eq!(publish_and_read(), 42);
        }
    }
}
