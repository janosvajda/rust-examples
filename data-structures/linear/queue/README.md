# Queue (ring buffer)

A FIFO (first in, first out) queue stored in a growable ring buffer, the same idea as `std::collections::VecDeque`. Shows why `Vec::remove(0)` is slow and how wrapping indexes with `%` avoids it.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `enqueue` | O(1) amortized |
| `dequeue` | O(1) |
| `peek` | O(1) |

## Rust concepts you'll see

- Modular arithmetic for circular indexes
- Using `Option<T>` slots and `take()` to move values out without `unsafe`
- Amortized growth by doubling

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p queue`.
