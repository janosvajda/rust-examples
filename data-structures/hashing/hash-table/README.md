# Hash table

A key-value map built from scratch with separate chaining. Shows hashing keys into buckets, handling collisions, the load factor, and rehashing when the table grows.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `insert` | O(1) amortized |
| `get` | O(1) average |
| `remove` | O(1) average |

## Rust concepts you'll see

- The `Hash` and `Eq` traits
- `RandomState::hash_one` and why std uses random hash keys
- `std::mem::replace` and `swap_remove`

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p hash-table`.
