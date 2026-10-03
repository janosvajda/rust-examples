//! # Queue (ring buffer)
//!
//! A **queue** is a collection where the first item you put in is the first item
//! you take out: **FIFO** (First In, First Out). Think of people waiting in line.
//! New people join at the **back**, and the person at the **front** is served first.
//!
//! ```text
//!   dequeue() ◄── front [ A | B | C ] back ◄── enqueue(D)
//! ```
//!
//! ## Why not just use a `Vec`?
//!
//! Adding to the back of a `Vec` is cheap, but removing from the front with
//! `vec.remove(0)` is **O(n)**: every remaining element has to shift one place
//! to the left.
//!
//! ## The ring buffer trick
//!
//! Instead of moving the elements, we move the *front index*. The buffer is
//! treated as a circle: when an index runs past the end, it wraps around to 0
//! using the remainder operator `%`.
//!
//! ```text
//!   capacity = 4, after enqueue A,B,C,D then dequeue twice, then enqueue E:
//!
//!   index:   0     1     2     3
//!          ┌─────┬─────┬─────┬─────┐
//!          │  E  │  -  │  C  │  D  │
//!          └─────┴─────┴─────┴─────┘
//!                         ▲
//!                       head = 2, len = 3
//!
//!   front is buffer[2] (C), then D, then wrap around to buffer[0] (E).
//!   The slot for the i-th element is (head + i) % capacity.
//! ```
//!
//! When the buffer is full, we allocate one twice as big and copy the elements
//! into it in order, starting at index 0.
//!
//! The standard library has a ready-made ring buffer: `std::collections::VecDeque`.
//! In real code use that. This example shows how it works inside.
//!
//! ## Complexity
//!
//! | Operation | Time           |
//! |-----------|----------------|
//! | `enqueue` | O(1) amortized (O(n) only when the buffer has to grow) |
//! | `dequeue` | O(1)           |
//! | `peek`    | O(1)           |
//!
//! ## Example
//!
//! ```
//! use queue::Queue;
//!
//! let mut queue = Queue::new();
//! queue.enqueue("first");
//! queue.enqueue("second");
//!
//! assert_eq!(queue.dequeue(), Some("first"));
//! assert_eq!(queue.dequeue(), Some("second"));
//! assert_eq!(queue.dequeue(), None);
//! ```

/// The capacity used for the first allocation.
const INITIAL_CAPACITY: usize = 4;

/// A FIFO queue stored in a growable ring buffer.
#[derive(Debug)]
pub struct Queue<T> {
    // Each slot is either empty (None) or holds an element (Some).
    // Using Option<T> lets us take an element out of a slot and leave a hole,
    // without needing `unsafe` code.
    buffer: Vec<Option<T>>,
    // Index of the front element in `buffer`.
    head: usize,
    // Number of elements currently in the queue.
    len: usize,
}

impl<T> Default for Queue<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Queue<T> {
    /// Creates an empty queue. No memory is allocated until the first `enqueue`.
    pub fn new() -> Self {
        Queue {
            buffer: Vec::new(),
            head: 0,
            len: 0,
        }
    }

    /// Adds `item` to the back of the queue. **O(1) amortized.**
    pub fn enqueue(&mut self, item: T) {
        if self.len == self.buffer.len() {
            self.grow();
        }
        // The back of the queue is `len` places after the head, wrapping around.
        let back = (self.head + self.len) % self.buffer.len();
        self.buffer[back] = Some(item);
        self.len += 1;
    }

    /// Removes and returns the front item, or `None` if the queue is empty. **O(1).**
    pub fn dequeue(&mut self) -> Option<T> {
        if self.len == 0 {
            return None;
        }
        // `take` moves the value out and leaves `None` in the slot.
        let item = self.buffer[self.head].take();
        self.head = (self.head + 1) % self.buffer.len();
        self.len -= 1;
        item
    }

    /// Returns a reference to the front item without removing it. **O(1).**
    pub fn peek(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        self.buffer[self.head].as_ref()
    }

    /// Returns the number of items in the queue.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the queue has no items.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns how many items fit before the buffer has to grow again.
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    /// Doubles the buffer size and "unwraps" the elements so the front is at index 0.
    ///
    /// ```text
    ///   before (full):  [ C | D | A | B ]  head = 2
    ///   after:          [ A | B | C | D | - | - | - | - ]  head = 0
    /// ```
    fn grow(&mut self) {
        let new_capacity = (self.buffer.len() * 2).max(INITIAL_CAPACITY);
        let mut new_buffer: Vec<Option<T>> = Vec::with_capacity(new_capacity);

        // Move the elements across in queue order (front to back).
        for i in 0..self.len {
            let index = (self.head + i) % self.buffer.len();
            new_buffer.push(self.buffer[index].take());
        }
        // Fill the rest with empty slots.
        new_buffer.resize_with(new_capacity, || None);

        self.buffer = new_buffer;
        self.head = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_queue_is_empty() {
        let queue: Queue<i32> = Queue::new();
        assert!(queue.is_empty());
        assert_eq!(queue.capacity(), 0);
        assert_eq!(queue.peek(), None);
    }

    #[test]
    fn dequeues_in_insertion_order() {
        let mut queue = Queue::new();
        for i in 1..=3 {
            queue.enqueue(i);
        }
        assert_eq!(queue.peek(), Some(&1));
        assert_eq!(queue.dequeue(), Some(1));
        assert_eq!(queue.dequeue(), Some(2));
        assert_eq!(queue.dequeue(), Some(3));
        assert_eq!(queue.dequeue(), None);
    }

    #[test]
    fn wraps_around_without_growing() {
        let mut queue = Queue::new();
        for i in 0..4 {
            queue.enqueue(i);
        }
        assert_eq!(queue.capacity(), 4);

        // Free two slots at the front, then fill them again from the back.
        assert_eq!(queue.dequeue(), Some(0));
        assert_eq!(queue.dequeue(), Some(1));
        queue.enqueue(4);
        queue.enqueue(5);

        // The new items wrapped around into the freed slots: no growth needed.
        assert_eq!(queue.capacity(), 4);
        let drained: Vec<_> = std::iter::from_fn(|| queue.dequeue()).collect();
        assert_eq!(drained, vec![2, 3, 4, 5]);
    }

    #[test]
    fn grows_while_wrapped_and_keeps_order() {
        let mut queue = Queue::new();
        for i in 0..4 {
            queue.enqueue(i);
        }
        queue.dequeue();
        queue.enqueue(4); // now wrapped: [4, 1, 2, 3], head = 1
        queue.enqueue(5); // full, so it grows

        assert_eq!(queue.capacity(), 8);
        let drained: Vec<_> = std::iter::from_fn(|| queue.dequeue()).collect();
        assert_eq!(drained, vec![1, 2, 3, 4, 5]);
    }
}
