<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Bloom filter

A probabilistic set: a few bits per item, never a false negative, and a false positive rate you choose. The demo stores 100,000 usernames in 117 KB and measures the real false positive rate.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `insert` | O(k) |
| `might_contain` | O(k) |

## Rust concepts you'll see

- Bit manipulation on a packed `Vec<u64>`
- Deriving k hash functions from two (double hashing)
- `?Sized` bounds so `str` can be passed directly
- Edition 2024 `impl Trait` capture rules and `+ use<T>`

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p bloom-filter`.
