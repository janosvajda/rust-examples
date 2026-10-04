// Lesson 9: why Rust.
//
// C gave programmers direct control of memory, and with it, a whole family of
// bugs: reading past the end of a buffer, using memory after freeing it,
// pointers to things that no longer exist, two threads writing at once. Rust
// keeps the control and the speed, and rejects these bugs, mostly before the
// program even runs. This program demonstrates both halves of that claim.

use std::hint::black_box;
use std::time::Instant;

// ---- 1. A Heartbleed-style bug ----------------------------------------------------------------
//
// A "heartbeat": the client sends some bytes and says how long they are; the
// server sends the same bytes back. In 2014, OpenSSL (written in C) trusted
// the CLAIMED length:
//
//     memcpy(response, request_payload, claimed_length);   // C: no check
//
// A client could send 1 byte and claim 64,000: the server then copied 64 KB
// of whatever memory followed, which could contain passwords and private
// keys, and sent it back. The same mistake in Rust:

/// Echoes `claimed_length` bytes of the payload back, or refuses.
fn heartbeat(payload: &[u8], claimed_length: usize) -> Result<&[u8], String> {
    // `get` checks the range: asking for more than exists gives None, not
    // whatever lies beyond. (`&payload[..claimed_length]` would stop the
    // program instead of reading other memory.)
    payload.get(..claimed_length).ok_or(format!(
        "refused: claimed {claimed_length} bytes, but only {} were sent",
        payload.len()
    ))
}

// ---- 2. Safety that the compiler checks ---------------------------------------------------------
//
// These classic C bugs don't compile in Rust (the real errors, Rust 1.99):
//
// Use after free:
//     let names = vec![String::from("Ada")];
//     drop(names);                         // the memory is freed here
//     println!("{}", names.len());
//     error[E0382]: borrow of moved value: `names`
//
// Changing a list while reading it (in C++, the push may move the list in
// memory, leaving the loop reading freed memory):
//     for n in &numbers { numbers.push(*n); }
//     error[E0502]: cannot borrow `numbers` as mutable because it is also borrowed as immutable
//
// Returning a pointer to something that's about to disappear:
//     fn get() -> &'static str { let text = String::from("hi"); &text }
//     error[E0515]: cannot return reference to local variable `text`

// ---- 3. ...at C speed ---------------------------------------------------------------------------

/// The idiomatic way: an iterator. No index, so nothing to check.
fn sum_iterator(numbers: &[u64]) -> u64 {
    numbers.iter().sum()
}

/// C-style indexing: every `numbers[i]` is checked, unless the compiler can
/// prove `i < len`, which it can here, so the checks disappear.
#[allow(clippy::needless_range_loop)] // written this way on purpose, to compare
fn sum_indexed(numbers: &[u64]) -> u64 {
    let mut total = 0;
    for i in 0..numbers.len() {
        total += numbers[i];
    }
    total
}

/// No checks at all, like C: `unsafe` promises the compiler every index is valid.
fn sum_unchecked(numbers: &[u64]) -> u64 {
    let mut total = 0;
    for i in 0..numbers.len() {
        // SAFETY: `i < numbers.len()`, from the loop's range.
        total += unsafe { *numbers.get_unchecked(i) };
    }
    total
}

fn best_of_five(f: impl Fn() -> u64) -> (u64, f64) {
    let mut best = f64::MAX;
    let mut result = 0;
    for _ in 0..5 {
        let start = Instant::now();
        result = black_box(f());
        best = best.min(start.elapsed().as_secs_f64() * 1000.0);
    }
    (result, best)
}

fn main() {
    println!("1. A heartbeat, in Rust");
    let payload = b"hi";
    println!("    honest client (2 bytes, claims 2):    {:?}", heartbeat(payload, 2).map(String::from_utf8_lossy));
    println!("    attacker (2 bytes, claims 64000):     {:?}", heartbeat(payload, 64_000));

    println!("\n2. The classic C memory bugs don't compile: see the comments in the code");

    println!("\n3. Adding up 50 million numbers, three ways (best of 5 runs)");
    let numbers: Vec<u64> = (0..50_000_000).collect();
    let numbers = black_box(numbers);
    let (a, iterator) = best_of_five(|| sum_iterator(&numbers));
    let (b, indexed) = best_of_five(|| sum_indexed(&numbers));
    let (c, unchecked) = best_of_five(|| sum_unchecked(&numbers));
    assert!(a == b && b == c);
    println!("    iterator (safe, idiomatic):      {iterator:6.2} ms");
    println!("    indexing (safe, checked):        {indexed:6.2} ms");
    println!("    get_unchecked (unsafe, like C):  {unchecked:6.2} ms");
    println!("    the same answer every time: {a}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_honest_heartbeat_is_echoed() {
        assert_eq!(heartbeat(b"hello", 5), Ok(&b"hello"[..]));
        assert_eq!(heartbeat(b"hello", 2), Ok(&b"he"[..]));
    }

    #[test]
    fn a_lying_length_is_refused_not_leaked() {
        assert!(heartbeat(b"hi", 64_000).is_err());
        assert!(heartbeat(b"", 1).is_err());
    }

    #[test]
    #[should_panic(expected = "range end index 64000 out of range for slice of length 2")]
    fn plain_slicing_stops_the_program_instead_of_reading_other_memory() {
        let payload: &[u8] = black_box(b"hi");
        let _ = &payload[..black_box(64_000)];
    }

    #[test]
    fn all_three_sums_agree() {
        let numbers: Vec<u64> = (0..1000).collect();
        assert_eq!(sum_iterator(&numbers), 499_500);
        assert_eq!(sum_indexed(&numbers), 499_500);
        assert_eq!(sum_unchecked(&numbers), 499_500);
    }
}
