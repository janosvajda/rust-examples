// Lesson 7: virtual memory.
//
// Each ordinary process has its own virtual address space. The processor
// translates virtual addresses to places in RAM, page by page,
// using tables the operating system controls. This program shows three
// consequences, starting fresh instances of itself as child processes.
// Part 2 deliberately violates Rust's rules: its usual crashes illustrate
// OS protection, but undefined behaviour has no guaranteed outcome in Rust.

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
            // Address 16 is in the lowest virtual page, which desktop operating
            // systems normally leave unmapped to catch null-pointer bugs.
            // DELIBERATE UNDEFINED BEHAVIOUR: this pointer is not valid for reads.
            // `volatile` does not make an invalid access valid. Do not copy this
            // into normal code; even in a child, Rust does not guarantee a crash.
            let value = unsafe { std::ptr::read_volatile(16 as *const u8) };
            println!("unexpectedly read {value}");
        }
        "write-to-a-constant" => {
            // String literals normally live in pages marked read-only.
            let text: &'static str = "can't touch this";
            // DELIBERATE UNDEFINED BEHAVIOUR: mutating a string literal is invalid
            // in Rust, regardless of the OS page permissions. Usually the OS
            // stops this child, but `unsafe` does not guarantee that outcome.
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
    let output = Command::new(exe)
        .arg(task)
        .output()
        .expect("can start a child process");
    if output.status.success() {
        return String::from_utf8_lossy(&output.stdout).trim().to_string();
    }
    describe_failure(output.status)
}

#[cfg(unix)]
fn describe_failure(status: std::process::ExitStatus) -> String {
    use std::os::unix::process::ExitStatusExt;
    match status.signal() {
        Some(11) => {
            String::from("stopped by the operating system: signal 11, SIGSEGV (segmentation fault)")
        }
        Some(10) if cfg!(target_os = "macos") => {
            String::from("stopped by the operating system: signal 10, SIGBUS (bus error)")
        }
        Some(7) if cfg!(target_os = "linux") => {
            String::from("stopped by the operating system: signal 7, SIGBUS (bus error)")
        }
        Some(signal) => format!("stopped by the operating system: signal {signal}"),
        None => format!("failed: {status}"),
    }
}

#[cfg(windows)]
fn describe_failure(status: std::process::ExitStatus) -> String {
    if status.code() == Some(0xC000_0005u32 as i32) {
        format!("stopped by the operating system: {status} (access violation)")
    } else {
        format!("failed: {status}")
    }
}

#[cfg(not(any(unix, windows)))]
fn describe_failure(status: std::process::ExitStatus) -> String {
    format!("failed: {status}")
}

/// Ask the OS for its base page size instead of assuming 4096 bytes.
#[cfg(unix)]
fn system_page_size() -> Option<usize> {
    unsafe extern "C" {
        fn getpagesize() -> std::ffi::c_int;
    }
    // SAFETY: getpagesize takes no arguments and returns the OS page size.
    let bytes = unsafe { getpagesize() };
    usize::try_from(bytes).ok().filter(|&size| size > 0)
}

#[cfg(windows)]
fn system_page_size() -> Option<usize> {
    // The C layout of Windows' SYSTEM_INFO, including fields we do not use.
    #[repr(C)]
    struct SystemInfo {
        oem_id: u32,
        page_size: u32,
        minimum_address: *mut std::ffi::c_void,
        maximum_address: *mut std::ffi::c_void,
        active_processor_mask: usize,
        processor_count: u32,
        processor_type: u32,
        allocation_granularity: u32,
        processor_level: u16,
        processor_revision: u16,
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetSystemInfo(info: *mut SystemInfo);
    }
    let mut info = std::mem::MaybeUninit::<SystemInfo>::uninit();
    // SAFETY: the pointer refers to correctly sized and aligned writable storage.
    // GetSystemInfo fills the structure before we read its page_size field.
    let bytes = unsafe {
        GetSystemInfo(info.as_mut_ptr());
        info.assume_init().page_size
    };
    usize::try_from(bytes).ok().filter(|&size| size > 0)
}

