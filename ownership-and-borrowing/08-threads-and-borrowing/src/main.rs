// Lesson 8: threads and borrowing.
//
// Borrowing, thread-safety traits and synchronisation APIs prevent data races
// in safe Rust. Locks and atomics provide coordinated shared mutation.
// Two marker traits describe permissions across threads:
//   Send  a value of this type can be MOVED to another thread.
//   Sync  a value of this type can be SHARED (&T) between threads.
// The compiler checks both automatically.

use std::sync::{Arc, Mutex, RwLock};
use std::thread;

fn main() {
    println!("1. thread::spawn cannot hold a borrow of these short-lived locals…");
    let names = [String::from("Ana"), String::from("Bob")];
    // let handle = thread::spawn(|| println!("{names:?}"));
    // error[E0373]: closure may outlive the current function, but it borrows `names`,
    //               which is owned by the current function
    // The new thread could keep running after `names` is dropped.
    let handle = thread::spawn(move || names.len()); // fix 1: move ownership into the thread
    println!(
        "    the thread took ownership and counted {} names",
        handle.join().unwrap()
    );

    println!("\n2. …but scoped threads can");
    let scores = vec![70, 85, 90, 100];
    let (sum, max) = thread::scope(|s| {
        // Threads started through this scope's s.spawn are joined before
        // scope returns, so they can borrow scores, even several at once.
        // An ordinary thread::spawn here would still be unscoped.
        let sum = s.spawn(|| scores.iter().sum::<i32>());
        let max = s.spawn(|| scores.iter().max().copied());
        (sum.join().unwrap(), max.join().unwrap())
    });
    println!("    sum = {sum}, max = {max:?}, and `scores` is still ours: {scores:?}");

    println!("\n3. Scoped threads follow the borrowing rules: one writer per value");
    let mut data = vec![1, 2, 3, 4, 5, 6];
    // let mut total = 0; // uncomment with the two s.spawn calls below
    thread::scope(|s| {
        // `chunks_mut` splits the Vec into separate mutable slices, so each
        // thread gets its own part. Two threads writing to the SAME value
        // would be rejected:
        // s.spawn(|| total += 1);
        // s.spawn(|| total += 1);
        // error[E0499]: cannot borrow `total` as mutable more than once at a time
        for chunk in data.chunks_mut(2) {
            s.spawn(move || {
                for x in chunk {
                    *x *= 10;
                }
            });
        }
    });
    println!("    data = {data:?}");

    println!("\n4. Shared ownership across threads: Arc instead of Rc");
    // use std::rc::Rc;
    // let shared = Rc::new(5);
    // thread::spawn(move || println!("{shared}"));
    // error[E0277]: `Rc<i32>` cannot be sent between threads safely
    // Rc's reference count isn't updated atomically, so two threads could
    // corrupt it. Arc ("atomically reference counted") can be.
    let config = Arc::new(String::from("shared settings"));
    let workers: Vec<_> = (0..3)
        .map(|id| {
            let config = Arc::clone(&config); // each thread gets its own Arc
            thread::spawn(move || format!("worker {id} sees \"{config}\""))
        })
        .collect();
    for worker in workers {
        println!("    {}", worker.join().unwrap());
    }

    println!("\n5. Shared mutation across threads: Mutex instead of RefCell");
    // use std::cell::RefCell;
    // let shared = Arc::new(RefCell::new(0));
    // thread::spawn(move || *shared.borrow_mut() += 1);
    // error[E0277]: `RefCell<i32>` cannot be shared between threads safely
    // RefCell's borrow counter isn't thread-safe. A Mutex makes other
    // threads WAIT for their turn instead.
    let counter = Arc::new(Mutex::new(0));
    let handles: Vec<_> = (0..8)
        .map(|_| {
            let counter = Arc::clone(&counter);
            thread::spawn(move || {
                for _ in 0..1000 {
                    // lock() waits for coordinated exclusive access. The lock is
                    // released when the guard is dropped, at the end of the line.
                    *counter.lock().unwrap() += 1;
                }
            })
        })
        .collect();
    for handle in handles {
        handle.join().unwrap();
    }
    println!(
        "    8 threads × 1000 increments = {}",
        *counter.lock().unwrap()
    );

    println!("\n6. RwLock: many readers OR one writer, across threads");
    let settings = RwLock::new(String::from("dark mode"));
    {
        let a = settings.read().unwrap(); // several readers at once: fine
        let b = settings.read().unwrap();
        println!("    two readers: {a}, {b}");
    } // readers released
    settings.write().unwrap().push_str(", large font"); // one writer
    println!("    after writing: {}", settings.read().unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scoped_threads_can_borrow_local_data() {
        let words = ["a", "bb", "ccc"];
        let total = thread::scope(|s| {
            let lengths: Vec<_> = words.iter().map(|w| s.spawn(move || w.len())).collect();
            lengths
                .into_iter()
                .map(|h| h.join().unwrap())
                .sum::<usize>()
        });
        assert_eq!(total, 6);
        assert_eq!(words.len(), 3); // still ours
    }

    #[test]
    fn each_thread_can_change_its_own_part() {
        let mut v = vec![1, 1, 1, 1];
        thread::scope(|s| {
            for (i, chunk) in v.chunks_mut(1).enumerate() {
                s.spawn(move || chunk[0] += i);
            }
        });
        assert_eq!(v, vec![1, 2, 3, 4]);
    }

    #[test]
    fn mutex_counter_is_exact() {
        let counter = Arc::new(Mutex::new(0));
        let handles: Vec<_> = (0..4)
            .map(|_| {
                let c = Arc::clone(&counter);
                thread::spawn(move || {
                    for _ in 0..500 {
                        *c.lock().unwrap() += 1;
                    }
                })
            })
            .collect();
        handles.into_iter().for_each(|h| h.join().unwrap());
        assert_eq!(*counter.lock().unwrap(), 2000);
    }

    #[test]
    fn rwlock_allows_several_readers() {
        let lock = RwLock::new(1);
        let a = lock.read().unwrap();
        let b = lock.read().unwrap();
        assert_eq!(*a + *b, 2);
        assert!(lock.try_write().is_err()); // no writer while reading
    }
}
