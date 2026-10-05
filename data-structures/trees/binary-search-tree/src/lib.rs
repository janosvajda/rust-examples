//! # Binary search tree (BST)
//!
//! A **binary tree** is made of nodes, where each node has at most two
//! children: a *left* one and a *right* one. A **binary search tree** adds one
//! rule, the **BST property**:
//!
//! > Everything in a node's **left** subtree is **smaller** than the node,
//! > and everything in its **right** subtree is **bigger**.
//!
//! ```text
//!               8
//!             /   \
//!            3     10
//!           / \      \
//!          1   6      14
//!             / \     /
//!            4   7   13
//! ```
//!
//! To find 6, start at the root: 6 < 8 so go left, 6 > 3 so go right, found it.
//! Every comparison discards a whole subtree, like guessing a number with
//! "higher / lower" hints.
//!
//! ## Sorted order for free
//!
//! Visiting **left subtree → node → right subtree** (an *in-order traversal*)
//! produces the values in ascending order: `1 3 4 6 7 8 10 13 14`.
//!
//! ## The catch: balance
//!
//! Operations take time proportional to the tree's **height**. A tree built from
//! random values has height around log₂(n). But insert values that are already
//! sorted, and every new node goes to the right of the previous one:
//!
//! ```text
//!   insert 1, 2, 3, 4:     1
//!                           \
//!                            2          height = n
//!                             \         ...it's just a linked list!
//!                              3
//!                               \
//!                                4
//! ```
//!
//! *Self-balancing* trees (AVL, red-black, B-trees) fix this by restructuring
//! themselves on insert and remove. Rust's `std::collections::BTreeMap` and
//! `BTreeSet` use a B-tree, so they are always O(log n). Use those in real code.
//!
//! ## Complexity
//!
//! | Operation  | Balanced tree | Worst case (degenerate) |
//! |------------|---------------|-------------------------|
//! | `insert`   | O(log n)      | O(n)                    |
//! | `contains` | O(log n)      | O(n)                    |
//! | `remove`   | O(log n)      | O(n)                    |
//! | `in_order` | O(n)          | O(n)                    |
//!
//! ## Example
//!
//! ```
//! use binary_search_tree::BinarySearchTree;
//!
//! let mut tree = BinarySearchTree::new();
//! for value in [8, 3, 10, 1, 6] {
//!     tree.insert(value);
//! }
//!
//! assert!(tree.contains(&6));
//! assert_eq!(tree.in_order(), vec![&1, &3, &6, &8, &10]);
//!
//! tree.remove(&3);
//! assert_eq!(tree.in_order(), vec![&1, &6, &8, &10]);
//! ```

use std::cmp::Ordering;

/// An optional, owned child node. `None` means "no child here".
type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    value: T,
    left: Link<T>,
    right: Link<T>,
}

impl<T> Node<T> {
    fn new(value: T) -> Self {
        Node {
            value,
            left: None,
            right: None,
        }
    }
}

/// A binary search tree holding unique values (like a set).
#[derive(Debug)]
pub struct BinarySearchTree<T: Ord> {
    root: Link<T>,
    len: usize,
}

impl<T: Ord> BinarySearchTree<T> {
    /// Creates an empty tree.
    pub fn new() -> Self {
        BinarySearchTree { root: None, len: 0 }
    }

    /// Inserts `value`. Returns `false` if it was already in the tree.
    ///
    /// We walk down from the root, going left or right at each node, until we
    /// reach an empty spot. That spot is exactly where the value belongs.
    pub fn insert(&mut self, value: T) -> bool {
        // `current` is a mutable reference to a *link* (not to a node). That
        // lets us overwrite the link itself when we find the empty spot.
        let mut current = &mut self.root;

        while let Some(node) = current {
            current = match value.cmp(&node.value) {
                Ordering::Less => &mut node.left,
                Ordering::Greater => &mut node.right,
                Ordering::Equal => return false, // no duplicates
            };
        }

        *current = Some(Box::new(Node::new(value)));
        self.len += 1;
        true
    }

    /// Returns `true` if the tree contains `value`.
    pub fn contains(&self, value: &T) -> bool {
        let mut current = &self.root;

        while let Some(node) = current {
            current = match value.cmp(&node.value) {
                Ordering::Less => &node.left,
                Ordering::Greater => &node.right,
                Ordering::Equal => return true,
            };
        }

        false
    }

