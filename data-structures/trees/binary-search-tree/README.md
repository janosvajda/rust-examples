<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

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

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p binary-search-tree`.
