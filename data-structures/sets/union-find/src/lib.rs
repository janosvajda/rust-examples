//! # Union-find (disjoint set union)
//!
//! **Union-find** keeps track of items split into non-overlapping groups. It
//! answers two questions very fast:
//!
//! - **`union(a, b)`**: merge the group containing `a` with the group containing `b`.
//! - **`find(a)`**: which group is `a` in? Two items are in the same group if
//!   `find` returns the same answer for both.
//!
//! Typical uses: "are these two computers on the same network?", "which pixels
//! belong to the same region of an image?", and Kruskal's algorithm for the
//! cheapest way to connect everything (see the demo in `main.rs`).
//!
//! ## How groups are stored: a forest of trees
//!
//! Each item points to a **parent**. Following parents eventually reaches the
//! **root**, an item that is its own parent. The root is the group's
//! representative, and `find` returns it.
//!
//! ```text
//!   groups {0, 1, 2, 3} and {4, 5}:
//!
//!        0          4         parent: [0, 0, 0, 2, 4, 4]
//!       / \         |                  0  1  2  3  4  5   ← item
//!      1   2        5
//!          |
//!          3
//! ```
//!
//! `union(3, 5)` finds both roots (0 and 4) and points one at the other. That's all.
//!
//! ## Two tricks that make it almost O(1)
//!
//! A naive version can build long chains, making `find` O(n). Two simple
//! tricks fix that:
//!
//! 1. **Union by size**: always attach the *smaller* tree under the *larger*
//!    one's root. A tree can then only get taller when it doubles in size, so
//!    its height stays at most log₂(n).
//! 2. **Path compression**: after `find(x)` reaches the root, point every
//!    item on the way **directly at the root**. The next `find` on any of
//!    them takes a single step.
//!
//! ```text
//!   find(3) with path compression:
//!
//!        0                 0
//!        |               / | \
//!        1       ──►    1  2  3
//!        |
//!        2
//!        |
//!        3
//! ```
//!
//! With both tricks, each operation takes **O(α(n))** amortized time. α is the
//! inverse Ackermann function, which grows so slowly that it's at most 4 for
//! any n you could ever store. In practice, constant time.
//!
//! ## Example
//!
//! ```
//! use union_find::UnionFind;
//!
//! let mut groups = UnionFind::new(5);
//! groups.union(0, 1);
//! groups.union(3, 4);
//!
//! assert!(groups.connected(0, 1));
//! assert!(!groups.connected(1, 3));
//! assert_eq!(groups.group_count(), 3); // {0, 1}, {2}, {3, 4}
//! ```

/// Disjoint groups of the items `0..n`.
#[derive(Debug, Clone)]
pub struct UnionFind {
    // parent[i] is the item above i in its tree. A root is its own parent.
    parent: Vec<usize>,
    // size[r] is the number of items in the tree rooted at r.
    // Only meaningful for roots.
    size: Vec<usize>,
    group_count: usize,
}

impl UnionFind {
    /// Creates `n` items, each in its own group.
    pub fn new(n: usize) -> Self {
        UnionFind {
            parent: (0..n).collect(), // everyone is their own root
            size: vec![1; n],
            group_count: n,
        }
    }

    /// Returns the root (representative) of the group containing `item`.
    ///
    /// Takes `&mut self` because path compression rewrites parent pointers
    /// even though the groups themselves don't change.
    ///
    /// # Panics
    /// Panics if `item` is out of range.
    pub fn find(&mut self, item: usize) -> usize {
        // First pass: walk up to the root.
        let mut root = item;
        while self.parent[root] != root {
            root = self.parent[root];
        }

        // Second pass: path compression. Point everything on the path
        // directly at the root.
        let mut current = item;
        while self.parent[current] != root {
            let next = self.parent[current];
            self.parent[current] = root;
            current = next;
        }

        root
    }

    /// Merges the groups containing `a` and `b`.
    /// Returns `false` if they were already in the same group.
    pub fn union(&mut self, a: usize, b: usize) -> bool {
        let root_a = self.find(a);
        let root_b = self.find(b);
        if root_a == root_b {
            return false;
        }

        // Union by size: hang the smaller tree under the larger one's root.
        let (big, small) = if self.size[root_a] >= self.size[root_b] {
            (root_a, root_b)
        } else {
            (root_b, root_a)
        };
        self.parent[small] = big;
        self.size[big] += self.size[small];
        self.group_count -= 1;
        true
    }

    /// Returns `true` if `a` and `b` are in the same group.
    pub fn connected(&mut self, a: usize, b: usize) -> bool {
        self.find(a) == self.find(b)
    }

    /// Returns the number of items in `item`'s group.
    pub fn group_size(&mut self, item: usize) -> usize {
        let root = self.find(item);
        self.size[root]
    }

    /// Returns the number of separate groups.
    pub fn group_count(&self) -> usize {
        self.group_count
    }

    /// Returns the total number of items.
    pub fn len(&self) -> usize {
        self.parent.len()
    }

    /// Returns `true` if there are no items.
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_with_every_item_alone() {
        let mut groups = UnionFind::new(3);
        assert_eq!(groups.group_count(), 3);
        for item in 0..3 {
            assert_eq!(groups.find(item), item);
            assert_eq!(groups.group_size(item), 1);
        }
    }

    #[test]
    fn union_merges_groups() {
        let mut groups = UnionFind::new(6);
        assert!(groups.union(0, 1));
        assert!(groups.union(1, 2));
        assert!(groups.union(4, 5));
        assert!(!groups.union(0, 2)); // already together

        assert!(groups.connected(0, 2));
        assert!(!groups.connected(2, 4));
        assert_eq!(groups.group_size(1), 3);
        assert_eq!(groups.group_count(), 3); // {0,1,2}, {3}, {4,5}
    }

    #[test]
    fn connectivity_is_transitive() {
        let mut groups = UnionFind::new(4);
        groups.union(0, 1);
        groups.union(2, 3);
        assert!(!groups.connected(0, 3));
        groups.union(1, 2);
        assert!(groups.connected(0, 3));
        assert_eq!(groups.group_count(), 1);
    }

    #[test]
    fn path_compression_flattens_the_tree() {
        let mut groups = UnionFind::new(4);
        // Build the chain 3 → 2 → 1 → 0 by hand, which union by size would never do.
        groups.parent = vec![0, 0, 1, 2];
        assert_eq!(groups.find(3), 0);
        assert_eq!(groups.parent, vec![0, 0, 0, 0]);
    }

    #[test]
    fn trees_stay_shallow() {
        let n = 1 << 12;
        let mut groups = UnionFind::new(n);
        for i in 1..n {
            groups.union(i - 1, i);
        }
        // Count how far each item is from its root, without compressing.
        let depth = |groups: &UnionFind, mut item: usize| {
            let mut steps = 0;
            while groups.parent[item] != item {
                item = groups.parent[item];
                steps += 1;
            }
            steps
        };
        let deepest = (0..n).map(|i| depth(&groups, i)).max().unwrap();
        assert!(deepest <= 12, "depth {deepest} is more than log2(n)");
    }
}
