//! # Stack
//!
//! A **stack** is a collection where the last item you put in is the first item
//! you take out: **LIFO** (Last In, First Out). Think of a stack of plates.
//!
//! ```text
//!         push(3)            pop() -> 3
//!            │                   ▲
//!            ▼                   │
//!   ┌───┐  ┌───┐              ┌───┐
//!   │   │  │ 3 │ ◄── top      │   │
//!   │ 2 │  │ 2 │              │ 2 │ ◄── top
//!   │ 1 │  │ 1 │              │ 1 │
//!   └───┘  └───┘              └───┘
//! ```
//!
//! The stack is backed by a `Vec<T>`: the end of the vector is the top of the
//! stack. See the README for how it is stored in memory, how the CPU
//! processes it, and the optimisations in this crate.
//!
//! ## Example
//!
//! ```
//! use stack::Stack;
//!
//! let mut stack = Stack::new();
//! stack.push(1);
//! stack.push(2);
//!
//! assert_eq!(stack.peek(), Some(&2));
//! assert_eq!(stack.pop(), Some(2));
//! assert_eq!(stack.pop(), Some(1));
//! assert_eq!(stack.pop(), None);
//! ```

/// A LIFO stack backed by a `Vec<T>`.
///
/// It is generic over `T`, so it can hold any type: `Stack<i32>`,
/// `Stack<String>`, `Stack<char>`, and so on.
#[derive(Debug)]
pub struct Stack<T> {
    // The last element of the Vec is the top of the stack.
    items: Vec<T>,
}

impl<T> Default for Stack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Stack<T> {
    /// Creates an empty stack. No memory is allocated until the first `push`.
    pub fn new() -> Self {
        Stack { items: Vec::new() }
    }

    /// Creates an empty stack with room for `capacity` items, so the first
    /// `capacity` pushes never reallocate.
    pub fn with_capacity(capacity: usize) -> Self {
        Stack {
            items: Vec::with_capacity(capacity),
        }
    }

    /// Puts `item` on top of the stack. **O(1) amortized.**
    pub fn push(&mut self, item: T) {
        self.items.push(item);
    }

    /// Removes and returns the top item, or `None` if the stack is empty. **O(1).**
    ///
    /// Returning an `Option` instead of panicking makes the caller handle the
    /// empty case. This is the usual Rust style.
    pub fn pop(&mut self) -> Option<T> {
        self.items.pop()
    }

    /// Returns a reference to the top item without removing it. **O(1).**
    pub fn peek(&self) -> Option<&T> {
        self.items.last()
    }

    /// Returns the number of items on the stack.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` if the stack has no items.
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns how many items fit before the next reallocation.
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }
}

/// Pushes every item from an iterator, in order (the last one ends up on top).
///
/// Faster than calling `push` in a loop when the iterator knows its length
/// (ranges, slices, `Vec`s, …): space is reserved **once**, and the items are
/// written without a capacity check per item.
impl<T> Extend<T> for Stack<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, items: I) {
        self.items.extend(items);
    }
}

/// Marks a byte in [`BRACKETS`] as a closing bracket.
const CLOSER: u8 = 1;

/// Classifies every possible byte with a single lookup:
/// `0` = not a bracket, [`CLOSER`] = a closing bracket, and for an opening
/// bracket, the closing bracket it expects.
///
/// Computed at compile time, so it costs nothing at runtime.
static BRACKETS: [u8; 256] = {
    let mut table = [0u8; 256];
    table[b'(' as usize] = b')';
    table[b'[' as usize] = b']';
    table[b'{' as usize] = b'}';
    table[b')' as usize] = CLOSER;
    table[b']' as usize] = CLOSER;
    table[b'}' as usize] = CLOSER;
    table
};

