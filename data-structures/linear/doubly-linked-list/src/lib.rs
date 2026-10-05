//! # Doubly linked list (`Rc`, `RefCell` and `Weak`)
//!
//! In a **doubly** linked list every node points to both its **next** and its
//! **previous** node, and the list remembers both ends. That makes adding and
//! removing at *either* end O(1), and lets you walk the list in both directions.
//!
//! ```text
//!   head                                 tail
//!    │                                     │
//!    ▼                                     ▼
//!  ┌───┐ ──next──► ┌───┐ ──next──► ┌───┐
//!  │ A │           │ B │           │ C │
//!  └───┘ ◄──prev── └───┘ ◄──prev── └───┘
//! ```
//!
//! ## Why this is hard in Rust
//!
//! In the singly linked list (see `linear/linked-list`) each node had exactly
//! one owner, the node before it, so `Box` was enough. Here node B is pointed
//! to by A (`next`) *and* by C (`prev`). `Box` can't express two owners, so we
//! need the standard library's shared-ownership tools:
//!
//! | Tool        | What it does                                                    |
//! |-------------|-----------------------------------------------------------------|
//! | `Rc<T>`     | **R**eference **c**ounted pointer: several owners share one value. Freed when the last `Rc` is dropped. |
//! | `RefCell<T>`| Lets you mutate through a shared reference. Rust's borrow rules are checked **at runtime** instead of compile time (`borrow()` / `borrow_mut()`). |
//! | `Weak<T>`   | A pointer that does **not** keep the value alive. You `upgrade()` it to an `Rc` when you need it, which fails if the value is gone. |
//!
//! ## The reference cycle trap
//!
//! If both `next` and `prev` were `Rc`, A would keep B alive and B would keep A
//! alive. Their counts would never reach zero, so the memory would **leak**
//! even after the list is dropped. Rust's ownership rules don't prevent this.
//!
//! The fix: the forward links (`next`) are strong `Rc`s and own the nodes,
//! while the backward links (`prev`) are `Weak` and don't. There's a test
//! below (`dropping_the_list_frees_every_value`) that checks nothing leaks.
//!
//! ## Alternatives
//!
//! - In real code use `std::collections::VecDeque` (or `std::collections::LinkedList`).
//! - Another common Rust approach is an **arena**: keep all nodes in a `Vec`
//!   and link them by index instead of by pointer. No `Rc` or `RefCell` needed.
//!   The `hashing/lru-cache` example does this.
//!
//! ## Complexity
//!
//! | Operation                 | Time |
//! |---------------------------|------|
//! | `push_front` / `push_back`| O(1) |
//! | `pop_front` / `pop_back`  | O(1) |
//! | `peek_front` / `peek_back`| O(1) |
//!
//! ## Example
//!
//! ```
//! use doubly_linked_list::DoublyLinkedList;
//!
//! let mut list = DoublyLinkedList::new();
//! list.push_back(2);
//! list.push_back(3);
//! list.push_front(1);
//!
//! assert_eq!(list.to_vec(), vec![1, 2, 3]);
//! assert_eq!(list.to_vec_reversed(), vec![3, 2, 1]);
//! assert_eq!(list.pop_back(), Some(3));
//! assert_eq!(list.pop_front(), Some(1));
//! ```

use std::cell::{Ref, RefCell};
use std::rc::{Rc, Weak};

/// A strong, owning link to a node (or `None` at the end of the list).
type Link<T> = Option<Rc<RefCell<Node<T>>>>;

/// A non-owning link back to the previous node.
type WeakLink<T> = Option<Weak<RefCell<Node<T>>>>;

struct Node<T> {
    value: T,
    next: Link<T>,
    prev: WeakLink<T>,
}

impl<T> Node<T> {
    fn new_shared(value: T) -> Rc<RefCell<Node<T>>> {
        Rc::new(RefCell::new(Node {
            value,
            next: None,
            prev: None,
        }))
    }
}

/// A doubly linked list with O(1) operations at both ends.
pub struct DoublyLinkedList<T> {
    head: Link<T>,
    // `tail` is a second strong reference to the last node. That's fine: it
    // doesn't form a cycle, because nothing points back to the list itself.
    tail: Link<T>,
    len: usize,
}

impl<T> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> DoublyLinkedList<T> {
    /// Creates an empty list.
    pub fn new() -> Self {
        DoublyLinkedList {
            head: None,
            tail: None,
            len: 0,
        }
    }

    /// Adds `value` at the front. **O(1).**
    pub fn push_front(&mut self, value: T) {
        let new_node = Node::new_shared(value);
        match self.head.take() {
            Some(old_head) => {
                // old_head ◄──prev── (weak)   new_node ──next──► old_head (strong)
                old_head.borrow_mut().prev = Some(Rc::downgrade(&new_node));
                new_node.borrow_mut().next = Some(old_head);
                self.head = Some(new_node);
            }
            None => {
                // The only node is both the head and the tail.
                self.tail = Some(Rc::clone(&new_node));
                self.head = Some(new_node);
            }
        }
        self.len += 1;
    }

    /// Adds `value` at the back. **O(1).**
    pub fn push_back(&mut self, value: T) {
        let new_node = Node::new_shared(value);
        match self.tail.take() {
            Some(old_tail) => {
                new_node.borrow_mut().prev = Some(Rc::downgrade(&old_tail));
                old_tail.borrow_mut().next = Some(Rc::clone(&new_node));
                self.tail = Some(new_node);
            }
            None => {
                self.head = Some(Rc::clone(&new_node));
                self.tail = Some(new_node);
            }
        }
        self.len += 1;
    }

