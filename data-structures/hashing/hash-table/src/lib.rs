//! # Hash table (separate chaining)
//!
//! A **hash table** maps keys to values and finds any key in **O(1)** time on
//! average, no matter how many entries it holds. Rust's `HashMap` and Python's
//! `dict` are hash tables.
//!
//! ## The idea
//!
//! 1. Keep an array of **buckets**.
//! 2. Turn the key into a number with a **hash function**: `hash("apple") = 8452…`.
//! 3. Use `hash % number_of_buckets` as the bucket index.
//! 4. Store the entry in that bucket.
//!
//! To look a key up, hash it again and go straight to the same bucket. No
//! searching through the other buckets.
//!
//! ## Collisions
//!
//! Two different keys can land in the same bucket. This example uses
//! **separate chaining**: each bucket is a small list of `(key, value)` pairs,
//! and we search that list when needed.
//!
//! ```text
//!   buckets
//!   ┌───┐
//!   │ 0 │ ─► ("pear", 3)
//!   │ 1 │
//!   │ 2 │ ─► ("apple", 5) ─► ("kiwi", 1)    ◄── collision: both hashed to 2
//!   │ 3 │ ─► ("plum", 7)
//!   └───┘
//! ```
//!
//! (The other common approach is *open addressing*: on a collision, try the
//! next free slot in the array. Rust's `HashMap` uses an advanced form of it
//! called SwissTable.)
//!
//! ## Load factor and resizing
//!
//! The **load factor** is `entries / buckets`. As it grows, the chains get
//! longer and lookups slow down. When it passes [`MAX_LOAD_FACTOR`], we double
//! the number of buckets and **rehash** every entry. Their indexes change,
//! because `hash % buckets` changes. Resizing is O(n), but it happens rarely
//! enough that inserts are still O(1) on average (*amortized*).
//!
//! ## Complexity
//!
//! | Operation | Average        | Worst case (every key in one bucket) |
//! |-----------|----------------|--------------------------------------|
//! | `insert`  | O(1) amortized | O(n)                                 |
//! | `get`     | O(1)           | O(n)                                 |
//! | `remove`  | O(1)           | O(n)                                 |
//!
//! The worst case only happens with a bad hash function, or when an attacker
//! picks keys on purpose. Rust's default hasher (SipHash) uses a random key
//! per program run so attackers can't predict which keys collide.
//!
//! ## Example
//!
//! ```
//! use hash_table::HashTable;
//!
//! let mut prices = HashTable::new();
//! prices.insert("apple", 5);
//! prices.insert("pear", 3);
//!
//! assert_eq!(prices.get(&"apple"), Some(&5));
//! assert_eq!(prices.insert("apple", 6), Some(5)); // returns the old value
//! assert_eq!(prices.remove(&"pear"), Some(3));
//! assert_eq!(prices.len(), 1);
//! ```

use std::hash::{BuildHasher, Hash, RandomState};

/// How many buckets to create on the first insert.
const INITIAL_BUCKETS: usize = 8;

/// Resize once there are more than 0.75 entries per bucket on average.
/// This is a common choice (Java's `HashMap` uses it too): low enough to keep
/// chains short, high enough not to waste too much memory on empty buckets.
pub const MAX_LOAD_FACTOR: f64 = 0.75;

/// A hash map from `K` to `V`, using separate chaining.
///
/// Keys must implement `Hash` (so we can compute a bucket) and `Eq` (so we can
/// tell two keys in the same bucket apart).
#[derive(Debug)]
pub struct HashTable<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    len: usize,
    // Builds the hasher. RandomState picks random keys per table, which is
    // what protects std's HashMap against collision attacks.
    hasher: RandomState,
}

impl<K: Hash + Eq, V> Default for HashTable<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: Hash + Eq, V> HashTable<K, V> {
    /// Creates an empty table. No buckets are allocated until the first insert.
    pub fn new() -> Self {
        HashTable {
            buckets: Vec::new(),
            len: 0,
            hasher: RandomState::new(),
        }
    }

    /// Inserts `key → value`. If the key was already present, the value is
    /// replaced and the old one is returned.
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // Check *before* inserting whether one more entry would overload the table.
        if self.buckets.is_empty()
            || (self.len + 1) as f64 / self.buckets.len() as f64 > MAX_LOAD_FACTOR
        {
            self.resize();
        }

        let index = self.bucket_index(&key);
        let bucket = &mut self.buckets[index];

        // If the key is already in the chain, replace its value.
        for (existing_key, existing_value) in bucket.iter_mut() {
            if *existing_key == key {
                // `mem::replace` puts the new value in and gives back the old one.
                return Some(std::mem::replace(existing_value, value));
            }
        }

