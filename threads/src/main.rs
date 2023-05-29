use std::thread;
use std::time::Duration;
use yansi::Paint;

fn main() {
    thread::spawn(|| {
        for i in 1..=100 {
            println!("{}{}", Paint::red("Hello from first spawn: "), Paint::red(i));
            thread::sleep(Duration::from_millis(1000)); // wait 1 second
        }
    });

    thread::spawn(|| {
        for i in 1..=100 {
            println!("{}{}", Paint::green("Hello from second spawn: "), Paint::green(i));
            thread::sleep(Duration::from_millis(10)); // wait 10 milliseconds
        }
    });

    for i in 1..=154 {
        println!("{}{}", Paint::blue("Hello from main"), Paint::blue(i));
        thread::sleep(Duration::from_millis(1)); // wait 1 millisecond
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello_threads() {
        // Invoke the main logic in a separate thread for testing
        let handle = thread::spawn(|| {
            main();
        });

        // Wait for a sufficient time to ensure completion of thread executions
        thread::sleep(Duration::from_secs(3));

        // No assertions needed, as we're testing the behavior visually
        // Inspect the console output manually to verify the expected behavior

        // Join the main thread
        handle.join().unwrap();
    }
}
