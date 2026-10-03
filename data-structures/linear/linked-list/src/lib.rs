//! # Singly linked list
//!
//! A **linked list** stores each element in its own *node*. Every node holds a
//! value and a link to the next node. The list itself only remembers the
//! first node, called the **head**.
//!
//! ```text
//!   head
//!    │
//!    ▼
//!  ┌───┬───┐    ┌───┬───┐    ┌───┬──────┐
//!  │ 3 │ ●─┼───►│ 2 │ ●─┼───►│ 1 │ None │
//!  └───┴───┘    └───┴───┘    └───┴──────┘
//!   value next
//! ```
//!
//! For speed, all nodes live in one `Vec` (an *arena*) and link to each other
//! by index instead of by pointer. Removed nodes leave a free slot that the
//! next push reuses. See the README for how this is stored in memory, how the
//! CPU processes it, and why it's faster than one `Box` per node.
//!
//! ## Example
//!
//! ```
//! use linked_list::LinkedList;
//!
//! let mut list = LinkedList::new();
//! list.push_front(1);
//! list.push_front(2);
//! list.push_front(3);
//!
//! // Iterating starts at the head, so the last pushed value comes first.
//! let values: Vec<_> = list.iter().copied().collect();
//! assert_eq!(values, vec![3, 2, 1]);
//!
//! list.reverse();
//! assert_eq!(list.pop_front(), Some(1));
//! ```

/// Marks "no node": the end of the list, or the end of the free list.
const NIL: u32 = u32::MAX;

/// One slot in the arena. A slot either holds a value that is in the list,
/// or is free and waiting to be reused.
#[derive(Debug)]
enum Slot<T> {
    Used { value: T, next: u32 },
    Free { next_free: u32 },
}

/// A singly linked list whose nodes all live in one `Vec`, linked by index.
#[derive(Debug)]
pub struct LinkedList<T> {
    // Every node, used or free, in one contiguous block of memory.
    slots: Vec<Slot<T>>,
    // Index of the first node in the list, or NIL if the list is empty.
    head: u32,
    // Index of the first free slot. Free slots are chained through
    // `next_free`, forming a second linked list of reusable slots.
    free_head: u32,
    len: usize,
}

impl<T> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> LinkedList<T> {
    /// Creates an empty list. Nothing is allocated until the first push.
    pub fn new() -> Self {
        LinkedList {
            slots: Vec::new(),
            head: NIL,
            free_head: NIL,
            len: 0,
        }
    }

    /// Creates an empty list with room for `capacity` nodes.
    pub fn with_capacity(capacity: usize) -> Self {
        LinkedList {
            slots: Vec::with_capacity(capacity),
            ..Self::new()
        }
    }

    /// Adds `value` at the front of the list. **O(1) amortized.**
    pub fn push_front(&mut self, value: T) {
        let node = Slot::Used {
            value,
            next: self.head,
        };
        self.head = if self.free_head != NIL {
            // Reuse a free slot instead of growing the Vec.
            let index = self.free_head;
            match self.slots[index as usize] {
                Slot::Free { next_free } => self.free_head = next_free,
                Slot::Used { .. } => unreachable!("free list points at a used slot"),
            }
            self.slots[index as usize] = node;
            index
        } else {
            // No free slot: append a new one at the end of the Vec.
            let index = self.slots.len();
            assert!(index < NIL as usize, "a LinkedList can hold at most u32::MAX - 1 nodes");
            self.slots.push(node);
            index as u32
        };
        self.len += 1;
    }

    /// Removes the front value and returns it. **O(1).**
    pub fn pop_front(&mut self) -> Option<T> {
        if self.head == NIL {
            return None;
        }
        let index = self.head;
        // The removed node's slot becomes the first free slot.
        let freed = Slot::Free {
            next_free: self.free_head,
        };
        match std::mem::replace(&mut self.slots[index as usize], freed) {
            Slot::Used { value, next } => {
                self.head = next;
                self.free_head = index;
                self.len -= 1;
                Some(value)
            }
            Slot::Free { .. } => unreachable!("head points at a free slot"),
        }
    }

    /// Returns a reference to the front value. **O(1).**
    pub fn peek_front(&self) -> Option<&T> {
        self.iter().next()
    }

    /// Returns the number of values in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list has no values.
    pub fn is_empty(&self) -> bool {
        self.head == NIL
    }

    /// Returns `true` if any value in the list equals `value`. **O(n).**
    pub fn contains(&self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.iter().any(|v| v == value)
    }