    /// Returns the smallest value: keep going left until you can't.
    pub fn min(&self) -> Option<&T> {
        let mut node = self.root.as_ref()?;
        while let Some(left) = &node.left {
            node = left;
        }
        Some(&node.value)
    }

    /// Returns the largest value: keep going right until you can't.
    pub fn max(&self) -> Option<&T> {
        let mut node = self.root.as_ref()?;
        while let Some(right) = &node.right {
            node = right;
        }
        Some(&node.value)
    }

    /// Removes `value`. Returns `false` if it wasn't in the tree.
    ///
    /// Removing is the trickiest operation, because the tree must still be a
    /// valid BST afterwards.
    ///
    /// Once we find the node, there are three cases:
    ///
    /// ```text
    ///   1. No children: just delete it.
    ///
    ///          5                5
    ///         / \      ──►     /
    ///        3   8            3          (remove 8)
    ///
    ///   2. One child: the child takes the node's place.
    ///
    ///          5                5
    ///         / \      ──►     / \
    ///        3   8            3   9      (remove 8)
    ///             \
    ///              9
    ///
    ///   3. Two children: replace the value with its *in-order successor*, the
    ///      smallest value in the right subtree, then remove that successor.
    ///      The successor is bigger than everything on the left and smaller than
    ///      everything else on the right, so the BST property still holds.
    ///
    ///          5                6
    ///         / \      ──►     / \
    ///        3   8            3   8      (remove 5; successor is 6)
    ///           /
    ///          6
    /// ```
    pub fn remove(&mut self, value: &T) -> bool {
        let removed = remove_from(&mut self.root, value);
        if removed {
            self.len -= 1;
        }
        removed
    }

    /// Returns all values in ascending order (an in-order traversal).
    pub fn in_order(&self) -> Vec<&T> {
        let mut result = Vec::with_capacity(self.len);
        in_order_walk(&self.root, &mut result);
        result
    }

    /// Returns the number of levels in the tree (0 for an empty tree).
    ///
    /// This is the number that decides how fast the tree is.
    pub fn height(&self) -> usize {
        height_of(&self.root)
    }

    /// Returns the number of values in the tree.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the tree has no values.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }
}

impl<T: Ord> Default for BinarySearchTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Drop for BinarySearchTree<T> {
    fn drop(&mut self) {
        // Rotate left children upward, then detach one childless node at a
        // time. No recursive drop and no allocation during cleanup.
        while let Some(mut node) = self.root.take() {
            if let Some(mut left) = node.left.take() {
                node.left = left.right.take();
                left.right = Some(node);
                self.root = Some(left);
            } else {
                self.root = node.right.take();
            }
        }
    }
}

fn in_order_walk<'a, T>(link: &'a Link<T>, result: &mut Vec<&'a T>) {
    let mut stack = Vec::new();
    let mut current = link.as_deref();
    loop {
        while let Some(node) = current {
            stack.push(node);
            current = node.left.as_deref();
        }
        let Some(node) = stack.pop() else { break };
        result.push(&node.value);
        current = node.right.as_deref();
    }
}

fn height_of<T>(link: &Link<T>) -> usize {
    let Some(root) = link.as_deref() else {
        return 0;
    };
    let mut stack = vec![(root, 1)];
    let mut height = 0;
    while let Some((node, depth)) = stack.pop() {
        height = height.max(depth);
        if let Some(left) = node.left.as_deref() {
            stack.push((left, depth + 1));
        }
        if let Some(right) = node.right.as_deref() {
            stack.push((right, depth + 1));
        }
    }
    height
}

fn remove_from<T: Ord>(link: &mut Link<T>, value: &T) -> bool {
    let mut current = link;
    loop {
        match current.as_ref().map(|node| value.cmp(&node.value)) {
            None => return false,
            Some(Ordering::Less) => current = &mut current.as_mut().unwrap().left,
            Some(Ordering::Greater) => current = &mut current.as_mut().unwrap().right,
            Some(Ordering::Equal) => {
                let mut node = current.take().unwrap();
                *current = match (node.left.take(), node.right.take()) {
                    (None, None) => None,
                    (Some(child), None) | (None, Some(child)) => Some(child),
                    (Some(left), Some(right)) => {
                        node.left = Some(left);
                        node.right = Some(right);
                        node.value = take_min(&mut node.right);
                        Some(node)
                    }
                };
                return true;
            }
        }
    }
}

