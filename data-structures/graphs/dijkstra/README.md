# Dijkstra's shortest path

Finds the cheapest route in a graph with weighted edges, using a min-heap as a priority queue. Demo: fastest driving routes between Hungarian cities.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `dijkstra` | O((V + E) log V) |
| `shortest_path` | O((V + E) log V) |

## Rust concepts you'll see

- `std::collections::BinaryHeap` with `Reverse` as a min-heap
- Lazy deletion of outdated heap entries
- Rebuilding a path from `previous` links

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p dijkstra`.
