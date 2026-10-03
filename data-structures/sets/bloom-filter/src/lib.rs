//! # Bloom filter
//!
//! A **Bloom filter** is a set that uses very little memory, in exchange for
//! being *occasionally wrong in one direction*:
//!
//! - "**Definitely not** in the set": always correct.
//! - "**Probably** in the set": usually correct, but sometimes a *false positive*.
//!
//! It never stores the items themselves, only a few bits per item. A filter
//! for a million email addresses with a 1% false positive rate takes about
//! 1.2 MB. A `HashSet` of the same addresses would take tens of MB.
//!
//! ## Where it's used
//!
//! A Bloom filter sits in front of something slow, to skip work that is
//! certainly unnecessary:
//!
//! - Databases (Cassandra, RocksDB) check it before reading a file from disk:
//!   "this key is definitely not in that file, don't open it".
//! - Browsers have used it to check URLs against a list of malicious sites.
//! - "Have I already crawled this page?" in web crawlers.
//!
//! ## How it works
//!
//! The filter is an array of `m` bits, all starting at 0, and `k` different
//! hash functions.
//!
//! - **Insert**: hash the item `k` times and set those `k` bits to 1.
//! - **Check**: hash the item the same `k` ways. If **any** of those bits is 0,
//!   the item was never inserted. If **all** are 1, it probably was.
//!
//! ```text
//!   m = 16 bits, k = 3 hashes
//!
//!   insert "cat"  → bits 2, 7, 13
//!   insert "dog"  → bits 4, 7, 11
//!
//!   bits:  0 0 1 0 1 0 0 1 0 0 0 1 0 1 0 0
//!              ▲   ▲     ▲       ▲   ▲
//!
//!   check "cat"   → 2, 7, 13 all set       → probably present ✔
//!   check "fish"  → 1, 7, 13: bit 1 is 0   → definitely absent ✔
//!   check "owl"   → 4, 11, 13 all set      → probably present ✘ false positive!
//!                   (set by dog, dog and cat)
//! ```
//!
//! Items can't be removed: clearing a bit could erase part of another item.
//!
//! ## Choosing the size
//!
//! For `n` expected items and a target false positive rate `p`, the best
//! choices are:
//!
//! ```text
//!   m = −n · ln(p) / (ln 2)²     bits
//!   k = (m / n) · ln 2           hash functions
//! ```
//!
//! [`BloomFilter::new`] computes these for you. Roughly: about 10 bits per
//! item gives 1%, and each extra 5 bits per item cuts the rate by about 10×.
//!
//! ## Getting `k` hash functions from one
//!
//! Writing `k` different hash functions would be tedious. Instead we compute
//! two hashes `h1` and `h2` and combine them as `h1 + i·h2` for `i = 0..k`.
//! This *double hashing* trick (Kirsch and Mitzenmacher, 2006) works as well
//! as `k` independent hash functions.
//!
//! ## Complexity
//!
//! `insert` and `might_contain` are **O(k)**, independent of how many items
//! are in the filter. Memory is `m` bits, whatever the size of the items.
//!
//! ## Example
//!
//! ```
//! use bloom_filter::BloomFilter;
//!
//! let mut seen = BloomFilter::new(1000, 0.01);
//! seen.insert("https://www.rust-lang.org");
//!
//! assert!(seen.might_contain("https://www.rust-lang.org")); // never a false negative
//! assert!(!seen.might_contain("https://example.com"));      // almost certainly
//! ```

use std::hash::{DefaultHasher, Hash, Hasher};

/// A Bloom filter over any hashable type.
#[derive(Debug, Clone)]
pub struct BloomFilter {
    // The bit array, packed 64 bits per u64 to save memory.
    bits: Vec<u64>,
    bit_count: usize,
    hash_count: usize,
    items_added: usize,
}

impl BloomFilter {
    /// Creates a filter sized for `expected_items` with a false positive rate
    /// of about `false_positive_rate` (for example `0.01` for 1%).
    ///
    /// # Panics
    /// Panics if `expected_items` is 0 or the rate is not between 0 and 1.
    pub fn new(expected_items: usize, false_positive_rate: f64) -> Self {
        assert!(expected_items > 0, "expected_items must be positive");
        assert!(
            false_positive_rate > 0.0 && false_positive_rate < 1.0,
            "false_positive_rate must be between 0 and 1"
        );

        let n = expected_items as f64;
        let ln2 = std::f64::consts::LN_2;
        let bit_count = (-n * false_positive_rate.ln() / (ln2 * ln2)).ceil() as usize;
        let hash_count = ((bit_count as f64 / n) * ln2).round().max(1.0) as usize;

        Self::with_size(bit_count, hash_count)
    }

