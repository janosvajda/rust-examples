<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Binary search tree

A tree where everything on the left of a node is smaller and everything on the right is bigger. Covers insert, search, min/max, in-order traversal, the three cases of removal, and why unbalanced trees are slow.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `insert` | O(log n) balanced, O(n) worst |
| `contains` | O(log n) balanced, O(n) worst |
| `remove` | O(log n) balanced, O(n) worst |
| `in_order` | O(n) |

## Rust concepts you'll see

- Walking a tree with a `&mut` cursor to insert in place
- Recursion over `Option<Box<Node<T>>>`
- `let ... else` and `std::cmp::Ordering`

## A tree can be very tall

Inserting values that are already sorted turns this unbalanced tree into a long chain, and searching it becomes O(n), like a list. (The [AVL tree](../avl-tree/) fixes that.) A chain of a million nodes would overflow the call stack if every operation used recursion, so walking, measuring and dropping the tree use loops instead.

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p binary-search-tree`.
