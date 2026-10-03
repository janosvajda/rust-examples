<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Topological sort

Orders tasks so each comes after everything it depends on, like Cargo's build order. Uses Kahn's algorithm, and finds the actual cycle with a three-colour depth-first search when no order exists.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `topological_sort` | O(V + E) |
| `find_cycle` | O(V + E) |

## Rust concepts you'll see

- Directed graphs and in-degrees
- Returning `Result` with a custom error type
- Three-colour DFS for cycle detection
- Edition 2024 `if let` chains (`if a && let Some(x) = b`)

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p topological-sort`.
