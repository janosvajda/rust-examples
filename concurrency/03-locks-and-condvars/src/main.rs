// Lesson 3: locks and condition variables.
//
// When threads must share data rather than send it:
//   Arc<T>        shared ownership across threads
//   Mutex<T>      one thread at a time may access the data
//   RwLock<T>     many readers OR one writer
//   Condvar       sleep until another thread says "something changed"
//   Barrier       wait until N threads have all reached the same point
//   File::lock    a lock on a file, shared with other PROGRAMS, not only threads

use std::collections::VecDeque;
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, Barrier, Condvar, Mutex, RwLock};
use std::thread;
use std::time::Duration;

// ---- 1. Mutex: one at a time ---------------------------------------------------------------
//
// Moving a plain Mutex into several threads doesn't work: the first thread
// takes ownership.
//     error[E0382]: use of moved value: `counter`
// Arc gives every thread its own handle to the same Mutex.

fn count_with_mutex(threads: usize, increments: usize) -> usize {
    let counter = Arc::new(Mutex::new(0));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..increments {
                    // `lock()` waits for its turn and returns a guard. The
                    // lock is released when the guard is dropped (end of line).
                    *counter.lock().unwrap() += 1;
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    *counter.lock().unwrap()
}

// ---- 2. Poisoning: a lock held during a panic ------------------------------------------------

/// If a thread panics while holding the lock, the data might be half-updated.
/// Rust marks the Mutex as POISONED: `lock()` then returns an error, and
/// `.unwrap()` on it panics with:
///     called `Result::unwrap()` on an `Err` value: PoisonError { .. }
fn poisoned_lock_demo() -> (bool, i32) {
    let balance = Arc::new(Mutex::new(100));
    let b = Arc::clone(&balance);
    let _ = thread::spawn(move || {
        let mut guard = b.lock().unwrap();
        *guard -= 30; // first half of an update…
        panic!("crashed mid-update"); // …and the second half never happens
    })
    .join();

    let poisoned = balance.is_poisoned();
    // You can still get the data if you decide it's usable anyway.
    let value = match balance.lock() {
        Ok(guard) => *guard,
        Err(poisoned) => *poisoned.into_inner(),
    };
    (poisoned, value)
}

// ---- 3. RwLock: many readers or one writer ---------------------------------------------------

fn read_mostly_config() -> Vec<String> {
    let config = Arc::new(RwLock::new(String::from("theme=dark")));
    let mut handles = Vec::new();
    for id in 0..3 {
        let config = Arc::clone(&config);
        handles.push(thread::spawn(move || {
            let current = config.read().unwrap(); // readers don't block each other
            format!("reader {id} sees {current}")
        }));
    }
    let mut lines: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    config.write().unwrap().push_str(";font=large"); // a writer waits for all readers
    lines.push(format!("after the write: {}", config.read().unwrap()));
    lines
}

// ---- 4. Condvar: wait until there's something to do -------------------------------------------

/// A queue that consumers can wait on without spinning. `wait` releases
/// the lock while sleeping and takes it back when woken.
struct JobQueue {
    jobs: Mutex<VecDeque<u32>>,
    not_empty: Condvar,
}

impl JobQueue {
    fn push(&self, job: u32) {
        self.jobs.lock().unwrap().push_back(job);
        self.not_empty.notify_one(); // wake one waiting consumer
    }

    fn pop(&self) -> u32 {
        let mut jobs = self.jobs.lock().unwrap();
        // Always wait in a loop: a thread can wake up without anything
        // having changed (a "spurious wakeup"). `wait_while` does the loop.
        jobs = self
            .not_empty
            .wait_while(jobs, |jobs| jobs.is_empty())
            .unwrap();
        jobs.pop_front().unwrap()
    }
}

fn condvar_demo() -> Vec<u32> {
    let queue = Arc::new(JobQueue {
        jobs: Mutex::new(VecDeque::new()),
        not_empty: Condvar::new(),
    });
    let consumer = {
        let queue = Arc::clone(&queue);
        thread::spawn(move || (0..3).map(|_| queue.pop()).collect::<Vec<_>>())
    };
    for job in [10, 20, 30] {
        thread::sleep(Duration::from_millis(20)); // the consumer is asleep meanwhile
        queue.push(job);
    }
    consumer.join().unwrap()
}

// ---- 5. Barrier: everyone waits for everyone --------------------------------------------------

fn barrier_demo() -> Vec<String> {
    let barrier = Arc::new(Barrier::new(3));
    let log = Arc::new(Mutex::new(Vec::new()));
    let handles: Vec<_> = (0..3)
        .map(|id| {
            let barrier = Arc::clone(&barrier);
            let log = Arc::clone(&log);
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(20 * id)); // finish phase 1 at different times
                log.lock().unwrap().push(format!("phase 1 done by {id}"));
                barrier.wait(); // nobody continues until all 3 are here
                log.lock().unwrap().push(format!("phase 2 started by {id}"));
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    Arc::try_unwrap(log).unwrap().into_inner().unwrap()
}

// ---- 6. File locks: coordinating with other programs --------------------------------
//
// A Mutex only works inside one program. A file lock is managed by the
// operating system, so it also works between separate programs (or several
// copies of the same program) using the same file.

/// Append one line to a shared log file. The exclusive lock makes sure no
/// other writer, in this program or another one, writes at the same moment.
fn append_line(path: &Path, line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.lock()?; // waits until no one else holds a lock on the file
    writeln!(file, "{line}")?;
    Ok(()) // closing the file (dropping it) releases the lock
}

fn file_lock_demo() -> io::Result<(usize, String)> {
    let path = std::env::temp_dir().join("rust-examples-shared.log");
    File::create(&path)?; // start empty

    // Every thread opens the file itself, just like separate programs would.
    let writers: Vec<_> = (0..4)
        .map(|id| {
            let path = path.clone();
            thread::spawn(move || {
                for n in 0..25 {
                    append_line(&path, &format!("writer {id}, line {n}")).expect("can write");
                }
            })
        })
        .collect();
    for writer in writers {
        writer.join().expect("writer finished");
    }
    let lines = std::fs::read_to_string(&path)?.lines().count();

    // try_lock doesn't wait: it says right away whether someone else holds the lock.
    let holder = OpenOptions::new().append(true).open(&path)?;
    holder.lock()?;
    let other = OpenOptions::new().append(true).open(&path)?;
    let attempt = format!("{:?}", other.try_lock());
    drop(holder);
    std::fs::remove_file(&path)?;
    Ok((lines, attempt))
}

fn main() {
    println!("1. Mutex");
    println!(
        "    8 threads × 10,000 increments = {}",
        count_with_mutex(8, 10_000)
    );

    println!("\n2. Poisoning");
    let (poisoned, value) = poisoned_lock_demo();
    println!("    poisoned: {poisoned}; data left behind: {value} (the update stopped halfway)");

    println!("\n3. RwLock");
    for line in read_mostly_config() {
        println!("    {line}");
    }

    println!("\n4. Condvar: a consumer sleeps until jobs arrive");
    println!("    consumer received {:?}", condvar_demo());

    println!("\n5. Barrier: all threads finish phase 1 before any starts phase 2");
    for line in barrier_demo() {
        println!("    {line}");
    }

    println!("\n6. File locks: shared with other programs");
    match file_lock_demo() {
        Ok((lines, attempt)) => {
            println!("    4 writers × 25 lines → {lines} complete lines in the log");
            println!("    try_lock while another handle holds the lock: {attempt}");
        }
        Err(error) => println!("    file locks unavailable here: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutex_counter_loses_nothing() {
        assert_eq!(count_with_mutex(4, 5_000), 20_000);
    }

    #[test]
    fn a_panic_while_locked_poisons_the_mutex() {
        let (poisoned, value) = poisoned_lock_demo();
        assert!(poisoned);
        assert_eq!(value, 70);
    }

    #[test]
    fn rwlock_readers_then_writer() {
        let lines = read_mostly_config();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[3], "after the write: theme=dark;font=large");
    }

    #[test]
    fn condvar_consumer_gets_every_job_in_order() {
        assert_eq!(condvar_demo(), [10, 20, 30]);
    }

    #[test]
    fn barrier_separates_the_phases() {
        let log = barrier_demo();
        let last_phase_one = log.iter().rposition(|l| l.starts_with("phase 1")).unwrap();
        let first_phase_two = log.iter().position(|l| l.starts_with("phase 2")).unwrap();
        assert!(last_phase_one < first_phase_two);
    }

    #[test]
    fn a_file_lock_blocks_other_handles_until_released() {
        let path = std::env::temp_dir().join("rust-examples-lock-test.txt");
        let first = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .unwrap();
        let second = OpenOptions::new().append(true).open(&path).unwrap();
        first.lock().unwrap();
        assert!(second.try_lock().is_err()); // someone else holds it
        first.unlock().unwrap();
        assert!(second.try_lock().is_ok()); // free again
        drop((first, second));
        std::fs::remove_file(&path).unwrap();
    }
}
