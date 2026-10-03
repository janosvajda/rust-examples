<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Data structures in Rust

Classic data structures written from scratch, one small crate each. Every
implementation explains itself in its doc comments: what the structure is,
ASCII diagrams of how it works, the cost of each operation, and the Rust
techniques involved. The doc comments include examples that run as tests.

**Optimised examples.** [Stack](linear/stack/) and [Linked list](linear/linked-list/)
go one level deeper. Their READMEs explain how the data is stored in memory,
how the CPU processes it, and how the code was optimised.

In real code, use the standard library (`Vec`, `VecDeque`, `HashMap`,
`BTreeMap`, `BinaryHeap`) or a well-known crate. These examples show how such collections work inside.

## Categories

```
data-structures/
├─ linear/     items in a sequence
│  ├─ stack/                 LIFO, backed by a Vec                  (optimised)
│  ├─ queue/                 FIFO, ring buffer
│  ├─ linked-list/           singly linked list in an arena         (optimised)
│  └─ doubly-linked-list/    links both ways, with Rc, RefCell and Weak
├─ trees/      hierarchical data
│  ├─ binary-search-tree/    ordered set with insert / search / remove
│  ├─ avl-tree/              self-balancing BST with rotations
│  └─ trie/                  prefix tree for strings and autocomplete
├─ heaps/      always know the smallest item
│  └─ binary-heap/           min-heap, priority queue, heap sort
├─ hashing/    constant-time lookup by key
│  ├─ hash-table/            hash map with separate chaining and resizing
│  ├─ lru-cache/             hash map + linked list in an arena
│  └─ hash-map-memory-allocation/   timing allocation of a million HashMap-heavy structs
├─ sets/       membership and grouping
│  ├─ union-find/            disjoint groups, Kruskal's minimum spanning tree
│  └─ bloom-filter/          probabilistic set: tiny, no false negatives
└─ graphs/     nodes connected by edges
   ├─ graph/                 adjacency list, BFS, DFS, connected components
   ├─ dijkstra/              weighted shortest paths with a priority queue
   └─ topological-sort/      dependency order and cycle detection
```

## Suggested learning order

Later examples build on ideas from earlier ones.

1. [Stack](linear/stack/): generics, `Option`, wrapping a `Vec`.
2. [Queue](linear/queue/): ring buffers and amortized growth.
3. [Linked list](linear/linked-list/): a chain of nodes, the arena pattern, iterators, and why memory layout decides speed.
4. [Doubly linked list](linear/doubly-linked-list/): when one owner isn't enough. `Rc`, `RefCell`, `Weak` and reference cycles.
5. [Binary search tree](trees/binary-search-tree/): the linked list idea with two children instead of one, plus recursion.
6. [AVL tree](trees/avl-tree/): fixes the BST's worst case with rotations.
7. [Trie](trees/trie/): a tree where each node can have many children.
8. [Binary heap](heaps/binary-heap/): a tree stored in a plain `Vec`.
9. [Hash table](hashing/hash-table/): hashing, collisions and resizing.
10. [LRU cache](hashing/lru-cache/): a hash map and a linked list working together. Compare its index-based list with step 4.
11. [Union-find](sets/union-find/): trees stored as parent indexes in a `Vec`, like the heap in step 8.
12. [Bloom filter](sets/bloom-filter/): hashing from step 9, traded for memory.
13. [Graph](graphs/graph/): BFS uses the queue idea from step 2, DFS the stack from step 1.
14. [Dijkstra](graphs/dijkstra/): graphs plus the heap from step 8.
15. [Topological sort](graphs/topological-sort/): directed graphs, and DFS used to find cycles.

[hash-map-memory-allocation](hashing/hash-map-memory-allocation/) is a
separate benchmark that times allocating and freeing a million structs that
each contain a `HashMap`.

## Complexity cheat sheet

| Structure          | Insert            | Remove            | Search / lookup   | Notes |
|--------------------|-------------------|-------------------|-------------------|-------|
| Stack              | O(1)\*            | O(1)              | top only: O(1)    | LIFO |
| Queue (ring buffer)| O(1)\*            | O(1)              | front only: O(1)  | FIFO |
| Linked list        | front: O(1)       | front: O(1)       | O(n)              | no random access |
| Doubly linked list | both ends: O(1)   | both ends: O(1)   | O(n)              | walk in both directions |
| Binary search tree | O(log n) / O(n)   | O(log n) / O(n)   | O(log n) / O(n)   | balanced / worst case |
| AVL tree           | O(log n)          | O(log n)          | O(log n)          | always balanced |
| Trie               | O(k)              | n/a               | O(k)              | k = word length |
| Binary heap        | O(log n)          | min: O(log n)     | min: O(1)         | build from a Vec in O(n) |
| Hash table         | O(1)\*            | O(1) avg          | O(1) avg          | O(n) worst case |
| LRU cache          | O(1) avg          | evicts in O(1)    | O(1) avg          | fixed capacity |
| Union-find         | union: ≈ O(1)     | n/a               | find: ≈ O(1)      | O(α(n)) amortized |
| Bloom filter       | O(k)              | not possible      | O(k)              | k = hash count; false positives possible |
| Graph BFS / DFS    | edge: O(1)        | n/a               | O(V + E)          | V nodes, E edges |
| Dijkstra           | n/a               | n/a               | O((V + E) log V)  | non-negative weights |
| Topological sort   | n/a               | n/a               | O(V + E)          | directed graphs only |

\* amortized: an occasional resize costs O(n), but averaged over many inserts it is O(1).

## Running the examples

Every crate is part of the repository's Cargo workspace. From the repository root:

```bash
cargo run  -p trie        # run one demo
cargo test -p trie        # its unit tests and doc examples
cargo doc  -p trie --open # read its documentation in the browser
```

Or `cd` into a crate's directory and use `cargo run`, `cargo test` and
`cargo doc --open` without `-p`.
