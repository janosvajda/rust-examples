# Doubly linked list

A list where every node links to both its neighbours, so you can add and remove at either end in O(1) and walk it in both directions. Built with `Rc`, `RefCell` and `Weak`, and shows how a `Weak` back-pointer prevents a memory leak.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `push_front / push_back` | O(1) |
| `pop_front / pop_back` | O(1) |
| `peek_front / peek_back` | O(1) |

## Rust concepts you'll see

- `Rc<T>` for shared ownership
- `RefCell<T>` for mutation through shared references, with borrow rules checked at runtime
- `Weak<T>` to break reference cycles, and a test that proves nothing leaks
- Returning `Ref<T>` guards from `peek`

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p doubly-linked-list`.
