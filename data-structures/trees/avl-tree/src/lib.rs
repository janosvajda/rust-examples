//! # AVL tree (self-balancing binary search tree)
//!
//! The `trees/binary-search-tree` example ends with a problem: insert values
//! in sorted order and the tree degenerates into a linked list, so every
//! operation becomes O(n). An **AVL tree** (named after its inventors,
//! Adelson-Velsky and Landis, 1962) fixes this by **rebalancing itself** after
//! every insert and remove.
//!
//! ## The balance rule
//!
//! For every node, the heights of its left and right subtrees may differ by
//! **at most 1**. That difference is the node's **balance factor**:
//!
//! ```text
//!   balance factor = height(left) − height(right)      allowed: −1, 0, +1
//! ```
//!
//! This rule keeps the height below about 1.44 × log₂(n), so search, insert
//! and remove are always **O(log n)**, whatever order the values arrive in.
//!
//! ## Rotations
//!
//! When an insert or remove pushes a balance factor to +2 or −2, the tree fixes
//! it with a **rotation**: a small local rearrangement that keeps the BST order
//! but changes which node is on top.
//!
//! ```text
//!   Right rotation (the left side is too tall):
//!
//!           z                      y
//!          / \                   /   \
//!         y   T4                x     z
//!        / \        ──►        / \   / \
//!       x   T3               T1  T2 T3  T4
//!      / \
//!    T1   T2
//!
//!   In-order before and after: T1 x T2 y T3 z T4. The order is unchanged,
//!   but the height went down by one.
//! ```
//!
//! A **left rotation** is the mirror image. There are four cases, named after
//! the path from the unbalanced node to the newly added one:
//!
//! | Case        | Shape                              | Fix                                   |
//! |-------------|------------------------------------|---------------------------------------|
//! | Left-Left   | left child is left-heavy (or even) | rotate right                          |
//! | Right-Right | right child is right-heavy (or even) | rotate left                         |
//! | Left-Right  | left child is right-heavy          | rotate the left child left, then rotate right |
//! | Right-Left  | right child is left-heavy          | rotate the right child right, then rotate left |
//!
//! ```text
//!   Left-Right example, insert 3, 1, 2:
//!
//!       3              3              2
//!      /              /              / \
//!     1      ──►     2      ──►     1   3
//!      \            /
//!       2          1
//!            rotate 1 left   rotate 3 right
//! ```
//!
//! ## How the code is written
//!
//! The binary search tree example edits the tree in place through `&mut`
//! references. Rotations replace the root of a subtree, which is awkward to do
//! through a reference. So here every recursive function **takes ownership**
//! of a subtree and **returns the new root** of that subtree:
//!
//! ```text
//! fn insert(node: Link<T>, value: T) -> (Box<Node<T>>, bool)
//! ```
//!
//! The caller stores whatever comes back, so a rotation anywhere below is
//! picked up automatically.
//!
//! ## Complexity
//!
//! | Operation  | Time         |
//! |------------|--------------|
//! | `insert`   | O(log n)     |
//! | `contains` | O(log n)     |
//! | `remove`   | O(log n)     |
//! | `in_order` | O(n)         |
//!
//! Rust's `BTreeMap` uses a different self-balancing tree (a B-tree), which is
//! faster on modern CPUs because each node holds many values in a row.
//!
//! ## Example
//!
//! ```
//! use avl_tree::AvlTree;
//!
//! let mut tree = AvlTree::new();
//! for value in 1..=1000 {
//!     tree.insert(value); // sorted input: the worst case for a plain BST
//! }
//!
//! assert_eq!(tree.height(), 10);    // a plain BST would have height 1000
//! assert!(tree.contains(&500));
//! ```

use std::cmp::Ordering;
use std::fmt::Display;

type Link<T> = Option<Box<Node<T>>>;

#[derive(Debug)]
struct Node<T> {
    value: T,
    // Height of the subtree rooted here: 1 for a leaf. Storing it means we
    // never have to walk a whole subtree to compute it.
    height: usize,
    left: Link<T>,
    right: Link<T>,
}

impl<T> Node<T> {
    fn leaf(value: T) -> Box<Self> {
        Box::new(Node {
            value,
            height: 1,
            left: None,
            right: None,
        })
    }
}

/// A self-balancing binary search tree holding unique values (like a set).
#[derive(Debug)]
pub struct AvlTree<T: Ord> {
    root: Link<T>,
    len: usize,
}

impl<T: Ord> Default for AvlTree<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> AvlTree<T> {
    /// Creates an empty tree.
    pub fn new() -> Self {
        AvlTree { root: None, len: 0 }
    }

    /// Inserts `value`. Returns `false` if it was already in the tree.
    pub fn insert(&mut self, value: T) -> bool {
        // Take the root out, let `insert` return the (possibly new) root, and
        // put it back.
        let (new_root, inserted) = insert(self.root.take(), value);
        self.root = Some(new_root);
        if inserted {
            self.len += 1;
        }
        inserted
    }

