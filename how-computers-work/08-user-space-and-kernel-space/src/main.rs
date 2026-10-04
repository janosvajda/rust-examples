// Lesson 8: user space and kernel space.
//
// Your program runs in USER mode: it may compute, and use its own memory,
// but it may not touch the disk, the network, the screen or other programs.
// For all of that, it asks the KERNEL, the core of the operating system,
// through a SYSTEM CALL. A system call switches the processor into kernel
// mode and back, and that switch has a cost. This program measures it.
//
// The `hello-syscall` folder next to this one makes system calls by hand, in
// assembly. This program uses the standard library, which makes them for you.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::time::{Duration, Instant};

const BYTES: usize = 1_000_000;

/// Writes one byte at a time, straight to the file: every `write_all` of one
/// byte is one `write` system call.
fn write_unbuffered(file: &mut File) -> std::io::Result<Duration> {
    let start = Instant::now();
    for _ in 0..BYTES {
        file.write_all(b"x")?;
    }
    Ok(start.elapsed())
}

/// The same bytes through a BufWriter: they're collected in an 8 KB buffer
/// in the program's own memory, and handed to the kernel 8 KB at a time.
fn write_buffered(file: File) -> std::io::Result<Duration> {
    let start = Instant::now();
    let mut writer = BufWriter::new(file);
    for _ in 0..BYTES {
        writer.write_all(b"x")?;
    }
    writer.flush()?;
    Ok(start.elapsed())
}

/// The same `write_all` calls into a Vec: no system calls at all, only work in user space.
fn write_to_memory() -> std::io::Result<Duration> {
    let start = Instant::now();
    let mut memory: Vec<u8> = Vec::with_capacity(BYTES);
    for _ in 0..BYTES {
        memory.write_all(b"x")?;
    }
    std::hint::black_box(&memory);
    Ok(start.elapsed())
}

fn main() -> std::io::Result<()> {
    let dir = std::env::temp_dir();
    let (path_a, path_b) = (dir.join("rust-examples-unbuffered.txt"), dir.join("rust-examples-buffered.txt"));

    println!("Writing {BYTES} bytes, one byte at a time:");
    let unbuffered = write_unbuffered(&mut File::create(&path_a)?)?;
    let buffered = write_buffered(File::create(&path_b)?)?;
    let in_memory = write_to_memory()?;
    let calls_buffered = BYTES.div_ceil(8 * 1024);

    println!("    straight to the kernel: {:>9.2?}   {BYTES} system calls", unbuffered);
    println!("    through a BufWriter:    {:>9.2?}   about {calls_buffered} system calls", buffered);
    println!("    into a Vec in memory:   {:>9.2?}   no system calls", in_memory);
    println!(
        "    one `write` system call to a file took about {} ns on this machine",
        (unbuffered.saturating_sub(buffered)).as_nanos() / BYTES as u128
    );

    assert_eq!(std::fs::metadata(&path_a)?.len(), BYTES as u64);
    assert_eq!(std::fs::metadata(&path_b)?.len(), BYTES as u64);
    std::fs::remove_file(&path_a)?;
    std::fs::remove_file(&path_b)?;

    println!("\nEvery one of these asks the kernel:");
    println!("    this process's id:    {}  (getpid)", std::process::id());
    println!("    files in the temp dir: {}  (open, then reading directory entries, then close)", std::fs::read_dir(&dir)?.count());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ways_write_the_same_bytes() {
        let dir = std::env::temp_dir();
        let (a, b) = (dir.join("rust-examples-test-a.txt"), dir.join("rust-examples-test-b.txt"));
        write_unbuffered(&mut File::create(&a).unwrap()).unwrap();
        write_buffered(File::create(&b).unwrap()).unwrap();
        assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
        std::fs::remove_file(a).unwrap();
        std::fs::remove_file(b).unwrap();
    }
}
