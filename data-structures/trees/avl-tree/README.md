<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# AVL tree

A binary search tree that rebalances itself with rotations after every insert and remove, so it stays O(log n) whatever order the values arrive in. The demo draws the tree as rotations happen and compares its height with the plain BST.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `insert` | O(log n) |
| `contains` | O(log n) |
| `remove` | O(log n) |
| `height` | O(1) |

## Rust concepts you'll see

- Recursive functions that take ownership of a subtree and return its new root
- The four rotation cases (LL, RR, LR, RL)
- A test helper that checks every invariant after each operation
- Using another workspace crate as a dependency (`binary-search-tree`)

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p avl-tree`.