    /// Creates a filter with exactly `bit_count` bits and `hash_count` hash functions.
    pub fn with_size(bit_count: usize, hash_count: usize) -> Self {
        assert!(bit_count > 0 && hash_count > 0, "sizes must be positive");
        BloomFilter {
            // Round up to whole u64 words.
            bits: vec![0; bit_count.div_ceil(64)],
            bit_count,
            hash_count,
            items_added: 0,
        }
    }

    /// Adds `item` to the filter.
    ///
    /// `?Sized` lets you pass unsized types like `str` directly, as in
    /// `filter.insert("text")`.
    pub fn insert<T: Hash + ?Sized>(&mut self, item: &T) {
        for index in self.bit_indexes(item) {
            self.bits[index / 64] |= 1 << (index % 64);
        }
        self.items_added += 1;
    }

    /// Returns `false` if `item` was **definitely never** inserted, and `true`
    /// if it **probably** was.
    pub fn might_contain<T: Hash + ?Sized>(&self, item: &T) -> bool {
        self.bit_indexes(item)
            .all(|index| self.bits[index / 64] & (1 << (index % 64)) != 0)
    }

    /// Returns the number of bits in the filter.
    pub fn bit_count(&self) -> usize {
        self.bit_count
    }

    /// Returns the number of hash functions used per item.
    pub fn hash_count(&self) -> usize {
        self.hash_count
    }

    /// Returns how many times `insert` was called. (The filter can't tell
    /// whether the same item was inserted twice.)
    pub fn items_added(&self) -> usize {
        self.items_added
    }

    /// Estimates the current false positive rate from how full the filter is:
    /// `(1 − e^(−k·n/m))^k`. It rises as more items are added.
    pub fn estimated_false_positive_rate(&self) -> f64 {
        let k = self.hash_count as f64;
        let n = self.items_added as f64;
        let m = self.bit_count as f64;
        (1.0 - (-k * n / m).exp()).powf(k)
    }

    /// Returns the `k` bit positions for `item`, using double hashing.
    ///
    /// About `+ use<T>`: since Rust 2024, a returned `impl Trait` is assumed to
    /// borrow every reference parameter, including `&self`. That would stop
    /// `insert` from changing `self.bits` while it loops over these indexes.
    /// This iterator only holds a few numbers, so `use<T>` says it borrows nothing.
    fn bit_indexes<T: Hash + ?Sized>(&self, item: &T) -> impl Iterator<Item = usize> + use<T> {
        let h1 = hash_with_seed(item, 0);
        // `| 1` makes h2 odd, so the k positions don't all collapse onto the
        // same bit when h2 happens to be a multiple of the bit count.
        let h2 = hash_with_seed(item, 1) | 1;
        let bit_count = self.bit_count as u64;

        // `wrapping_*` lets the arithmetic overflow silently instead of
        // panicking in debug builds. Overflow is fine for hashing.
        (0..self.hash_count as u64)
            .map(move |i| (h1.wrapping_add(i.wrapping_mul(h2)) % bit_count) as usize)
    }
}

/// Hashes `item` together with `seed`, so different seeds give unrelated hashes.
///
/// `DefaultHasher::new()` always uses the same keys, so results are
/// repeatable within a program. (`std::collections::HashMap` uses random keys
/// instead.)
fn hash_with_seed<T: Hash + ?Sized>(item: &T, seed: u64) -> u64 {
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    item.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_filter_contains_nothing() {
        let filter = BloomFilter::new(100, 0.01);
        assert!(!filter.might_contain("anything"));
        assert_eq!(filter.estimated_false_positive_rate(), 0.0);
    }

    #[test]
    fn sizing_matches_the_formulas() {
        // 1000 items at 1%: about 9.6 bits per item and 7 hash functions.
        let filter = BloomFilter::new(1000, 0.01);
        assert_eq!(filter.bit_count(), 9586);
        assert_eq!(filter.hash_count(), 7);
    }

    #[test]
    fn never_gives_false_negatives() {
        let mut filter = BloomFilter::new(10_000, 0.01);
        for i in 0..10_000 {
            filter.insert(&i);
        }
        for i in 0..10_000 {
            assert!(filter.might_contain(&i), "{i} was inserted but not found");
        }
    }

    #[test]
    fn false_positive_rate_is_close_to_the_target() {
        let mut filter = BloomFilter::new(10_000, 0.01);
        for i in 0..10_000 {
            filter.insert(&i);
        }
        // Check 100,000 values that were never inserted.
        let false_positives = (10_000..110_000).filter(|i| filter.might_contain(i)).count();
        let rate = false_positives as f64 / 100_000.0;
        assert!(rate < 0.02, "false positive rate {rate} is far above 1%");
    }

    #[test]
    fn works_with_strings_and_str() {
        let mut filter = BloomFilter::new(10, 0.01);
        filter.insert("ferris");
        assert!(filter.might_contain(&String::from("ferris")));
    }
}
