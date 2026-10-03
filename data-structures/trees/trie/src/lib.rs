//! # Trie (prefix tree)
//!
//! A **trie** stores strings by sharing their common prefixes. Each edge is
//! one character, and a path from the root spells out a word. A node is marked
//! (`*` below) when a word ends there.
//!
//! ```text
//!   words: car, cart, cat, dog
//!
//!          (root)
//!          /    \
//!         c      d
//!         |      |
//!         a      o
//!        / \     |
//!       r*  t*   g*
//!       |
//!       t*
//! ```
//!
//! "car" and "cart" share the path `c → a → r`. "car" ends at the `r` node,
//! which is why that node is marked even though it has a child.
//!
//! ## Why use a trie?
//!
//! Looking up a word takes **O(k)** steps, where k is the word's length. That
//! doesn't depend on how many words are stored. A trie also answers questions
//! that a hash map can't answer efficiently:
//!
//! - "Which words start with `ca`?" (autocomplete)
//! - "Is `ca` the beginning of any word?" (word games, input validation)
//!
//! The trade-off is memory: every node has its own map of children.
//!
//! ## Complexity (k = length of the word or prefix)
//!
//! | Operation          | Time                               |
//! |--------------------|------------------------------------|
//! | `insert`           | O(k)                               |
//! | `contains`         | O(k)                               |
//! | `starts_with`      | O(k)                               |
//! | `words_with_prefix`| O(k + size of the matching subtree) |
//!
//! ## Example
//!
//! ```
//! use trie::Trie;
//!
//! let mut trie = Trie::new();
//! for word in ["car", "cart", "cat", "dog"] {
//!     trie.insert(word);
//! }
//!
//! assert!(trie.contains("car"));
//! assert!(!trie.contains("ca"));      // a prefix, not a stored word
//! assert!(trie.starts_with("ca"));
//! assert_eq!(trie.words_with_prefix("car"), vec!["car", "cart"]);
//! ```

use std::collections::BTreeMap;

/// One node in the trie.
#[derive(Debug, Default)]
struct TrieNode {
    // A BTreeMap keeps the children sorted by character, so
    // `words_with_prefix` returns words in alphabetical order.
    // A HashMap would be a little faster but the order would be random.
    children: BTreeMap<char, TrieNode>,
    // True if a word ends at this node.
    is_word_end: bool,
}

/// A set of strings stored as a prefix tree.
#[derive(Debug, Default)]
pub struct Trie {
    root: TrieNode,
    len: usize,
}

impl Trie {
    /// Creates an empty trie.
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts `word`. Returns `false` if it was already present.
    ///
    /// We follow the characters from the root, creating any nodes that don't
    /// exist yet, and mark the last node as the end of a word.
    pub fn insert(&mut self, word: &str) -> bool {
        let mut node = &mut self.root;
        for c in word.chars() {
            // `entry(..).or_default()` returns the existing child, or inserts
            // an empty node and returns that.
            node = node.children.entry(c).or_default();
        }

        if node.is_word_end {
            return false;
        }
        node.is_word_end = true;
        self.len += 1;
        true
    }

    /// Returns `true` if `word` was inserted (not just a prefix of a word).
    pub fn contains(&self, word: &str) -> bool {
        self.find_node(word).is_some_and(|node| node.is_word_end)
    }

    /// Returns `true` if any stored word starts with `prefix`.
    pub fn starts_with(&self, prefix: &str) -> bool {
        self.find_node(prefix).is_some()
    }

    /// Returns every stored word that starts with `prefix`, in alphabetical order.
    ///
    /// This is autocomplete: walk down to the node for `prefix`, then collect
    /// every word in the subtree below it.
    pub fn words_with_prefix(&self, prefix: &str) -> Vec<String> {
        let mut words = Vec::new();
        if let Some(node) = self.find_node(prefix) {
            let mut current_word = prefix.to_string();
            collect_words(node, &mut current_word, &mut words);
        }
        words
    }

    /// Returns the number of words stored.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if no words are stored.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Follows `text` character by character from the root.
    /// Returns the node where it ends, or `None` if the path doesn't exist.
    fn find_node(&self, text: &str) -> Option<&TrieNode> {
        let mut node = &self.root;
        for c in text.chars() {
            // `?` returns None early if there's no child for this character.
            node = node.children.get(&c)?;
        }
        Some(node)
    }
}

/// Depth-first walk that collects every word in the subtree under `node`.
///
/// `current_word` holds the characters on the path so far. We push a
/// character before visiting a child and pop it afterwards, so one `String` is
/// reused for the whole walk instead of allocating a new one per node.
fn collect_words(node: &TrieNode, current_word: &mut String, words: &mut Vec<String>) {
    if node.is_word_end {
        words.push(current_word.clone());
    }
    for (&c, child) in &node.children {
        current_word.push(c);
        collect_words(child, current_word, words);
        current_word.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn trie_from(words: &[&str]) -> Trie {
        let mut trie = Trie::new();
        for word in words {
            trie.insert(word);
        }
        trie
    }

    #[test]
    fn empty_trie() {
        let trie = Trie::new();
        assert!(trie.is_empty());
        assert!(!trie.contains("a"));
        assert!(trie.words_with_prefix("").is_empty());
    }

    #[test]
    fn insert_and_contains() {
        let trie = trie_from(&["car", "cart", "cat", "dog"]);
        assert_eq!(trie.len(), 4);
        assert!(trie.contains("car"));
        assert!(trie.contains("cart"));
        assert!(!trie.contains("ca"));
        assert!(!trie.contains("carts"));
        assert!(!trie.contains("cow"));
    }

    #[test]
    fn duplicate_insert_is_ignored() {
        let mut trie = trie_from(&["car"]);
        assert!(!trie.insert("car"));
        assert_eq!(trie.len(), 1);
    }

    #[test]
    fn prefix_checks() {
        let trie = trie_from(&["car", "dog"]);
        assert!(trie.starts_with(""));
        assert!(trie.starts_with("ca"));
        assert!(trie.starts_with("car"));
        assert!(!trie.starts_with("cat"));
    }

    #[test]
    fn autocomplete_returns_sorted_words() {
        let trie = trie_from(&["dog", "cat", "cart", "car", "do"]);
        assert_eq!(trie.words_with_prefix("ca"), vec!["car", "cart", "cat"]);
        assert_eq!(trie.words_with_prefix("do"), vec!["do", "dog"]);
        assert_eq!(trie.words_with_prefix(""), vec!["car", "cart", "cat", "do", "dog"]);
        assert!(trie.words_with_prefix("x").is_empty());
    }

    #[test]
    fn works_with_non_ascii_characters() {
        let trie = trie_from(&["alma", "álom", "ágy"]);
        assert!(trie.contains("álom"));
        assert_eq!(trie.words_with_prefix("á"), vec!["ágy", "álom"]);
    }
}
