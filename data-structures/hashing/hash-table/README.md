<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

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

## Updating is different from inserting

Inserting a key that's already there **replaces** its value: the number of entries stays the same, so it never makes the table grow. Only a new key can.

The "O(1) average" assumes keys are quick to hash and compare. Long strings take longer, because every byte is hashed. The hash keys are random, like the standard `HashMap`'s, so an attacker can't easily choose keys that all land in the same bucket.

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p hash-table`.