#[cfg(not(any(unix, windows)))]
fn system_page_size() -> Option<usize> {
    None
}

/// One byte offset per base page intersecting the buffer, including partial
/// first and last pages: a Vec's first byte need not be page-aligned.
fn page_offsets(address: usize, length: usize, page_size: usize) -> impl Iterator<Item = usize> {
    assert!(page_size > 0);
    let next_page = page_size - address % page_size;
    std::iter::once(0)
        .filter(move |_| length > 0)
        .chain((next_page..length).step_by(page_size))
}

fn main() {
    if let Some(task) = std::env::args().nth(1) {
        return child(&task);
    }

    println!("1. Compare addresses across runs (address space layout randomisation)");
    for run in 1..=3 {
        println!("    run {run}: {}", run_child("addresses"));
    }

    println!("\n2. Invalid accesses in child processes (usual OS protection failures)");
    println!("    These tasks deliberately break Rust's rules; a crash is not guaranteed.");
    println!(
        "    reading address 16:            {}",
        run_child("read-address-16")
    );
    println!(
        "    writing to a string constant:  {}",
        run_child("write-to-a-constant")
    );

    println!("\n3. Compare asking for zeroed memory with writing to its pages");
    let Some(page_size) = system_page_size() else {
        println!("    skipped: cannot query the OS page size on this platform");
        return;
    };
    println!("    OS base page size: {page_size} bytes");
    let size = 1 << 30; // 1 GiB = 1,073,741,824 bytes; try 64 << 20 for 64 MiB.
    let start = Instant::now();
    let mut memory = vec![0u8; size];
    let allocated = start.elapsed();
    let start = Instant::now();
    let mut pages_written = 0usize;
    for offset in page_offsets(memory.as_ptr() as usize, memory.len(), page_size) {
        memory[offset] = 1; // may cause a page fault if this page is not ready yet
        pages_written += 1;
    }
    let touched = start.elapsed();
    std::hint::black_box(&memory);
    println!(
        "    asking for {} MiB of zeroed memory: {allocated:.2?}",
        size >> 20
    );
    println!("    then writing one byte per base page: {touched:.2?}");
    println!("    base pages written: {pages_written} (this is not a page-fault count)");
}

#[cfg(test)]
mod tests {
    use super::page_offsets;

    #[test]
    fn statics_live_at_a_fixed_place_within_one_run() {
        let a = &super::COUNTER as *const u64;
        let b = &super::COUNTER as *const u64;
        assert_eq!(a, b);
        assert_eq!(super::COUNTER, 7);
    }

    #[test]
    fn visits_aligned_and_partial_pages_once() {
        // Tiny 16-byte pages make the boundary arithmetic easy to see.
        assert_eq!(page_offsets(32, 32, 16).collect::<Vec<_>>(), [0, 16]);
        assert_eq!(page_offsets(37, 32, 16).collect::<Vec<_>>(), [0, 11, 27]);
        assert_eq!(page_offsets(47, 2, 16).collect::<Vec<_>>(), [0, 1]);
        assert_eq!(page_offsets(37, 11, 16).collect::<Vec<_>>(), [0]);
        assert_eq!(page_offsets(37, 0, 16).count(), 0);
    }

    #[test]
    fn visits_every_page_for_4_kib_and_16_kib_page_sizes() {
        for page_size in [4096, 16384] {
            let mut memory = vec![0u8; 3 * page_size + 17];
            let address = memory.as_ptr() as usize;
            let offsets: Vec<_> = page_offsets(address, memory.len(), page_size).collect();
            let first_page = address / page_size;
            let last_page = (address + memory.len() - 1) / page_size;
            let visited: Vec<_> = offsets
                .iter()
                .map(|&offset| (address + offset) / page_size)
                .collect();
            assert_eq!(visited, (first_page..=last_page).collect::<Vec<_>>());
            for offset in offsets {
                memory[offset] = 1;
            }
        }
    }
}
