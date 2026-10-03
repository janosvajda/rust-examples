<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Singly linked list

A **linked list** stores each element in its own *node*, and every node links to the next one. This crate implements a singly linked list with push and pop at the front, in-place reversal and a borrowing iterator. For speed, its nodes live in one `Vec` (an *arena*) instead of being allocated one by one.

## Operations

| Operation | Time | What it does |
|-----------|------|--------------|
| `push_front` | O(1) amortized | Adds a value at the front |
| `pop_front` | O(1) | Removes and returns the front value |
| `peek_front` | O(1) | Looks at the front value |
| `contains` | O(n) | Searches for a value |
| `reverse` | O(n) | Reverses the list in place |
| `iter` | O(n) | Walks the list from front to back |

## How it is stored in memory

### The classic way: one heap allocation per node

The textbook Rust linked list gives every node its own heap allocation:

```rust
struct Node<T> { value: T, next: Option<Box<Node<T>>> }
```

```text
  heap (the allocator decides where each node goes)

     ┌────┬───┐                 ┌────┬───┐        ┌────┬──────┐
     │ 30 │ ●─┼────────────────►│ 20 │ ●─┼───────►│ 10 │ None │
     └────┴───┘                 └────┴───┘        └────┴──────┘
       ▲           other data          other data
      head
```

It's a good way to learn ownership, but it has two costs:
- Every `push_front` calls the memory allocator, and every `pop_front` frees memory. Those calls cost far more than the list operation itself.
- The nodes end up wherever the allocator puts them. In a long-running program they can be scattered all over the heap.

### This crate: an arena

All nodes are stored in **one `Vec`**, and a node refers to the next one by its **index** in that `Vec`:

```text
  LinkedList                     slots: Vec<Slot<T>>  (one contiguous block)
  ┌────────────────┐             ┌──────────────────────────────┐
  │ head = 2       │             │ 0: Used   value 10   next NIL │
  │ free_head = NIL│             │ 1: Used   value 20   next 0   │
  │ len = 3        │             │ 2: Used   value 30   next 1   │
  └────────────────┘             └──────────────────────────────┘

  list order: head ─► [2] 30 ─► [1] 20 ─► [0] 10 ─► NIL
```

- **Each slot** is either `Used` (holds a value and the index of the next node) or `Free` (waiting to be reused).
- **Indexes are `u32`**, half the size of a 64-bit pointer. The special value `NIL` (`u32::MAX`) means "no next node". With a `u64` value, a slot is 16 bytes: a 4-byte used/free tag, a 4-byte index and the 8-byte value. The tests check this.
- **Removed nodes are recycled.** `pop_front` doesn't free memory. It marks the slot `Free` and puts it on a **free list**, a second chain of indexes running through the unused slots. The next `push_front` takes a slot from there before growing the `Vec`:

```text
  after pop_front:  head = 1, free_head = 2

  │ 0: Used   value 10   next NIL │
  │ 1: Used   value 20   next 0   │ ◄── head
  │ 2: Free   next_free NIL       │ ◄── free_head: reused by the next push
```

- **The `Vec` never shrinks** while the list exists. Popped slots stay allocated, ready for reuse. That's the trade-off for never freeing memory one node at a time.

## How the CPU processes it

**Walking a linked list is "pointer chasing".** To find node *n + 1*, the CPU must first finish loading node *n*, because that's where the next link is stored. Each step waits for the previous one, so the CPU can't work ahead the way it can with an array. How long each step takes depends almost entirely on **where the next node is**:

- **Nearby in memory:** the node is probably already in the CPU cache, or the *prefetcher* has spotted the pattern and loaded it early. The step is fast.
- **Somewhere random:** the CPU has to go to main memory, which is roughly 100 times slower than the cache. A list whose nodes are scattered can be many times slower to walk than the same list laid out in order.

**The arena keeps nodes together.** All nodes are in one block. A list built by pushing is laid out in (reverse) memory order, which the prefetcher handles well. With one allocation per node, the layout is up to the allocator. It can be good or bad, and it changes from run to run.

**Each step does a little extra work.** Following an index means computing an address (`start of Vec + index × slot size`), checking the index is in bounds, and checking the slot is `Used`. Following a raw pointer needs none of that. These checks are cheap in practice: their outcome is always the same, so the CPU predicts them and runs them alongside the memory loads.

**Arrays are still faster.** A `Vec` or `VecDeque` stores the values themselves side by side, so the CPU can load many at once and never has to wait for a link. Iterating an array is several times faster than iterating any linked list. Choose a linked list for its operations, not for speed.

## Optimisations in this crate

1. **The arena itself.** Replacing one allocation per node with one shared `Vec` removes the allocator from almost every `push_front` and `pop_front`. Building and modifying the list becomes several times faster.
2. **The free list.** Popped slots are reused immediately, so a list that keeps changing size reuses the same memory instead of growing.
3. **`u32` indexes.** Smaller links make smaller slots, so more of them fit in each cache line.
4. **`with_capacity` and `collect`.** `LinkedList::with_capacity(n)` reserves room for `n` nodes up front. `collect()` does this automatically when the iterator knows its length.
5. **No custom `Drop`.** A chain of `Box`es is dropped recursively, which can overflow the call stack for very long lists, so the classic version needs a hand-written loop. Here dropping the list just drops one `Vec`.
6. **Still 100% safe code.** A wrong index can't corrupt memory: `Vec` indexing is bounds-checked, and a link to a `Free` slot panics.

### What was tried and not kept

- **Storing the links in a separate array** from the values, so following the chain touches less memory. It made no measurable difference, so the simpler layout stayed.
- **Removing the bounds and used/free checks with `unsafe` code.** This was *not* faster, because those checks weren't what the CPU was waiting on. The memory loads were. The code stays safe.

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p linked-list`.