fn take_min<T>(link: &mut Link<T>) -> T {
    let mut current = link;
    while current.as_ref().expect("nonempty subtree").left.is_some() {
        current = &mut current.as_mut().unwrap().left;
    }
    let node = current.take().unwrap();
    *current = node.right;
    node.value
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tree_from(values: &[i32]) -> BinarySearchTree<i32> {
        let mut tree = BinarySearchTree::new();
        for &value in values {
            tree.insert(value);
        }
        tree
    }

    fn sorted(tree: &BinarySearchTree<i32>) -> Vec<i32> {
        tree.in_order().into_iter().copied().collect()
    }

    // The tree from the module docs.
    const EXAMPLE: [i32; 9] = [8, 3, 10, 1, 6, 14, 4, 7, 13];

    #[test]
    fn empty_tree() {
        let tree: BinarySearchTree<i32> = BinarySearchTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.min(), None);
        assert_eq!(tree.max(), None);
        assert!(!tree.contains(&1));
    }

    #[test]
    fn insert_and_contains() {
        let tree = tree_from(&EXAMPLE);
        assert_eq!(tree.len(), 9);
        for value in EXAMPLE {
            assert!(tree.contains(&value));
        }
        assert!(!tree.contains(&5));
    }

    #[test]
    fn rejects_duplicates() {
        let mut tree = tree_from(&[2, 1]);
        assert!(!tree.insert(2));
        assert_eq!(tree.len(), 2);
    }

    #[test]
    fn in_order_is_sorted() {
        let tree = tree_from(&EXAMPLE);
        assert_eq!(sorted(&tree), vec![1, 3, 4, 6, 7, 8, 10, 13, 14]);
        assert_eq!(tree.min(), Some(&1));
        assert_eq!(tree.max(), Some(&14));
    }

    #[test]
    fn remove_leaf() {
        let mut tree = tree_from(&EXAMPLE);
        assert!(tree.remove(&13));
        assert_eq!(sorted(&tree), vec![1, 3, 4, 6, 7, 8, 10, 14]);
    }

    #[test]
    fn remove_node_with_one_child() {
        let mut tree = tree_from(&EXAMPLE);
        assert!(tree.remove(&10)); // 10 has only a right child (14)
        assert_eq!(sorted(&tree), vec![1, 3, 4, 6, 7, 8, 13, 14]);
    }

    #[test]
    fn remove_node_with_two_children() {
        let mut tree = tree_from(&EXAMPLE);
        assert!(tree.remove(&3)); // 3 has children 1 and 6
        assert_eq!(sorted(&tree), vec![1, 4, 6, 7, 8, 10, 13, 14]);
        assert!(tree.remove(&8)); // the root
        assert_eq!(sorted(&tree), vec![1, 4, 6, 7, 10, 13, 14]);
        assert_eq!(tree.len(), 7);
    }

    #[test]
    fn remove_missing_value() {
        let mut tree = tree_from(&EXAMPLE);
        assert!(!tree.remove(&5));
        assert_eq!(tree.len(), 9);
    }

    #[test]
    fn remove_everything() {
        let mut tree = tree_from(&EXAMPLE);
        for value in EXAMPLE {
            assert!(tree.remove(&value));
        }
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }

    #[test]
    fn sorted_input_makes_a_tall_tree() {
        let balanced = tree_from(&[4, 2, 6, 1, 3, 5, 7]);
        let degenerate = tree_from(&[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(balanced.height(), 3);
        assert_eq!(degenerate.height(), 7);
    }
    #[test]
    fn deep_tree_operations_and_drop_use_bounded_call_stack() {
        std::thread::Builder::new()
            .stack_size(128 * 1024)
            .spawn(|| {
                let mut root = None;
                for value in (0..20_000).rev() {
                    root = Some(Box::new(Node {
                        value,
                        left: None,
                        right: root,
                    }));
                }
                let mut tree = BinarySearchTree { root, len: 20_000 };
                assert_eq!(tree.height(), 20_000);
                assert_eq!(tree.in_order().len(), 20_000);
                assert!(tree.remove(&19_999));
                assert_eq!(tree.height(), 19_999);
                drop(tree);
            })
            .unwrap()
            .join()
            .unwrap();
    }
}