        bucket.push((key, value));
        self.len += 1;
        None
    }

    /// Returns a reference to the value for `key`, if present.
    pub fn get(&self, key: &K) -> Option<&V> {
        if self.buckets.is_empty() {
            return None;
        }
        let bucket = &self.buckets[self.bucket_index(key)];
        bucket.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    /// Returns a mutable reference to the value for `key`, if present.
    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        if self.buckets.is_empty() {
            return None;
        }
        let index = self.bucket_index(key);
        self.buckets[index]
            .iter_mut()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    /// Returns `true` if the table contains `key`.
    pub fn contains_key(&self, key: &K) -> bool {
        self.get(key).is_some()
    }

    /// Removes `key` and returns its value, if it was present.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        if self.buckets.is_empty() {
            return None;
        }
        let index = self.bucket_index(key);
        let bucket = &mut self.buckets[index];
        let position = bucket.iter().position(|(k, _)| k == key)?;
        self.len -= 1;
        // The order inside a bucket doesn't matter, so `swap_remove` is fine:
        // it moves the last pair into the gap in O(1) instead of shifting.
        Some(bucket.swap_remove(position).1)
    }

    /// Returns an iterator over all `(key, value)` pairs, in no particular order.
    pub fn iter(&self) -> impl Iterator<Item = (&K, &V)> {
        self.buckets
            .iter()
            .flat_map(|bucket| bucket.iter().map(|(k, v)| (k, v)))
    }

    /// Returns the number of entries.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the table has no entries.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of buckets.
    pub fn bucket_count(&self) -> usize {
        self.buckets.len()
    }

    /// Returns the current load factor: entries per bucket.
    pub fn load_factor(&self) -> f64 {
        if self.buckets.is_empty() {
            0.0
        } else {
            self.len as f64 / self.buckets.len() as f64
        }
    }

    /// Returns the length of the longest chain. A good hash function keeps this small.
    pub fn longest_chain(&self) -> usize {
        self.buckets.iter().map(Vec::len).max().unwrap_or(0)
    }

    /// Hashes `key` and maps the hash to a bucket index.
    fn bucket_index(&self, key: &K) -> usize {
        let hash: u64 = self.hasher.hash_one(key);
        (hash % self.buckets.len() as u64) as usize
    }

    /// Doubles the number of buckets and moves every entry to its new bucket.
    fn resize(&mut self) {
        let new_count = (self.buckets.len() * 2).max(INITIAL_BUCKETS);
        let mut new_buckets: Vec<Vec<(K, V)>> = Vec::with_capacity(new_count);
        new_buckets.resize_with(new_count, Vec::new);

        // Swap in the empty buckets first, so `bucket_index` uses the new count.
        let old_buckets = std::mem::replace(&mut self.buckets, new_buckets);
        for (key, value) in old_buckets.into_iter().flatten() {
            let index = self.bucket_index(&key);
            self.buckets[index].push((key, value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_table() {
        let table: HashTable<&str, i32> = HashTable::new();
        assert!(table.is_empty());
        assert_eq!(table.get(&"missing"), None);
        assert_eq!(table.bucket_count(), 0);
        assert_eq!(table.load_factor(), 0.0);
    }

    #[test]
    fn insert_get_and_replace() {
        let mut table = HashTable::new();
        assert_eq!(table.insert("a", 1), None);
        assert_eq!(table.insert("b", 2), None);
        assert_eq!(table.insert("a", 10), Some(1));
        assert_eq!(table.len(), 2);
        assert_eq!(table.get(&"a"), Some(&10));
        assert_eq!(table.get(&"b"), Some(&2));
        assert!(!table.contains_key(&"c"));
    }

    #[test]
    fn get_mut_changes_the_value() {
        let mut table = HashTable::new();
        table.insert("count", 1);
        *table.get_mut(&"count").unwrap() += 1;
        assert_eq!(table.get(&"count"), Some(&2));
        assert_eq!(table.get_mut(&"missing"), None);
    }

    #[test]
    fn remove_entries() {
        let mut table = HashTable::new();
        table.insert(1, "one");
        table.insert(2, "two");
        assert_eq!(table.remove(&1), Some("one"));
        assert_eq!(table.remove(&1), None);
        assert_eq!(table.len(), 1);
        assert_eq!(table.get(&2), Some(&"two"));
    }

    #[test]
    fn grows_and_keeps_every_entry() {
        let mut table = HashTable::new();
        for i in 0..1000 {
            table.insert(i, i * i);
            assert!(table.load_factor() <= MAX_LOAD_FACTOR);
        }
        assert_eq!(table.len(), 1000);
        assert_eq!(table.bucket_count(), 2048);
        for i in 0..1000 {
            assert_eq!(table.get(&i), Some(&(i * i)));
        }
    }

    #[test]
    fn iter_visits_every_entry_once() {
        let mut table = HashTable::new();
        for i in 0..50 {
            table.insert(i, ());
        }
        let mut keys: Vec<_> = table.iter().map(|(k, _)| *k).collect();
        keys.sort();
        assert_eq!(keys, (0..50).collect::<Vec<_>>());
    }

    #[test]
    fn works_with_string_keys() {
        let mut table = HashTable::new();
        table.insert(String::from("rust"), 2015);
        assert_eq!(table.get(&String::from("rust")), Some(&2015));
    }
}
