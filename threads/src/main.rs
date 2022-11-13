use std::thread;
use std::time::Duration;
use yansi::Paint;

fn main() {
   
    thread::spawn(|| {
        for i in 1..101 {
            println!("{}{}", Paint::red("Hello from first spawn: "), Paint::red(i));
            thread::sleep(Duration::from_millis(1000)); // wait 1 second
        }
    });

    thread::spawn(|| {
        for i in 1..101 {
            println!("{}{}", Paint::green("Hello from second spawn: "), Paint::green(i));
            thread::sleep(Duration::from_millis(10)); // wait 10 milliseconds
        }
    });

    for i in 1..155 {
        println!("{}{}", Paint::blue("Hello from main"), Paint::blue(i));
        thread::sleep(Duration::from_millis(1)); // the for in the app's main thread wait 1 milliseconds
    }
}