    /// Removes `value`. Returns `false` if it wasn't in the tree.
    pub fn remove(&mut self, value: &T) -> bool {
        let (new_root, removed) = remove(self.root.take(), value);
        self.root = new_root;
        if removed {
            self.len -= 1;
        }
        removed
    }

    /// Returns `true` if the tree contains `value`.
    /// Exactly the same as in a plain BST: balancing only changes the shape.
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

    /// Returns all values in ascending order.
    pub fn in_order(&self) -> Vec<&T> {
        fn walk<'a, T>(link: &'a Link<T>, out: &mut Vec<&'a T>) {
            if let Some(node) = link {
                walk(&node.left, out);
                out.push(&node.value);
                walk(&node.right, out);
            }
        }
        let mut out = Vec::with_capacity(self.len);
        walk(&self.root, &mut out);
        out
    }

    /// Returns the height of the tree (0 when empty). **O(1)**, because each
    /// node stores its own height.
    pub fn height(&self) -> usize {
        height(&self.root)
    }

    /// Returns the number of values.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the tree is empty.
    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    /// Draws the tree sideways: the root is on the left, the right subtree
    /// above it and the left subtree below. Tilt your head left to read it.
    ///
    /// ```
    /// use avl_tree::AvlTree;
    ///
    /// let mut tree = AvlTree::new();
    /// for value in [1, 2, 3] {
    ///     tree.insert(value);
    /// }
    /// assert_eq!(tree.draw(), "    3\n2\n    1\n");
    /// ```
    pub fn draw(&self) -> String
    where
        T: Display,
    {
        fn draw_node<T: Display>(link: &Link<T>, depth: usize, out: &mut String) {
            if let Some(node) = link {
                draw_node(&node.right, depth + 1, out);
                out.push_str(&format!("{}{}\n", "    ".repeat(depth), node.value));
                draw_node(&node.left, depth + 1, out);
            }
        }
        let mut out = String::new();
        draw_node(&self.root, 0, &mut out);
        out
    }
}

fn height<T>(link: &Link<T>) -> usize {
    link.as_ref().map_or(0, |node| node.height)
}

/// Recomputes a node's height from its children's stored heights.
fn update_height<T>(node: &mut Node<T>) {
    node.height = 1 + height(&node.left).max(height(&node.right));
}

/// Positive: left side taller. Negative: right side taller.
fn balance_factor<T>(node: &Node<T>) -> isize {
    height(&node.left) as isize - height(&node.right) as isize
}

/// Rotates `z` to the right and returns the new subtree root (its left child `y`).
/// See the diagram in the module docs.
fn rotate_right<T>(mut z: Box<Node<T>>) -> Box<Node<T>> {
    let mut y = z.left.take().expect("rotate_right needs a left child");
    z.left = y.right.take(); // T3 moves across to z
    update_height(&mut z); // z is now lower, so update it first
    y.right = Some(z);
    update_height(&mut y);
    y
}

/// Mirror image of [`rotate_right`].
fn rotate_left<T>(mut z: Box<Node<T>>) -> Box<Node<T>> {
    let mut y = z.right.take().expect("rotate_left needs a right child");
    z.right = y.left.take();
    update_height(&mut z);
    y.left = Some(z);
    update_height(&mut y);
    y
}

/// Updates `node`'s height and, if it's out of balance, applies one of the
/// four rotation cases. Returns the new root of this subtree.
fn rebalance<T>(mut node: Box<Node<T>>) -> Box<Node<T>> {
    update_height(&mut node);
    let balance = balance_factor(&node);

    if balance > 1 {
        // Left side too tall. If the left child leans right, it's the
        // Left-Right case: straighten it into Left-Left first.
        let left = node
            .left
            .take()
            .expect("balance > 1 means there is a left child");
        node.left = Some(if balance_factor(&left) < 0 {
            rotate_left(left)
        } else {
            left
        });
        return rotate_right(node);
    }

    if balance < -1 {
        // Mirror image: Right-Left becomes Right-Right, then rotate left.
        let right = node
            .right
            .take()
            .expect("balance < -1 means there is a right child");
        node.right = Some(if balance_factor(&right) > 0 {
            rotate_right(right)
        } else {
            right
        });
        return rotate_left(node);
    }

    node
}

/// Inserts `value` into the subtree and returns its new root, plus whether
/// anything was inserted.
fn insert<T: Ord>(link: Link<T>, value: T) -> (Box<Node<T>>, bool) {
    let Some(mut node) = link else {
        return (Node::leaf(value), true);
    };

    let inserted = match value.cmp(&node.value) {
        Ordering::Less => {
            let (child, inserted) = insert(node.left.take(), value);
            node.left = Some(child);
            inserted
        }
        Ordering::Greater => {
            let (child, inserted) = insert(node.right.take(), value);
            node.right = Some(child);
            inserted
        }
        Ordering::Equal => return (node, false), // duplicate: nothing changed
    };

    // On the way back up from the recursion, every node on the path gets a
    // chance to rebalance.
    (rebalance(node), inserted)
}

