// Lesson 1: threads.
//
// A thread is an independent path of execution. The operating system runs
// threads at the same time on different CPU cores (or takes turns on one).
//   thread::spawn(closure)   starts a new thread running the closure
//   handle.join()            waits for it to finish and gets its result

use std::thread;
use std::time::{Duration, Instant};

/// Parses a setting, and panics if it isn't a number: a stand-in for any
/// bug that crashes a thread.
fn parse_setting(text: &str) -> u32 {
    text.trim().parse().expect("the setting must be a number")
}

/// Some work that takes a while, to see threads overlap.
fn slow_square(n: u64) -> u64 {
    thread::sleep(Duration::from_millis(200));
    n * n
}

fn main() {
    println!("1. Spawning a thread and waiting for it");
    let handle = thread::spawn(|| {
        for i in 1..=3 {
            println!("    worker: step {i}");
            thread::sleep(Duration::from_millis(10));
        }
        "worker finished" // the thread's result
    });
    println!("    main: the worker is running in the background");
    let result = handle.join().unwrap(); // blocks until the thread ends
    println!("    main: got \"{result}\"");

    println!("\n2. Several threads at once");
    let start = Instant::now();
    let handles: Vec<_> = (1..=4)
        .map(|n| thread::spawn(move || slow_square(n)))
        .collect();
    let squares: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    println!(
        "    {squares:?}: four 200 ms jobs took {} ms, not 800",
        start.elapsed().as_millis()
    );

    println!("\n3. Threads take ownership of what they use: `move`");
    let names = [String::from("Ana"), String::from("Bob")];
    let greeter = thread::spawn(move || {
        // `names` now belongs to this thread. Without `move`:
        //     error[E0373]: closure may outlive the current function, but it borrows `names`
        names.iter().map(|n| format!("hi {n}")).collect::<Vec<_>>()
    });
    println!("    {:?}", greeter.join().unwrap());
    // println!("{names:?}");   // error[E0382]: borrow of moved value: `names`

    println!("\n4. Scoped threads can borrow instead");
    let scores = [70, 85, 90, 100];
    let (low, high) = scores.split_at(2);
    let (sum_low, sum_high) = thread::scope(|s| {
        let a = s.spawn(|| low.iter().sum::<i32>());
        let b = s.spawn(|| high.iter().sum::<i32>());
        (a.join().unwrap(), b.join().unwrap())
    }); // every scoped thread has finished here, so borrowing was safe
    println!(
        "    halves: {sum_low} + {sum_high} = {}, and scores is still ours",
        sum_low + sum_high
    );

    println!("\n5. A panic stays inside its thread");
    let crashing = thread::Builder::new()
        .name(String::from("unlucky-worker"))
        .spawn(|| parse_setting("not a number"))
        .unwrap();
    // The panic message is printed by Rust, then join() returns Err instead
    // of crashing this thread too.
    match crashing.join() {
        Ok(value) => println!("    worker succeeded: {value}"),
        Err(_) => println!("    the worker panicked, but main carries on"),
    }

    println!("\n6. How many threads is sensible?");
    let cores = thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    println!("    this machine can run {cores} threads in parallel");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn join_returns_the_threads_result() {
        let handle = thread::spawn(|| 6 * 7);
        assert_eq!(handle.join().unwrap(), 42);
    }

    #[test]
    fn threads_run_in_parallel() {
        let start = Instant::now();
        let handles: Vec<_> = (1..=4)
            .map(|n| thread::spawn(move || slow_square(n)))
            .collect();
        let results: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results, [1, 4, 9, 16]);
        // Sequentially this would take 800 ms. Allow plenty of slack.
        assert!(start.elapsed() < Duration::from_millis(700));
    }

    #[test]
    fn scoped_threads_borrow_local_data() {
        let data = [1, 2, 3, 4];
        let total: i32 = thread::scope(|s| {
            let handles: Vec<_> = data
                .chunks(2)
                .map(|c| s.spawn(move || c.iter().sum::<i32>()))
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).sum()
        });
        assert_eq!(total, 10);
        assert_eq!(data.len(), 4); // still ours
    }

    #[test]
    fn a_panicking_thread_returns_err_from_join() {
        let handle = thread::spawn(|| panic!("boom"));
        assert!(handle.join().is_err());
    }

    #[test]
    fn named_threads() {
        let handle = thread::Builder::new()
            .name(String::from("helper"))
            .spawn(|| thread::current().name().map(str::to_string))
            .unwrap();
        assert_eq!(handle.join().unwrap().as_deref(), Some("helper"));
    }
}
