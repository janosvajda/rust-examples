// Lesson 2: message passing.
//
// Instead of sharing data, threads can SEND data to each other through a
// channel. A channel has two ends:
//   Sender<T>     put messages in (can be cloned: many producers)
//   Receiver<T>   take messages out (only one: a single consumer)
// "Do not communicate by sharing memory; share memory by communicating."

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

// ---- 1. One producer, one consumer ------------------------------------------------------

fn countdown_over_channel() -> Vec<u32> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        for n in (1..=3).rev() {
            sender.send(n).unwrap(); // the value MOVES into the channel
            thread::sleep(Duration::from_millis(20));
        }
    }); // the thread ends and its sender is dropped → the channel closes
    // Iterating a receiver waits for each message, and stops when every
    // sender has been dropped.
    receiver.iter().collect()
}

// ---- 2. Many producers --------------------------------------------------------------------

#[derive(Debug)]
struct Reading {
    sensor: &'static str,
    value: f64,
}

fn collect_readings() -> Vec<String> {
    let (sender, receiver) = mpsc::channel();
    for (sensor, base) in [("kitchen", 21.0), ("garage", 12.0), ("attic", 28.0)] {
        let sender = sender.clone(); // each producer gets its own Sender
        thread::spawn(move || {
            for i in 0..2 {
                sender
                    .send(Reading {
                        sensor,
                        value: base + i as f64,
                    })
                    .unwrap();
            }
        });
    }
    drop(sender); // drop the original, or the receiver would wait forever
    let mut lines: Vec<String> = receiver
        .iter()
        .map(|r| format!("{}: {:.1} °C", r.sensor, r.value))
        .collect();
    lines.sort(); // arrival order varies between runs
    lines
}

// ---- 3. A bounded channel: backpressure ---------------------------------------------------

/// `sync_channel(2)` holds at most 2 messages. When it's full, `send`
/// WAITS until the consumer catches up, so a fast producer can't fill memory.
fn bounded_channel_demo() -> Vec<String> {
    let (sender, receiver) = mpsc::sync_channel(2);
    let start = std::time::Instant::now();
    let producer = thread::spawn(move || {
        let mut log = Vec::new();
        for job in 1..=5 {
            sender.send(job).unwrap(); // blocks while 2 messages are waiting
            log.push(format!(
                "sent {job} after {:>3} ms",
                start.elapsed().as_millis()
            ));
        }
        log
    });
    let mut received = Vec::new();
    for job in receiver.iter() {
        thread::sleep(Duration::from_millis(30)); // a slow consumer
        received.push(job);
    }
    let mut log = producer.join().unwrap();
    log.push(format!("received {received:?}"));
    log
}

// ---- 4. A worker pool: several consumers sharing one queue --------------------------------
//
// A Receiver can't be shared between threads directly:
//     error[E0277]: `std::sync::mpsc::Receiver<u32>` cannot be shared between threads safely
// Wrapping it in Arc<Mutex<…>> lets workers take turns pulling jobs.

fn worker_pool(jobs: Vec<u64>, workers: usize) -> Vec<(usize, u64, u64)> {
    let (job_sender, job_receiver) = mpsc::channel::<u64>();
    let (result_sender, result_receiver) = mpsc::channel();
    let job_receiver = Arc::new(Mutex::new(job_receiver));

    for id in 0..workers {
        let job_receiver = Arc::clone(&job_receiver);
        let result_sender = result_sender.clone();
        thread::spawn(move || {
            loop {
                // Lock only long enough to take one job, then release it.
                let job = job_receiver.lock().unwrap().recv();
                match job {
                    Ok(n) => {
                        thread::sleep(Duration::from_millis(10)); // "work"
                        result_sender.send((id, n, n * n)).unwrap();
                    }
                    Err(_) => break, // channel closed: no more jobs
                }
            }
        });
    }
    drop(result_sender);

    for job in jobs {
        job_sender.send(job).unwrap();
    }
    drop(job_sender); // close the queue so the workers stop

    let mut results: Vec<_> = result_receiver.iter().collect();
    results.sort_by_key(|&(_, job, _)| job);
    results
}

fn main() {
    println!("1. One producer, one consumer");
    println!("    received {:?}", countdown_over_channel());

    println!("\n2. Many producers, one consumer");
    for line in collect_readings() {
        println!("    {line}");
    }

    println!("\n3. A bounded channel makes a fast producer wait");
    for line in bounded_channel_demo() {
        println!("    {line}");
    }

    println!("\n4. A worker pool");
    for (worker, job, result) in worker_pool((1..=8).collect(), 3) {
        println!("    job {job} → {result} (done by worker {worker})");
    }

    println!("\n5. Waiting with a timeout");
    let (_sender, receiver) = mpsc::channel::<u32>();
    match receiver.recv_timeout(Duration::from_millis(100)) {
        Ok(n) => println!("    got {n}"),
        Err(e) => println!("    nothing arrived: {e}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_arrive_in_order_from_one_producer() {
        assert_eq!(countdown_over_channel(), [3, 2, 1]);
    }

    #[test]
    fn every_producers_messages_arrive() {
        assert_eq!(collect_readings().len(), 6);
    }

    #[test]
    fn bounded_channel_delivers_everything() {
        let log = bounded_channel_demo();
        assert_eq!(log.last().unwrap(), "received [1, 2, 3, 4, 5]");
    }

    #[test]
    fn worker_pool_processes_every_job_once() {
        let results = worker_pool((1..=20).collect(), 4);
        let jobs: Vec<u64> = results.iter().map(|&(_, job, _)| job).collect();
        assert_eq!(jobs, (1..=20).collect::<Vec<_>>());
        assert!(results.iter().all(|&(_, n, sq)| sq == n * n));
    }

    #[test]
    fn recv_reports_a_closed_channel() {
        let (sender, receiver) = mpsc::channel::<u8>();
        drop(sender);
        assert!(receiver.recv().is_err());
    }
}
