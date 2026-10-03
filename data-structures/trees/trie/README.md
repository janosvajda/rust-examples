<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Trie (prefix tree)

Stores strings by sharing common prefixes. Answers "is this a word?", "does any word start with this?" and autocomplete queries in time proportional to the word length.

The full explanation, with diagrams, lives in the doc comments in [`src/lib.rs`](src/lib.rs). The easiest way to read it is as generated documentation:

```bash
cargo doc --open
```

## Operations

| Operation | Time |
|-----------|------|
| `insert` | O(k) |
| `contains` | O(k) |
| `starts_with` | O(k) |
| `words_with_prefix` | O(k + matches) |

## Rust concepts you'll see

- `BTreeMap::entry(..).or_default()`
- Depth-first collection with a reused `String` buffer
- Working with `char`s, including non-ASCII text

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p trie`.
