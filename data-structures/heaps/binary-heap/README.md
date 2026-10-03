<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Binary heap (min-heap)

A complete binary tree stored in a `Vec`, where every parent is smaller than its children. Used as a priority queue, and to sort values with heap sort.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `push` | O(log n) |
| `pop` | O(log n) |
| `peek` | O(1) |
| `from_vec (heapify)` | O(n) |
| `heap_sort` | O(n log n) |

## Rust concepts you'll see

- Storing a tree in an array with index arithmetic
- Trait bounds (`T: Ord`)
- Why building a heap in place is O(n), not O(n log n)

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p binary-heap`.
