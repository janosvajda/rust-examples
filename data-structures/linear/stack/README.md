<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Stack

A **stack** is a LIFO (last in, first out) collection: the last item pushed is the first one popped, like a stack of plates. This crate implements one on top of a `Vec`, plus a bracket-balancing checker that shows a classic use of a stack.

## Operations

| Operation | Time | What it does |
|-----------|------|--------------|
| `push` | O(1) amortized | Puts an item on top |
| `pop` | O(1) | Removes and returns the top item |
| `peek` | O(1) | Looks at the top item without removing it |
| `extend` | O(n) | Pushes many items at once |
| `with_capacity` | O(1) | Creates a stack with room reserved in advance |

## How it is stored in memory

A `Stack<T>` is a thin wrapper around a `Vec<T>`. It has two parts:

```text
  the Stack itself (3 machine words)            heap buffer
  ┌──────────┬──────────┬──────────┐           ┌────┬────┬────┬────┬────┬────┬────┬────┐
  │ pointer ─┼──────────┼──────────┼──────────►│ 10 │ 20 │ 30 │ 40 │ 50 │    │    │    │
  │ capacity │          │          │           └────┴────┴────┴────┴────┴────┴────┴────┘
  │ = 8      │          │ len = 5  │            [0]  [1]  [2]  [3]  [4]   reserved, unused
  └──────────┴──────────┴──────────┘                            ▲
                                                               top
```

- **The header** holds a pointer to the buffer, the capacity (how many items fit), and the length (how many are stored). It's three machine words, 24 bytes on a 64-bit CPU, whatever `T` is. It lives wherever the `Stack` variable lives, usually on the call stack.
- **The items** live in one **contiguous** block on the heap, side by side, with no gaps or pointers between them.
- **The top of the stack** is at `pointer + (len − 1) × size_of::<T>()`. Finding it is a single calculation, with no searching.
- **An empty stack allocates nothing.** Memory is only requested on the first `push`.
- **`Option<Stack<T>>` takes no extra space.** A capacity can never be larger than `isize::MAX`, so Rust uses those impossible values to represent `None`. This trick is called a *niche*.

## How the CPU processes it

**Memory is read in cache lines.** A CPU never fetches a single value from main memory. It fetches a whole *cache line* (typically 64 bytes, 128 on some CPUs) into its fast on-chip caches. Reading from the cache is roughly 100 times faster than going to main memory.

Because a stack's items are contiguous, one cache line holds several of them: eight `u64`s in a 64-byte line. And because `push` and `pop` always work at the same end, the line holding the top is almost always already in the cache. The CPU's *prefetcher* notices sequential access patterns and loads the next lines before they're needed.

**`push`** compares the length with the capacity, writes the item at the end, and increments the length. Only when the buffer is full does it take the slow path: allocate a buffer twice as big and copy everything across. Doubling means this happens rarely (15 times for 100,000 pushes), so the average cost per push stays constant.

**`pop`** checks whether the length is zero, decrements it, and reads the item at the new end. It needs no bounds check, because an index of `len − 1` is always valid when `len > 0`. The result, an `Option<T>`, is returned directly in CPU registers rather than through memory.

**Branch prediction.** Both operations contain an `if` ("is it full?", "is it empty?"). The answer is almost always the same, so the CPU's branch predictor guesses it correctly and those checks are nearly free.

Compare this with the [linked list](../linked-list/), where each element can be in a different place in memory and every step can be a cache miss.

## Optimisations in this crate

### `with_capacity`: avoid reallocations

If you know roughly how many items are coming, `Stack::with_capacity(n)` allocates once up front. The first `n` pushes then never reallocate or copy. How much this saves depends on the memory allocator: some can grow a block in place cheaply, others must copy it.

### `extend`: push many items at once

Pushing in a loop checks "is it full?" on every iteration. And because a reallocation *could* happen at any time, the compiler must keep the stack's length and pointer up to date in memory after every push.

`stack.extend(items)` does better when the number of items is known in advance (a range, a slice, a `Vec`). It reserves space **once**, then writes all the items in a loop that contains no checks and no possible reallocation. A loop that simple can be **vectorised**: the compiler uses the CPU's SIMD instructions to write several items per instruction. This makes bulk pushes many times faster than a `push` loop.

### `is_balanced`: bytes and a lookup table

The bracket checker scans every character of a text, so its inner loop matters most. Three changes make it faster:

1. **Bytes instead of `char`s.** Iterating with `text.chars()` decodes UTF-8 one character at a time, which takes several steps per character. Iterating over `text.as_bytes()` skips the decoding. This is safe because of how UTF-8 is designed: every byte of a multi-byte character is `0x80` or higher, so a byte equal to `(` (`0x28`) is always a real `(` and never part of another character.

2. **A lookup table instead of a `match`.** A `match` on six brackets compiles to a series of comparisons. Instead, a 256-entry table, computed at compile time, classifies any byte with a **single memory read**: `BRACKETS[byte]` is 0 for "not a bracket" (most bytes), the expected closing bracket for an opening one, or a marker for a closing one. The table is only 256 bytes, so it stays in the CPU cache the whole time.

   ```text
   index:  …  '('  ')'  …  '['  ']'  …  '{'  '}'  …  any other byte
   value:  …  ')'  CLOSER  ']'  CLOSER  '}'  CLOSER     0
   ```

3. **Store the expected closing bracket, as a `u8`.** When we see `(`, we push `)`. Checking a closing bracket is then a single equality test. A `u8` is 1 byte instead of 4 for a `char`, so the stack itself is 4× smaller and fits better in the cache.

### What was left alone

`push`, `pop` and `peek` are already as cheap as they can be, with just a few instructions each and no hidden costs. There is nothing to gain from rewriting them, for example with `unsafe` code.

## Run it

```bash
cargo run     # the demo in src/main.rs
cargo test    # unit tests and the examples in the docs
```

From the repository root, add `-p stack`.
