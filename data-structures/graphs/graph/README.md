<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Graph with BFS and DFS

An undirected graph stored as an adjacency list. Covers breadth-first search, depth-first search (iterative and recursive), shortest paths by number of edges, and connected components.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `add_edge` | O(1) |
| `bfs / dfs` | O(V + E) |
| `shortest_path` | O(V + E) |
| `connected_components` | O(V + E) |

## Rust concepts you'll see

- Adjacency lists vs adjacency matrices
- Queue (BFS) vs stack (DFS): one small change, very different behaviour
- Why recursion can overflow and an explicit stack doesn't

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p graph`.
