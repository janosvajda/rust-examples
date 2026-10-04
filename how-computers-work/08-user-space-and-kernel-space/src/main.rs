// Lesson 8: user space and kernel space.
//
// Ordinary applications compute in USER mode and request protected services
// from the KERNEL through OS interfaces. A system call enters kernel mode.
// This program compares complete file-writing strategies; its timings include
// file work as well as the user/kernel transition, not just the mode switch.
//
// The `hello-syscall` folder next to this one makes system calls by hand, in
// assembly. This program uses the standard library, which makes them for you.

use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

const BYTES: usize = 1_000_000;

/// Each experiment owns a separate directory. Creating it is atomic: an
/// existing name is skipped, never reused or emptied.
struct TemporaryDirectory {
    path: PathBuf,
}

impl TemporaryDirectory {
    fn new(parent: &Path) -> std::io::Result<Self> {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        for _ in 0..1_000 {
            let number = NEXT.fetch_add(1, Ordering::Relaxed);
            let path = parent.join(format!("rust-examples-io-{}-{number}", std::process::id()));
            match std::fs::create_dir(&path) {
                Ok(()) => return Ok(Self { path }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => return Err(error),
            }
        }
        Err(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            "could not create a fresh experiment directory",
        ))
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        // Ordinary scope exit attempts cleanup, including when an I/O error
        // returns early. Drop cannot report cleanup errors to the caller.
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

/// Writes one byte at a time, straight to the file: each `write_all` of one
/// byte is normally one `write` system call.
fn write_unbuffered(file: &mut File) -> std::io::Result<Duration> {
    let start = Instant::now();
    for _ in 0..BYTES {
        file.write_all(b"x")?;
    }
    Ok(start.elapsed())
}

/// The same bytes through a BufWriter: they're collected in a buffer in the
/// program's own memory, and handed to the kernel a buffer at a time.
/// Returns the measured duration and the writer's actual buffer capacity.
fn write_buffered(file: File) -> std::io::Result<(Duration, usize)> {
    let start = Instant::now();
    let mut writer = BufWriter::new(file);
    let capacity = writer.capacity();
    for _ in 0..BYTES {
        writer.write_all(b"x")?;
    }
    writer.flush()?;
    Ok((start.elapsed(), capacity))
}

/// Appends to user-space memory, without explicit file writes. Allocation or
/// page faults can still need OS help.
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
    let experiment = TemporaryDirectory::new(&dir)?;
    let path_a = experiment.path.join("unbuffered.txt");
    let path_b = experiment.path.join("buffered.txt");

    println!("Writing {BYTES} bytes, one byte at a time:");
    let unbuffered = write_unbuffered(&mut File::create_new(&path_a)?)?;
    let (buffered, capacity) = write_buffered(File::create_new(&path_b)?)?;
    let in_memory = write_to_memory()?;
    // Estimates write requests, not a trace; partial writes or retries add work.
    // A zero-capacity writer would send each one-byte write straight through.
    let calls_buffered = BYTES.div_ceil(capacity.max(1));

    println!(
        "    straight to the kernel: {:>9.2?}   about {BYTES} write calls (estimate)",
        unbuffered
    );
    println!(
        "    through a BufWriter:    {:>9.2?}   about {calls_buffered} write calls (estimate)",
        buffered
    );
    println!("    actual buffer capacity: {capacity} bytes");
    println!(
        "    into a Vec in memory:   {:>9.2?}   no system calls for the writes",
        in_memory
    );
    println!(
        "    so each unbuffered write cost about {} ns extra on this machine (an estimate)",
        (unbuffered.saturating_sub(buffered)).as_nanos() / BYTES as u128
    );

    assert_eq!(std::fs::metadata(&path_a)?.len(), BYTES as u64);
    assert_eq!(std::fs::metadata(&path_b)?.len(), BYTES as u64);
    drop(experiment); // the files are closed; remove only this experiment's directory

    println!("\nThese ask the operating system for information:");
    println!(
        "    this process's id:    {}  (normally the getpid system call)",
        std::process::id()
    );
    let entries =
        std::fs::read_dir(&dir)?.try_fold(0usize, |count, entry| entry.map(|_| count + 1))?;
    println!(
        "    entries in the temp dir: {entries}  (includes subdirectories; entry errors are returned)"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_ways_write_the_same_bytes() {
        let experiment = TemporaryDirectory::new(&std::env::temp_dir()).unwrap();
        let (a, b) = (experiment.path.join("a.txt"), experiment.path.join("b.txt"));
        write_unbuffered(&mut File::create_new(&a).unwrap()).unwrap();
        write_buffered(File::create_new(&b).unwrap()).unwrap();
        assert_eq!(std::fs::read(&a).unwrap(), std::fs::read(&b).unwrap());
    }

    #[test]
    fn experiments_are_separate_and_preserve_existing_files() {
        let parent = TemporaryDirectory::new(&std::env::temp_dir()).unwrap();
        let existing = parent.path.join("rust-examples-unbuffered.txt");
        std::fs::write(&existing, b"keep this file").unwrap();
        let first = TemporaryDirectory::new(&parent.path).unwrap();
        let second = TemporaryDirectory::new(&parent.path).unwrap();
        assert_ne!(first.path, second.path);
        let first_path = first.path.clone();
        let second_path = second.path.clone();
        std::fs::write(first.path.join("data.txt"), b"Ada").unwrap();
        std::fs::write(second.path.join("data.txt"), b"Ben").unwrap();
        drop(first);
        assert!(!first_path.exists());
        assert_eq!(std::fs::read(second_path.join("data.txt")).unwrap(), b"Ben");
        drop(second);
        assert!(!second_path.exists());
        assert_eq!(std::fs::read(existing).unwrap(), b"keep this file");
    }
}
