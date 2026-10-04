// Lesson 4: Arc<T>, several owners across threads.
//
// Arc ("atomically reference counted") is Rc for threads: the same idea of
// counting owners, but the counter is changed with atomic CPU instructions,
// so two threads can clone and drop it at the same time safely.
//
//     let shared = Rc::new(5);
//     std::thread::spawn(move || println!("{shared}"));
//     error[E0277]: `Rc<i32>` cannot be sent between threads safely

use std::sync::{Arc, Mutex, RwLock};
use std::thread;

// ---- 1. Counting owners ----------------------------------------------------------------------

fn owners_while_cloning() -> (usize, usize, usize) {
    let data = Arc::new(String::from("shared"));
    let at_start = Arc::strong_count(&data);
    let second = Arc::clone(&data);
    let with_two = Arc::strong_count(&data);
    drop(second);
    (at_start, with_two, Arc::strong_count(&data))
}

// ---- 2. Read-only data, shared by many threads -------------------------------------------

/// Each thread sums its own part of ONE Vec. Nothing is copied: every thread
/// gets an Arc pointing to the same numbers.
fn sum_in_threads(numbers: Arc<Vec<i32>>, thread_count: usize) -> i32 {
    let chunk = numbers.len().div_ceil(thread_count).max(1);
    let handles: Vec<_> = (0..thread_count)
        .map(|i| {
            let numbers = Arc::clone(&numbers); // one more owner, for this thread
            thread::spawn(move || {
                let start = (i * chunk).min(numbers.len());
                let end = (start + chunk).min(numbers.len());
                numbers[start..end].iter().sum::<i32>()
            })
        })
        .collect();
    handles.into_iter().map(|h| h.join().expect("a worker panicked")).sum()
}

// ---- 3. Data that changes: Arc<Mutex<T>> --------------------------------------------------

/// Arc alone gives read-only access (like Rc). To change shared data, wrap it
/// in a Mutex: one thread at a time may lock it and change it.
fn count_in_threads(thread_count: usize, increments: usize) -> usize {
    let counter = Arc::new(Mutex::new(0));
    let handles: Vec<_> = (0..thread_count)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..increments {
                    *counter.lock().expect("no thread panicked while holding it") += 1;
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().expect("a worker panicked");
    }
    *counter.lock().expect("no thread panicked while holding it")
}

// ---- 4. Mostly read, rarely written: Arc<RwLock<T>> ---------------------------------------

/// Many readers at once, or one writer: good for settings that change rarely.
fn read_while_one_writes() -> Vec<String> {
    let settings = Arc::new(RwLock::new(String::from("light mode")));
    let writer = {
        let settings = Arc::clone(&settings);
        thread::spawn(move || *settings.write().expect("not poisoned") = String::from("dark mode"))
    };
    writer.join().expect("the writer panicked");
    let readers: Vec<_> = (0..3)
        .map(|i| {
            let settings = Arc::clone(&settings);
            thread::spawn(move || format!("reader {i} sees {}", settings.read().expect("not poisoned")))
        })
        .collect();
    readers.into_iter().map(|r| r.join().expect("a reader panicked")).collect()
}

fn main() {
    println!("1. Counting owners");
    let (start, two, after) = owners_while_cloning();
    println!("    at start {start}, after a clone {two}, after dropping it {after}");

    println!("\n2. Read-only data shared by 4 threads, without copying");
    let numbers = Arc::new((1..=100).collect::<Vec<i32>>());
    println!("    sum of 1..=100: {}", sum_in_threads(numbers, 4));

    println!("\n3. Arc<Mutex<T>>: 8 threads change one counter");
    println!("    8 × 1000 increments = {}", count_in_threads(8, 1000));

    println!("\n4. Arc<RwLock<T>>: one writer, many readers");
    for line in read_while_one_writes() {
        println!("    {line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_count_follows_the_clones() {
        assert_eq!(owners_while_cloning(), (1, 2, 1));
    }

    #[test]
    fn threads_share_one_vec() {
        let numbers = Arc::new((1..=100).collect::<Vec<i32>>());
        assert_eq!(sum_in_threads(Arc::clone(&numbers), 3), 5050);
        assert_eq!(Arc::strong_count(&numbers), 1); // every thread's clone is gone again
        // more threads than items: the extra threads get empty slices
        assert_eq!(sum_in_threads(Arc::new(vec![1, 2]), 5), 3);
    }

    #[test]
    fn no_increment_is_lost() {
        assert_eq!(count_in_threads(8, 1000), 8000);
    }

    #[test]
    fn every_reader_sees_the_written_value() {
        assert!(read_while_one_writes().iter().all(|line| line.ends_with("dark mode")));
    }
}
