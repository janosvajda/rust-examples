//! # LRU cache (hash map + doubly linked list in an arena)
//!
//! A **cache** keeps a limited number of recently used results so they don't
//! have to be fetched or computed again. When it's full and something new
//! arrives, something old has to go. An **LRU** (Least Recently Used) cache
//! evicts the entry that hasn't been used for the longest time.
//!
//! ```text
//!   capacity 3
//!
//!   put A, put B, put C     most recent ─► [C] [B] [A] ◄─ least recent
//!   get A                                   [A] [C] [B]      (A is used, moves to the front)
//!   put D                                   [D] [A] [C]      (full: B was least recent, evicted)
//! ```
//!
//! ## Two structures working together
//!
//! Both `get` and `put` must be **O(1)**, and no single structure can do it:
//!
//! - A **hash map** finds an entry by key in O(1), but has no notion of order.
//! - A **doubly linked list** keeps the usage order. It can move any node to
//!   the front, or drop the last node, in O(1), but finding a node by key is O(n).
//!
//! So we use both. The map goes from key to *the position of its node*, and
//! the list keeps the order:
//!
//! ```text
//!     HashMap                 entries: Vec<Entry>
//!   ┌─────┬───┐
//!   │ "A" │ 0 │ ──► [0] A   prev: None  next: 2      ◄─ head (most recent)
//!   │ "B" │ 1 │ ──► [1] B   prev: 2     next: None   ◄─ tail (least recent)
//!   │ "C" │ 2 │ ──► [2] C   prev: 0     next: 1
//!   └─────┴───┘
//!                   usage order: A ─► C ─► B
//! ```
//!
//! ## An arena instead of `Rc<RefCell<…>>`
//!
//! The `linear/doubly-linked-list` example links nodes with `Rc`, `RefCell` and
//! `Weak`. This one uses the other common Rust approach, an **arena**: every
//! node lives in one `Vec`, and the links are plain `usize` **indexes** into it.
//!
//! - No reference counting, no runtime borrow checks, no risk of `Rc` cycles.
//! - Nodes sit next to each other in memory, which is fast.
//! - The cost: you must keep the indexes correct yourself. A stale index is a
//!   logic bug, though never undefined behaviour, since `Vec` checks bounds.
//!
//! The arena never shrinks here: when the cache is full, the evicted entry's
//! slot is reused for the new one.
//!
//! ## Complexity
//!
//! | Operation | Time         |
//! |-----------|--------------|
//! | `get`     | O(1) average |
//! | `put`     | O(1) average |
//! | `peek`    | O(1) average |
//!
//! ("average" because of the hash map.)
//!
//! ## Example
//!
//! ```
//! use lru_cache::LruCache;
//!
//! let mut cache = LruCache::new(2);
//! cache.put("a", 1);
//! cache.put("b", 2);
//! cache.get(&"a");                // "a" is now the most recently used
//!
//! let evicted = cache.put("c", 3); // full: evicts the least recently used, "b"
//! assert_eq!(evicted, Some(("b", 2)));
//! assert_eq!(cache.get(&"b"), None);
//! assert_eq!(cache.get(&"a"), Some(&1));
//! ```

use std::collections::HashMap;
use std::hash::Hash;

/// One cache entry, which is also a node of the usage-order list.
#[derive(Debug)]
struct Entry<K, V> {
    key: K,
    value: V,
    prev: Option<usize>, // index of the more recently used neighbour
    next: Option<usize>, // index of the less recently used neighbour
}

/// A fixed-capacity cache that evicts the least recently used entry.
///
/// Keys are stored twice (in the map and in the entry, so an evicted entry can
/// be removed from the map), which is why they must be `Clone`.
#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>,
    entries: Vec<Entry<K, V>>,
    head: Option<usize>, // most recently used
    tail: Option<usize>, // least recently used
    capacity: usize,
}

