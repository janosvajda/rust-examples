<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Dijkstra's shortest path

Finds the cheapest route in a graph with weighted edges, using a min-heap as a priority queue. Demo: fastest driving routes between Hungarian cities.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `dijkstra` | O(V + E log(E + 1)) |
| `shortest_path` | O(V + E log(E + 1)) |

## Rust concepts you'll see

- `std::collections::BinaryHeap` with `Reverse` as a min-heap
- Lazy deletion of outdated heap entries
- Rebuilding a path from `previous` links

## Costs and failures

Each edge costs a `u32`, but a route's total is a `u64`, added with checked arithmetic. So a route costing `u32::MAX` plus one more step costs 4,294,967,296, instead of wrapping around to zero:

```rust
let mut graph = dijkstra::WeightedGraph::new(3);
graph.add_edge(0, 1, u32::MAX);
graph.add_edge(1, 2, 1);
assert_eq!(graph.shortest_path(0, 2)?, Some((4_294_967_296, vec![0, 1, 2])));
```

`shortest_path` returns `Ok(None)` when there's no route, and an error for a node that doesn't exist (or, in theory, a total bigger than a `u64`). `add_edge` with a node that doesn't exist is a programming mistake, so it panics.

Edsger Dijkstra published the algorithm in [1959](https://doi.org/10.1007/BF01386390). Its rule still works today: always settle the cheapest node not yet settled. It only works when no edge has a negative cost.

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p dijkstra`.
