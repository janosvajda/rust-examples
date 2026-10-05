//! # Binary heap (min-heap)
//!
//! A **binary heap** is a tree where every parent is smaller than (or equal to)
//! its children. This is called the **heap property**. It means the smallest
//! value is always at the root, so you can read it in O(1) and remove it in
//! O(log n).
//!
//! ```text
//!              1                 Every parent ≤ its children.
//!            /   \               Siblings are in no particular order:
//!           3     2              3 and 2 could be swapped.
//!          / \   /
//!         7   4 5
//! ```
//!
//! This is a **min-heap**. A **max-heap** is the same with the comparison flipped.
//!
//! ## The tree lives in a `Vec`
//!
//! A binary heap is always a *complete* tree: every level is full except maybe
//! the last, which fills from left to right. Because there are no gaps, the
//! tree can be stored in a plain `Vec`, level by level, with no pointers at all:
//!
//! ```text
//!   index:   0   1   2   3   4   5
//!          ┌───┬───┬───┬───┬───┬───┐
//!          │ 1 │ 3 │ 2 │ 7 │ 4 │ 5 │
//!          └───┴───┴───┴───┴───┴───┘
//!
//!   For the node at index i:
//!     parent      = (i - 1) / 2
//!     left child  = 2 * i + 1
//!     right child = 2 * i + 2
//! ```
//!
//! ## The two key moves
//!
//! - **Sift up** (after `push`): put the new value at the end, then swap it
//!   with its parent while it is smaller than the parent.
//! - **Sift down** (after `pop`): move the last value to the root, then swap it
//!   with its smaller child while it is bigger than that child.
//!
//! Both walk one path between the root and a leaf, and the tree's height is
//! log₂(n), so both are **O(log n)**.
//!
//! ## Complexity
//!
//! | Operation   | Time       |
//! |-------------|------------|
//! | `push`      | O(log n)   |
//! | `pop`       | O(log n)   |
//! | `peek`      | O(1)       |
//! | `from_vec`  | O(n), see [`MinHeap::from_vec`] for why it's not O(n log n) |
//! | [`heap_sort`] | O(n log n) |
//!
//! ## Where heaps are used
//!
//! - **Priority queues**: task schedulers, event simulations.
//! - **Dijkstra's shortest path** (see the `graphs/dijkstra` example).
//! - Finding the k smallest or largest items in a large stream.
//!
//! The standard library has `std::collections::BinaryHeap`, a **max**-heap.
//! Wrap values in `std::cmp::Reverse` to use it as a min-heap.
//!
//! ## Example
//!
//! ```
//! use binary_heap::MinHeap;
//!
//! let mut heap = MinHeap::new();
//! for value in [5, 2, 10, 1] {
//!     heap.push(value);
//! }
//!
//! assert_eq!(heap.peek(), Some(&1));
//! assert_eq!(heap.pop(), Some(1));
//! assert_eq!(heap.pop(), Some(2));
//! ```

/// A min-heap: `pop` always returns the smallest value.
///
/// `T: Ord` means the values must be comparable with `<`, `>` and so on.
/// That's the only thing a heap needs to know about its values.
#[derive(Debug)]
pub struct MinHeap<T: Ord> {
    data: Vec<T>,
}

impl<T: Ord> Default for MinHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> MinHeap<T> {
    /// Creates an empty heap.
    pub fn new() -> Self {
        MinHeap { data: Vec::new() }
    }

    /// Builds a heap from an existing vector in **O(n)**.
    ///
    /// The simple approach would be to `push` each value, which costs
    /// O(n log n). Instead we *heapify* the vector in place: we sift down every
    /// parent node, starting with the last parent and moving towards the root.
    ///
    /// Why is this O(n)? About half the nodes are leaves and need no work at all.
    /// A quarter sit one level up and sift down at most 1 step, an eighth at most
    /// 2 steps, and so on. Added up, that is less than 2n steps in total.
    pub fn from_vec(data: Vec<T>) -> Self {
        let mut heap = MinHeap { data };
        // Nodes at index len/2 and beyond have no children, so they are
        // already valid heaps of size 1. Start from the last parent.
        for index in (0..heap.data.len() / 2).rev() {
            heap.sift_down(index);
        }
        heap
    }

    /// Adds `value` to the heap. **O(log n).**
    ///
    /// ```text
    ///   push(0):         1                 1                 0
    ///                  /   \             /   \             /   \
    ///                 3     2    ──►    3     0    ──►    3     1
    ///                / \   / \         / \   / \         / \   / \
    ///               7   4 5   0       7   4 5   2       7   4 5   2
    ///                 add at end      0 < 2, swap       0 < 1, swap
    /// ```
    pub fn push(&mut self, value: T) {
        self.data.push(value);
        self.sift_up(self.data.len() - 1);
    }