    /// Reverses the list in place by flipping every `next` index. **O(n).**
    ///
    /// ```text
    ///   step 0:  reversed: NIL           remaining: [1] ─► [2] ─► [3]
    ///   step 1:  reversed: [1]           remaining: [2] ─► [3]
    ///   step 2:  reversed: [2] ─► [1]    remaining: [3]
    ///   step 3:  reversed: [3] ─► [2] ─► [1]
    /// ```
    pub fn reverse(&mut self) {
        let mut reversed = NIL;
        let mut current = self.head;
        while current != NIL {
            match &mut self.slots[current as usize] {
                Slot::Used { next, .. } => {
                    let following = *next;
                    *next = reversed;
                    reversed = current;
                    current = following;
                }
                Slot::Free { .. } => unreachable!("list points at a free slot"),
            }
        }
        self.head = reversed;
    }

    /// Returns an iterator over references to the values, from front to back.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            slots: &self.slots,
            next: self.head,
        }
    }
}

/// An iterator that borrows the list. Created by [`LinkedList::iter`].
pub struct Iter<'a, T> {
    slots: &'a [Slot<T>],
    next: u32,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next == NIL {
            return None;
        }
        match &self.slots[self.next as usize] {
            Slot::Used { value, next } => {
                self.next = *next;
                Some(value)
            }
            Slot::Free { .. } => unreachable!("list points at a free slot"),
        }
    }
}

/// Lets you write `for value in &list { ... }`.
impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// Lets you build a list with `.collect()`. Values are pushed to the front,
/// so `[1, 2, 3]` becomes the list `3 ─► 2 ─► 1`.
impl<T> FromIterator<T> for LinkedList<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let mut list = LinkedList::with_capacity(iter.size_hint().0);
        for value in iter {
            list.push_front(value);
        }
        list
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_list_is_empty() {
        let list: LinkedList<i32> = LinkedList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.peek_front(), None);
    }

    #[test]
    fn push_and_pop_front() {
        let mut list = LinkedList::new();
        list.push_front(1);
        list.push_front(2);
        assert_eq!(list.len(), 2);
        assert_eq!(list.peek_front(), Some(&2));
        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn iterates_from_front_to_back() {
        let list: LinkedList<_> = [1, 2, 3].into_iter().collect();
        let values: Vec<_> = list.iter().copied().collect();
        assert_eq!(values, vec![3, 2, 1]);
    }

    #[test]
    fn reverse_flips_the_order() {
        let mut list: LinkedList<_> = [1, 2, 3].into_iter().collect();
        list.reverse();
        let values: Vec<_> = (&list).into_iter().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);
        assert_eq!(list.len(), 3);
    }

    #[test]
    fn contains_finds_values() {
        let list: LinkedList<_> = ["a", "b"].into_iter().collect();
        assert!(list.contains(&"a"));
        assert!(!list.contains(&"z"));
    }

    #[test]
    fn freed_slots_are_reused() {
        let mut list = LinkedList::new();
        for i in 0..10 {
            list.push_front(i);
        }
        for _ in 0..5 {
            list.pop_front();
        }
        for i in 10..15 {
            list.push_front(i);
        }
        // Still only 10 slots: the 5 freed ones were reused.
        assert_eq!(list.slots.len(), 10);
        let values: Vec<_> = list.iter().copied().collect();
        assert_eq!(values, vec![14, 13, 12, 11, 10, 4, 3, 2, 1, 0]);
    }

    #[test]
    fn dropping_frees_every_value() {
        use std::rc::Rc;
        let tracker = Rc::new(());
        let mut list = LinkedList::new();
        for _ in 0..10 {
            list.push_front(Rc::clone(&tracker));
        }
        list.pop_front(); // a value that left the list is dropped by the caller
        assert_eq!(Rc::strong_count(&tracker), 10);
        drop(list);
        assert_eq!(Rc::strong_count(&tracker), 1);
    }

    #[test]
    fn memory_layout_matches_the_readme() {
        use std::mem::size_of;
        // tag (4) + next (4) + value (8)
        assert_eq!(size_of::<Slot<u64>>(), 16);
        // Vec (24) + len (8) + head (4) + free_head (4)
        assert_eq!(size_of::<LinkedList<u64>>(), 40);
        // The original Box-based node, for comparison: value (8) + pointer (8).
        struct BoxNode {
            _value: u64,
            _next: Option<Box<BoxNode>>,
        }
        assert_eq!(size_of::<BoxNode>(), 16);
    }

    #[test]
    fn dropping_a_long_list_does_not_overflow_the_stack() {
        let list: LinkedList<_> = (0..1_000_000).collect();
        drop(list);
    }
}
