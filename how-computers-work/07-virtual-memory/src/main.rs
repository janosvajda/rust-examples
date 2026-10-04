// Lesson 7: virtual memory.
//
// Every program gets its own private, pretend address space. The processor
// translates each "virtual" address to a real place in memory, page by page,
// using tables the operating system controls. This program shows three
// consequences, using copies of itself as child processes so that the crashes
// in part 2 happen in a child, never in the lesson itself.

use std::process::Command;
use std::time::Instant;

static COUNTER: u64 = 7;

/// What a child process does, chosen by its first argument.
fn child(task: &str) {
    match task {
        "addresses" => {
            let local = 0u8;
            let boxed = Box::new(0u8);
            println!(
                "code {:p}  static {:p}  stack {:p}  heap {:p}",
                child as *const u8, &COUNTER, &local, &*boxed
            );
        }
        "read-address-16" => {
            // Address 16 is in the lowest page, which the operating system
            // deliberately leaves unmapped: no program may use it.
            // SAFETY: none: this is the bug being demonstrated, in a child process.
            let value = unsafe { std::ptr::read_volatile(16 as *const u8) };
            println!("unexpectedly read {value}");
        }
        "write-to-a-constant" => {
            // String literals are in read-only memory.
            let text: &'static str = "can't touch this";
            // SAFETY: none: writing to read-only memory, in a child process.
            unsafe { std::ptr::write_volatile(text.as_ptr() as *mut u8, b'X') };
            println!("unexpectedly wrote to a constant");
        }
        _ => eprintln!("unknown task {task}"),
    }
}

/// Runs this same program again as a child process, doing `task`.
/// Returns its output, or a description of how it was stopped.
fn run_child(task: &str) -> String {
    let exe = std::env::current_exe().expect("knows its own path");
    let output = Command::new(exe).arg(task).output().expect("can start a child process");
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    describe_failure(output.status)
}

#[cfg(unix)]
fn describe_failure(status: std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    match status.signal() {
        Some(11) => String::from("stopped by the operating system: signal 11, SIGSEGV (segmentation fault)"),
        Some(10) if cfg!(target_os = "macos") => String::from("stopped by the operating system: signal 10, SIGBUS (bus error)"),
        Some(7) if cfg!(target_os = "linux") => String::from("stopped by the operating system: signal 7, SIGBUS (bus error)"),
        Some(signal) => format!("stopped by the operating system: signal {signal}"),
        None => format!("failed: {status}"),
    }
}

#[cfg(not(unix))]
fn describe_failure(status: std::process::ExitStatus) -> String {
    format!("stopped by the operating system: {status} (an access violation on Windows)")
}

fn main() {
    if let Some(task) = std::env::args().nth(1) {
        return child(&task);
    }

    println!("1. Every run gets different addresses (address space layout randomisation)");
    for run in 1..=3 {
        println!("    run {run}: {}", run_child("addresses"));
    }

    println!("\n2. Touching memory you don't own: the operating system stops the program");
    println!("    reading address 16:            {}", run_child("read-address-16"));
    println!("    writing to a string constant:  {}", run_child("write-to-a-constant"));

    println!("\n3. Memory is handed out when it's first used, page by page");
    let size = 1 << 30; // 1 GB
    let start = Instant::now();
    let mut memory = vec![0u8; size];
    let allocated = start.elapsed();
    let start = Instant::now();
    for offset in (0..size).step_by(4096) {
        memory[offset] = 1; // the first touch of each page
    }
    let touched = start.elapsed();
    std::hint::black_box(&memory);
    println!("    asking for 1 GB of zeroed memory: {allocated:.2?}");
    println!("    then touching every page once:    {touched:.2?}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn statics_live_at_a_fixed_place_within_one_run() {
        let a = &super::COUNTER as *const u64;
        let b = &super::COUNTER as *const u64;
        assert_eq!(a, b);
        assert_eq!(super::COUNTER, 7);
    }
}