    /// Removes and returns the smallest value, or `None` if the heap is empty. **O(log n).**
    pub fn pop(&mut self) -> Option<T> {
        if self.data.is_empty() {
            return None;
        }
        // Move the root (the minimum) to the end so we can `pop` it off the Vec
        // in O(1). The previous last value is now at the root, probably in the
        // wrong place, so we sift it down.
        let last = self.data.len() - 1;
        self.data.swap(0, last);
        let min = self.data.pop();
        if !self.data.is_empty() {
            self.sift_down(0);
        }
        min
    }

    /// Returns a reference to the smallest value without removing it. **O(1).**
    pub fn peek(&self) -> Option<&T> {
        self.data.first()
    }

    /// Returns the number of values in the heap.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns `true` if the heap has no values.
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Consumes the heap and returns its values in ascending order.
    pub fn into_sorted_vec(mut self) -> Vec<T> {
        let mut sorted = Vec::with_capacity(self.len());
        while let Some(value) = self.pop() {
            sorted.push(value);
        }
        sorted
    }

    /// Moves the value at `index` up until its parent is not bigger than it.
    fn sift_up(&mut self, mut index: usize) {
        while index > 0 {
            let parent = (index - 1) / 2;
            if self.data[parent] <= self.data[index] {
                break; // heap property holds, we're done
            }
            self.data.swap(parent, index);
            index = parent;
        }
    }

    /// Moves the value at `index` down until neither child is smaller than it.
    fn sift_down(&mut self, mut index: usize) {
        let len = self.data.len();
        loop {
            let left = 2 * index + 1;
            let right = left + 1;

            // Find the smallest of: this node, its left child, its right child.
            let mut smallest = index;
            if left < len && self.data[left] < self.data[smallest] {
                smallest = left;
            }
            if right < len && self.data[right] < self.data[smallest] {
                smallest = right;
            }

            if smallest == index {
                break; // already smaller than both children
            }
            // Swap with the *smaller* child, so the new parent is smaller than
            // the other child too.
            self.data.swap(index, smallest);
            index = smallest;
        }
    }
}

/// Sorts a vector using a heap: build a heap in O(n), then pop n times at
/// O(log n) each. **O(n log n)** in total, even in the worst case.
///
/// ```
/// use binary_heap::heap_sort;
///
/// assert_eq!(heap_sort(vec![3, 1, 2]), vec![1, 2, 3]);
/// ```
pub fn heap_sort<T: Ord>(values: Vec<T>) -> Vec<T> {
    MinHeap::from_vec(values).into_sorted_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks the heap property for every parent/child pair.
    fn is_valid_heap<T: Ord>(data: &[T]) -> bool {
        (1..data.len()).all(|i| data[(i - 1) / 2] <= data[i])
    }

    #[test]
    fn pops_in_ascending_order() {
        let mut heap = MinHeap::new();
        for value in [5, 2, 10, 1] {
            heap.push(value);
            assert!(is_valid_heap(&heap.data));
        }
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(5));
        assert_eq!(heap.pop(), Some(10));
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn empty_heap() {
        let mut heap: MinHeap<i32> = MinHeap::new();
        assert!(heap.is_empty());
        assert_eq!(heap.peek(), None);
        assert_eq!(heap.pop(), None);
    }

    #[test]
    fn from_vec_builds_a_valid_heap() {
        let heap = MinHeap::from_vec(vec![9, 4, 7, 1, 8, 2, 6, 3, 5]);
        assert!(is_valid_heap(&heap.data));
        assert_eq!(heap.peek(), Some(&1));
    }

    #[test]
    fn handles_duplicates() {
        assert_eq!(heap_sort(vec![3, 1, 3, 1, 2]), vec![1, 1, 2, 3, 3]);
    }

    #[test]
    fn heap_sort_matches_std_sort() {
        let values: Vec<i32> = (0..200).map(|i| (i * 37 + 11) % 101).collect();
        let mut expected = values.clone();
        expected.sort();
        assert_eq!(heap_sort(values), expected);
    }

    #[test]
    fn works_with_tuples_as_priorities() {
        // Tuples compare element by element, so (priority, name) sorts by priority first.
        let mut heap = MinHeap::new();
        heap.push((2, "write tests"));
        heap.push((1, "fix production bug"));
        heap.push((3, "refactor"));
        assert_eq!(heap.pop(), Some((1, "fix production bug")));
    }
}