impl<K: Hash + Eq + Clone, V> LruCache<K, V> {
    /// Creates a cache that holds at most `capacity` entries.
    ///
    /// # Panics
    /// Panics if `capacity` is 0.
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0, "capacity must be at least 1");
        LruCache {
            map: HashMap::with_capacity(capacity),
            entries: Vec::with_capacity(capacity),
            head: None,
            tail: None,
            capacity,
        }
    }

    /// Returns the value for `key` and marks it as the most recently used.
    ///
    /// This takes `&mut self` even though it's a read, because it changes
    /// the usage order.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let index = *self.map.get(key)?;
        self.move_to_front(index);
        Some(&self.entries[index].value)
    }

    /// Returns the value for `key` **without** changing the usage order.
    pub fn peek(&self, key: &K) -> Option<&V> {
        self.map.get(key).map(|&index| &self.entries[index].value)
    }

    /// Inserts or updates `key → value` and marks it as the most recently used.
    ///
    /// If the cache was full and `key` is new, the least recently used entry
    /// is evicted and returned.
    pub fn put(&mut self, key: K, value: V) -> Option<(K, V)> {
        // Existing key: update the value and move it to the front.
        if let Some(&index) = self.map.get(&key) {
            self.entries[index].value = value;
            self.move_to_front(index);
            return None;
        }

        let new_entry = Entry {
            key: key.clone(),
            value,
            prev: None,
            next: None,
        };

        // Not full yet: add a new slot to the arena.
        if self.entries.len() < self.capacity {
            let index = self.entries.len();
            self.entries.push(new_entry);
            self.map.insert(key, index);
            self.attach_front(index);
            return None;
        }

        // Full: reuse the least recently used entry's slot for the new entry.
        let index = self.tail.expect("a full cache has a tail");
        self.detach(index);
        let evicted = std::mem::replace(&mut self.entries[index], new_entry);
        self.map.remove(&evicted.key);
        self.map.insert(key, index);
        self.attach_front(index);
        Some((evicted.key, evicted.value))
    }

    /// Returns `true` if `key` is cached. Doesn't change the usage order.
    pub fn contains(&self, key: &K) -> bool {
        self.map.contains_key(key)
    }

    /// Returns the number of cached entries.
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns `true` if the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Returns the maximum number of entries.
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the keys from most to least recently used.
    pub fn keys_by_recency(&self) -> Vec<&K> {
        let mut keys = Vec::with_capacity(self.len());
        let mut current = self.head;
        while let Some(index) = current {
            keys.push(&self.entries[index].key);
            current = self.entries[index].next;
        }
        keys
    }

    /// Moves an entry that is already in the list to the front.
    fn move_to_front(&mut self, index: usize) {
        if self.head != Some(index) {
            self.detach(index);
            self.attach_front(index);
        }
    }

    /// Unlinks the entry at `index` from the list, joining its neighbours.
    ///
    /// ```text
    ///   before:  [P] ⇄ [X] ⇄ [N]
    ///   after:   [P] ⇄ [N]          (X is unlinked, but stays in the Vec)
    /// ```
    fn detach(&mut self, index: usize) {
        let (prev, next) = (self.entries[index].prev, self.entries[index].next);

        match prev {
            Some(p) => self.entries[p].next = next,
            None => self.head = next, // X was the head
        }
        match next {
            Some(n) => self.entries[n].prev = prev,
            None => self.tail = prev, // X was the tail
        }
    }

    /// Links the entry at `index` in as the new head (most recently used).
    fn attach_front(&mut self, index: usize) {
        self.entries[index].prev = None;
        self.entries[index].next = self.head;
        match self.head {
            Some(old_head) => self.entries[old_head].prev = Some(index),
            None => self.tail = Some(index), // the list was empty
        }
        self.head = Some(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_returns_cached_values() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);
        assert_eq!(cache.get(&"a"), Some(&1));
        assert_eq!(cache.get(&"b"), Some(&2));
        assert_eq!(cache.get(&"c"), None);
        assert_eq!(cache.len(), 2);
    }

    #[test]
    fn evicts_least_recently_used() {
        let mut cache = LruCache::new(3);
        cache.put("a", 1);
        cache.put("b", 2);
        cache.put("c", 3);
        cache.get(&"a");
        assert_eq!(cache.keys_by_recency(), vec![&"a", &"c", &"b"]);

        assert_eq!(cache.put("d", 4), Some(("b", 2)));
        assert_eq!(cache.keys_by_recency(), vec![&"d", &"a", &"c"]);
        assert!(!cache.contains(&"b"));
        assert_eq!(cache.len(), 3);
    }

    #[test]
    fn updating_a_key_does_not_evict() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);
        assert_eq!(cache.put("a", 10), None);
        assert_eq!(cache.keys_by_recency(), vec![&"a", &"b"]);
        assert_eq!(cache.peek(&"a"), Some(&10));
    }

    #[test]
    fn peek_does_not_change_order() {
        let mut cache = LruCache::new(2);
        cache.put("a", 1);
        cache.put("b", 2);
        assert_eq!(cache.peek(&"a"), Some(&1));
        // "a" is still least recently used, so it's evicted next.
        assert_eq!(cache.put("c", 3), Some(("a", 1)));
    }

    #[test]
    fn capacity_of_one() {
        let mut cache = LruCache::new(1);
        cache.put(1, "one");
        assert_eq!(cache.put(2, "two"), Some((1, "one")));
        assert_eq!(cache.get(&2), Some(&"two"));
        assert_eq!(cache.keys_by_recency(), vec![&2]);
    }

    #[test]
    fn matches_a_simple_reference_model() {
        // Compare against an obviously-correct (but O(n)) model: a Vec
        // ordered from most to least recently used.
        let mut cache = LruCache::new(4);
        let mut model: Vec<(u32, u32)> = Vec::new();

        for step in 0..500u32 {
            let key = (step * 7) % 11;
            if step % 3 == 0 {
                let expected = model.iter().position(|&(k, _)| k == key).map(|i| {
                    let entry = model.remove(i);
                    model.insert(0, entry);
                    entry.1
                });
                assert_eq!(cache.get(&key).copied(), expected);
            } else {
                model.retain(|&(k, _)| k != key);
                model.insert(0, (key, step));
                let expected_evicted = if model.len() > 4 { model.pop() } else { None };
                assert_eq!(cache.put(key, step), expected_evicted);
            }
            let model_keys: Vec<&u32> = model.iter().map(|(k, _)| k).collect();
            assert_eq!(cache.keys_by_recency(), model_keys);
        }
    }

    #[test]
    #[should_panic(expected = "capacity must be at least 1")]
    fn zero_capacity_panics() {
        let _cache: LruCache<i32, i32> = LruCache::new(0);
    }
}