/// Checks that every opening bracket in `text` is closed by the matching
/// bracket, in the right order. A classic use of a stack.
///
/// The idea:
/// 1. When we see an opening bracket `(`, `[` or `{`, we push the closing
///    bracket we now expect.
/// 2. When we see a closing bracket, it must be the most recently expected
///    one. That is exactly the top of the stack, so we pop and compare.
/// 3. At the end the stack must be empty, otherwise something was never closed.
///
/// Any other characters are ignored. It works on bytes and uses a lookup
/// table for speed; the README explains why that is safe and faster.
///
/// ```
/// use stack::is_balanced;
///
/// assert!(is_balanced("fn main() { let v = vec![1, 2]; }"));
/// assert!(!is_balanced("(]"));   // wrong closing bracket
/// assert!(!is_balanced("(()"));  // never closed
/// ```
pub fn is_balanced(text: &str) -> bool {
    let mut expected_closers: Stack<u8> = Stack::new();

    for &byte in text.as_bytes() {
        let class = BRACKETS[byte as usize];
        if class == 0 {
            continue; // most bytes: not a bracket
        }
        if class == CLOSER {
            // `pop` returns None if there is no open bracket to close.
            if expected_closers.pop() != Some(byte) {
                return false;
            }
        } else {
            expected_closers.push(class);
        }
    }

    expected_closers.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn new_stack_is_empty() {
        let stack: Stack<i32> = Stack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
        assert_eq!(stack.peek(), None);
        assert_eq!(stack.capacity(), 0); // no allocation yet
    }

    #[test]
    fn pops_in_reverse_order_of_pushes() {
        let mut stack = Stack::new();
        for i in 1..=3 {
            stack.push(i);
        }
        assert_eq!(stack.len(), 3);
        assert_eq!(stack.pop(), Some(3));
        assert_eq!(stack.pop(), Some(2));
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), None);
    }

    #[test]
    fn peek_does_not_remove() {
        let mut stack = Stack::new();
        stack.push("a");
        assert_eq!(stack.peek(), Some(&"a"));
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn with_capacity_does_not_reallocate() {
        let mut stack = Stack::with_capacity(100);
        let capacity = stack.capacity();
        assert!(capacity >= 100);
        for i in 0..100 {
            stack.push(i);
        }
        assert_eq!(stack.capacity(), capacity);
    }

    #[test]
    fn growth_doubles_capacity() {
        let mut stack = Stack::new();
        let mut capacities = vec![];
        for i in 0..100_000u64 {
            stack.push(i);
            if capacities.last() != Some(&stack.capacity()) {
                capacities.push(stack.capacity());
            }
        }
        // 4, 8, 16, ... 131072: 16 allocations, so 15 reallocations.
        assert_eq!(capacities.first(), Some(&4));
        assert_eq!(capacities.len(), 16);
        assert!(capacities.windows(2).all(|w| w[1] == w[0] * 2));
    }

    #[test]
    fn memory_layout_matches_the_readme() {
        // Three machine words: pointer, capacity, length.
        assert_eq!(size_of::<Stack<u64>>(), 3 * size_of::<usize>());
        assert_eq!(align_of::<Stack<u64>>(), align_of::<usize>());
        // The size of the stack itself doesn't depend on T: the items are on the heap.
        assert_eq!(size_of::<Stack<[u8; 1000]>>(), size_of::<Stack<u8>>());
        // The niche: Option<Stack<T>> needs no extra space.
        assert_eq!(size_of::<Option<Stack<u64>>>(), size_of::<Stack<u64>>());
    }

    #[test]
    fn balanced_brackets() {
        assert!(is_balanced(""));
        assert!(is_balanced("()[]{}"));
        assert!(is_balanced("{[()()]}"));
        assert!(!is_balanced(")"));
        assert!(!is_balanced("([)]"));
        assert!(!is_balanced("{"));
    }

    #[test]
    fn multi_byte_characters_are_ignored_correctly() {
        // Every byte of a multi-byte UTF-8 character is >= 0x80, so none of
        // them can be mistaken for an ASCII bracket.
        assert!(is_balanced("(ünïcödé 🦀 [テキスト] {∑})"));
        assert!(!is_balanced("(🦀]"));
    }

    #[test]
    fn table_matches_a_simple_match() {
        // The lookup table must classify every byte exactly like the obvious code.
        for byte in 0..=255u8 {
            let expected = match byte {
                b'(' => b')',
                b'[' => b']',
                b'{' => b'}',
                b')' | b']' | b'}' => CLOSER,
                _ => 0,
            };
            assert_eq!(BRACKETS[byte as usize], expected, "byte {byte:#04x}");
        }
    }
}