    /// Removes and returns the front value. **O(1).**
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|old_head| {
            match old_head.borrow_mut().next.take() {
                Some(new_head) => {
                    // The new head has nothing before it.
                    new_head.borrow_mut().prev = None;
                    self.head = Some(new_head);
                }
                None => {
                    // That was the last node, so drop the tail's reference too.
                    self.tail = None;
                }
            }
            self.len -= 1;
            into_value(old_head)
        })
    }

    /// Removes and returns the back value. **O(1).**
    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|old_tail| {
            // `upgrade` turns the Weak back into an Rc. It succeeds here because
            // the previous node is still owned by the node before it (or the head).
            match old_tail
                .borrow_mut()
                .prev
                .take()
                .and_then(|weak| weak.upgrade())
            {
                Some(new_tail) => {
                    // Drop the new tail's strong link to the node we're removing.
                    new_tail.borrow_mut().next = None;
                    self.tail = Some(new_tail);
                }
                None => {
                    self.head = None;
                }
            }
            self.len -= 1;
            into_value(old_tail)
        })
    }

    /// Returns a reference to the front value.
    ///
    /// The value lives inside a `RefCell`, so we can't hand out a plain `&T`.
    /// Instead we return a `Ref<T>`, a guard that keeps the `RefCell` borrowed
    /// for as long as you hold it. It derefs to `&T`, so `*list.peek_front().unwrap()`
    /// works like a normal reference.
    pub fn peek_front(&self) -> Option<Ref<'_, T>> {
        self.head
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.value))
    }

    /// Returns a reference to the back value. See [`DoublyLinkedList::peek_front`].
    pub fn peek_back(&self) -> Option<Ref<'_, T>> {
        self.tail
            .as_ref()
            .map(|node| Ref::map(node.borrow(), |node| &node.value))
    }

    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list is empty.
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Returns copies of the values from front to back, following `next` links.
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut values = Vec::with_capacity(self.len);
        // Cloning an Rc only increments a counter, it doesn't copy the node.
        let mut current = self.head.clone();
        while let Some(node) = current {
            let node = node.borrow();
            values.push(node.value.clone());
            current = node.next.clone();
        }
        values
    }

    /// Returns copies of the values from back to front, following `prev` links.
    pub fn to_vec_reversed(&self) -> Vec<T>
    where
        T: Clone,
    {
        let mut values = Vec::with_capacity(self.len);
        let mut current = self.tail.clone();
        while let Some(node) = current {
            let node = node.borrow();
            values.push(node.value.clone());
            current = node.prev.as_ref().and_then(Weak::upgrade);
        }
        values
    }
}

/// Takes the value out of a node that has just been unlinked.
///
/// `Rc::try_unwrap` only succeeds if this is the *last* strong reference. After
/// unlinking, nothing else owns the node (the neighbour's link to it was `Weak`
/// or has been removed), so it always succeeds here.
fn into_value<T>(node: Rc<RefCell<Node<T>>>) -> T {
    match Rc::try_unwrap(node) {
        Ok(cell) => cell.into_inner().value,
        Err(_) => unreachable!("an unlinked node has no other strong references"),
    }
}

/// Frees the nodes one at a time. The compiler-generated drop would be
/// recursive along the `next` chain and could overflow the stack on long lists.
impl<T> Drop for DoublyLinkedList<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_list_is_empty() {
        let list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        assert!(list.is_empty());
        assert!(list.peek_front().is_none());
        assert!(list.peek_back().is_none());
    }

    #[test]
    fn push_and_pop_at_both_ends() {
        let mut list = DoublyLinkedList::new();
        list.push_back(2);
        list.push_front(1);
        list.push_back(3);
        assert_eq!(list.len(), 3);
        assert_eq!(*list.peek_front().unwrap(), 1);
        assert_eq!(*list.peek_back().unwrap(), 3);

        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_back(), Some(3));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), None);
        assert_eq!(list.pop_front(), None);
        assert!(list.is_empty());
    }

    #[test]
    fn walks_in_both_directions() {
        let mut list = DoublyLinkedList::new();
        for i in 1..=4 {
            list.push_back(i);
        }
        assert_eq!(list.to_vec(), vec![1, 2, 3, 4]);
        assert_eq!(list.to_vec_reversed(), vec![4, 3, 2, 1]);
    }

    #[test]
    fn can_be_reused_after_being_emptied() {
        let mut list = DoublyLinkedList::new();
        list.push_front("a");
        list.pop_back();
        list.push_back("b");
        assert_eq!(list.to_vec(), vec!["b"]);
        assert_eq!(list.to_vec_reversed(), vec!["b"]);
    }

    #[test]
    fn dropping_the_list_frees_every_value() {
        // Every value holds a clone of `tracker`. If any node leaked (for
        // example because of an Rc cycle), its clone would never be dropped
        // and the count would stay above 1.
        let tracker = Rc::new(());
        let mut list = DoublyLinkedList::new();
        for _ in 0..10 {
            list.push_back(Rc::clone(&tracker));
        }
        assert_eq!(Rc::strong_count(&tracker), 11);

        drop(list);
        assert_eq!(Rc::strong_count(&tracker), 1);
    }

    #[test]
    fn dropping_a_long_list_does_not_overflow_the_stack() {
        let mut list = DoublyLinkedList::new();
        for i in 0..1_000_000 {
            list.push_back(i);
        }
        drop(list);
    }
}
