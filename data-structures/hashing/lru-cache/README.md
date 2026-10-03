# LRU cache

A fixed-size cache that evicts the least recently used entry. Combines a `HashMap` (find by key) with a doubly linked list (usage order), both O(1). The list lives in a `Vec` and is linked by indexes (an arena), the usual Rust alternative to `Rc<RefCell<…>>`.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `get` | O(1) average |
| `put` | O(1) average |
| `peek` | O(1) average |

## Rust concepts you'll see

- The arena pattern: nodes in a `Vec`, links as `usize` indexes
- Combining two data structures to get O(1) for both lookup and ordering
- Testing against a simple reference model
- `#[should_panic]` tests

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p lru-cache`.
