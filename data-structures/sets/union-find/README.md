# Union-find (disjoint set union)

Tracks which items belong to the same group, with nearly constant-time `union` and `find` thanks to union by size and path compression. The demo uses it in Kruskal's algorithm to find the cheapest network that connects every office.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `find` | O(α(n)), effectively O(1) |
| `union` | O(α(n)), effectively O(1) |
| `connected` | O(α(n)), effectively O(1) |

## Rust concepts you'll see

- Representing a forest of trees in a single `Vec` of parent indexes
- Why a `find` that only reads still needs `&mut self` (path compression)
- Kruskal's minimum spanning tree algorithm

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p union-find`.