/// Removes `value` from the subtree and returns its new root (possibly empty),
/// plus whether anything was removed.
///
/// The three cases are the same as in the plain BST (no children, one child,
/// two children). The only addition is the `rebalance` on the way back up.
fn remove<T: Ord>(link: Link<T>, value: &T) -> (Link<T>, bool) {
    let Some(mut node) = link else {
        return (None, false);
    };

    match value.cmp(&node.value) {
        Ordering::Less => {
            let (child, removed) = remove(node.left.take(), value);
            node.left = child;
            (Some(rebalance(node)), removed)
        }
        Ordering::Greater => {
            let (child, removed) = remove(node.right.take(), value);
            node.right = child;
            (Some(rebalance(node)), removed)
        }
        Ordering::Equal => {
            let replacement = match (node.left.take(), node.right.take()) {
                (None, None) => None,
                (Some(child), None) | (None, Some(child)) => Some(child),
                (Some(left), Some(right)) => {
                    // Replace this value with the smallest value on the right.
                    let (new_right, successor) = take_min(right);
                    node.value = successor;
                    node.left = Some(left);
                    node.right = new_right;
                    Some(rebalance(node))
                }
            };
            (replacement, true)
        }
    }
}

/// Removes the smallest node from a subtree. Returns the subtree's new root
/// and the removed value.
fn take_min<T>(mut node: Box<Node<T>>) -> (Link<T>, T) {
    match node.left.take() {
        Some(left) => {
            let (new_left, min) = take_min(left);
            node.left = new_left;
            (Some(rebalance(node)), min)
        }
        None => {
            // This is the leftmost node. Its right child (if any) takes its place.
            let node = *node; // move out of the Box
            (node.right, node.value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Checks every AVL invariant: BST order, correct stored heights, and
    /// balance factors within −1..=1. Returns the subtree's height.
    fn check<T: Ord>(link: &Link<T>, lower: Option<&T>, upper: Option<&T>) -> usize {
        let Some(node) = link else { return 0 };
        assert!(
            lower.is_none_or(|low| &node.value > low),
            "BST order broken"
        );
        assert!(
            upper.is_none_or(|high| &node.value < high),
            "BST order broken"
        );

        let left = check(&node.left, lower, Some(&node.value));
        let right = check(&node.right, Some(&node.value), upper);
        assert!(left.abs_diff(right) <= 1, "node out of balance");
        assert_eq!(node.height, 1 + left.max(right), "stored height is wrong");
        node.height
    }

    fn assert_valid<T: Ord>(tree: &AvlTree<T>) {
        check(&tree.root, None, None);
        assert_eq!(tree.in_order().len(), tree.len());
    }

    #[test]
    fn empty_tree() {
        let tree: AvlTree<i32> = AvlTree::new();
        assert!(tree.is_empty());
        assert_eq!(tree.height(), 0);
        assert!(!tree.contains(&1));
    }

    #[test]
    fn each_rotation_case_balances_three_nodes() {
        // Left-Left, Right-Right, Left-Right, Right-Left
        for order in [[3, 2, 1], [1, 2, 3], [3, 1, 2], [1, 3, 2]] {
            let mut tree = AvlTree::new();
            for value in order {
                tree.insert(value);
            }
            assert_valid(&tree);
            assert_eq!(tree.height(), 2, "order {order:?}");
            assert_eq!(tree.root.as_ref().unwrap().value, 2, "order {order:?}");
        }
    }

    #[test]
    fn sorted_inserts_stay_balanced() {
        let mut tree = AvlTree::new();
        for value in 0..1000 {
            tree.insert(value);
            assert_valid(&tree);
        }
        assert_eq!(tree.height(), 10);
    }

    #[test]
    fn rejects_duplicates() {
        let mut tree = AvlTree::new();
        assert!(tree.insert(1));
        assert!(!tree.insert(1));
        assert_eq!(tree.len(), 1);
    }

    #[test]
    fn remove_keeps_tree_balanced() {
        let mut tree = AvlTree::new();
        for value in 0..200 {
            tree.insert(value);
        }
        // Remove every other value, plus some that aren't there.
        for value in (0..200).step_by(2) {
            assert!(tree.remove(&value));
            assert!(!tree.remove(&value));
            assert_valid(&tree);
        }
        let remaining: Vec<i32> = tree.in_order().into_iter().copied().collect();
        assert_eq!(remaining, (1..200).step_by(2).collect::<Vec<_>>());
    }

    #[test]
    fn remove_everything() {
        let mut tree = AvlTree::new();
        let values: Vec<i32> = (0..100).map(|i| (i * 37) % 100).collect();
        for &value in &values {
            tree.insert(value);
        }
        for value in values {
            assert!(tree.remove(&value));
            assert_valid(&tree);
        }
        assert!(tree.is_empty());
        assert_eq!(tree.len(), 0);
    }
}
