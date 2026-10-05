// Lesson 5: your own smart pointer.
//
// Box, Rc and Arc aren't magic. Two traits make a type a "smart pointer":
//   Deref   lets `*x` and method calls reach the value inside
//   Drop    runs clean-up code when the value goes away
// This lesson builds a small smart pointer with both, then uses Drop for the
// pattern behind MutexGuard, File and every other self-cleaning type: RAII.

use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

// ---- 1. A smart pointer that counts how it's used ------------------------------------------

/// Owns a value on the heap, like a Box, and counts every read and write.
pub struct Tracked<T> {
    value: Box<T>,
    reads: Cell<u32>, // Cell: counting must work through `&self` (see ownership lesson 7)
    writes: u32,
}

impl<T> Tracked<T> {
    pub fn new(value: T) -> Self {
        Tracked {
            value: Box::new(value),
            reads: Cell::new(0),
            writes: 0,
        }
    }

    pub fn stats(&self) -> (u32, u32) {
        (self.reads.get(), self.writes)
    }
}

/// `*tracked` and `tracked.method()` reach the value inside.
impl<T> Deref for Tracked<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.reads.set(self.reads.get().saturating_add(1));
        &self.value
    }
}

/// `*tracked = …` and `tracked.push(…)` change the value inside.
impl<T> DerefMut for Tracked<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.writes = self.writes.saturating_add(1);
        &mut self.value
    }
}

// ---- 2. Drop: code that runs when a value goes away -----------------------------------------

/// Records its name when dropped, so we can watch the order.
struct Noisy<'a> {
    name: &'static str,
    log: &'a std::cell::RefCell<Vec<&'static str>>,
}

impl Drop for Noisy<'_> {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.name);
    }
}

// ---- 3. RAII: a resource that cleans itself up ----------------------------------------------

/// A temporary file that is deleted when this value goes away: at the end of
/// the scope, on an early `return`, on `?`, or during a panic.
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn create(contents: &str) -> std::io::Result<Self> {
        // A new name for every file: this process's id plus a counter.
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let name = format!(
            "rust-examples-temp-{}-{}.txt",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        // `create_new` fails if the file already exists, so we never take over
        // (and later delete) a file that belongs to someone else.
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)?;
        file.write_all(contents.as_bytes())?;
        Ok(TempFile { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        // `drop` can't return an error, so a failed removal is ignored.
        let _ = fs::remove_file(&self.path);
    }
}

fn count_words_via_temp_file(text: &str) -> std::io::Result<usize> {
    let file = TempFile::create(text)?;
    let contents = fs::read_to_string(file.path())?;
    if contents.is_empty() {
        return Ok(0); // early return: `file` is dropped here, and deleted
    }
    Ok(contents.split_whitespace().count())
} // normal end: `file` is dropped here, and deleted

fn shout(text: &str) -> String {
    text.to_uppercase()
}

fn main() {
    println!("1. Deref: a Tracked<String> works like a String");
    let mut name = Tracked::new(String::from("ferris"));
    println!("    length: {}", name.len()); // a String method, through Deref
    name.push_str(" the crab"); // through DerefMut
    println!("    shouted: {}", shout(&name)); // &Tracked<String> → &String → &str: deref coercion
    let (reads, writes) = name.stats();
    println!("    used: {reads} reads, {writes} write");

    println!("\n2. Drop runs in reverse order of creation");
    let log = std::cell::RefCell::new(Vec::new());
    {
        let _first = Noisy {
            name: "first",
            log: &log,
        };
        let _second = Noisy {
            name: "second",
            log: &log,
        };
        let early = Noisy {
            name: "early",
            log: &log,
        };
        drop(early); // `drop(x)` ends a value early; `x.drop()` isn't allowed
        log.borrow_mut().push("— end of scope —");
    }
    println!("    {:?}", log.borrow());

    println!("\n3. RAII: a temporary file deletes itself");
    match count_words_via_temp_file("one two three") {
        Ok(words) => println!(
            "    {words} words; the owned temporary file was removed when its guard dropped"
        ),
        Err(error) => println!("    error: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deref_reaches_the_value_and_counts() {
        let mut numbers = Tracked::new(vec![1, 2]);
        assert_eq!(numbers.len(), 2); // read
        numbers.push(3); // write
        assert_eq!(*numbers, [1, 2, 3]); // read
        assert_eq!(numbers.stats(), (2, 1));
    }

    #[test]
    fn deref_coercion_turns_a_tracked_string_into_a_str() {
        let name = Tracked::new(String::from("abc"));
        assert_eq!(shout(&name), "ABC");
    }

    #[test]
    fn drop_order_is_reverse_creation_order() {
        let log = std::cell::RefCell::new(Vec::new());
        {
            let _a = Noisy {
                name: "a",
                log: &log,
            };
            let _b = Noisy {
                name: "b",
                log: &log,
            };
        }
        assert_eq!(*log.borrow(), ["b", "a"]);
    }

    #[test]
    fn temporary_files_are_distinct_and_removed_on_drop() {
        let first = TempFile::create("one").unwrap();
        let second = TempFile::create("two").unwrap();
        let path = first.path().to_owned();
        assert_ne!(path, second.path());
        drop(first);
        assert!(!path.exists());
        assert_eq!(fs::read_to_string(second.path()).unwrap(), "two");
        assert_eq!(count_words_via_temp_file("a b c").unwrap(), 3);
        assert_eq!(count_words_via_temp_file("").unwrap(), 0);
    }

    #[test]
    fn the_temp_file_is_gone_even_after_a_panic() {
        let mut path = None;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let file = TempFile::create("x").unwrap();
            path = Some(file.path().to_owned());
            panic!("something went wrong");
        }));
        assert!(result.is_err());
        assert!(!path.unwrap().exists());
    }
}
