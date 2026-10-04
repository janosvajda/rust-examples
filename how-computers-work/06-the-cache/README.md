<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: The cache

## The idea in one sentence

Main memory is about **fifty times slower** than the processor's own small, fast memories, the **caches**, so the same program can run ten times faster or slower depending only on the **order** in which it reads memory.

## Imagine…

Imagine the cook from [lesson 4](../04-a-tiny-cpu/) again. Most ingredients are kept in a big **warehouse** down the road: everything fits there, but each trip takes ages. So the cook keeps the ingredients they're using right now on a small **shelf** next to the stove, and a bigger **cupboard** in the kitchen.

When the cook fetches flour from the warehouse, they don't bring one spoonful: they bring the whole bag, because the next spoonful will probably be needed soon. If the recipe uses ingredients in the order they're stored, almost everything comes from the shelf. If it jumps all over the warehouse, nearly every step is another long trip.

The warehouse is **main memory**. The shelf and the cupboard are the **caches**. The bag is a **cache line**.

## Precisely: measured

All numbers below were measured by `cargo run --release` on an Apple M2 Mac. On your computer they'll differ, but the pattern won't. The cache sizes are the ones macOS reports (`sysctl hw.l1dcachesize hw.l2cachesize`). The M2's two kinds of core have different cache sizes.

### How long does one read take?

The program makes a random chain through a table: each entry says which entry to read next. Each read's address depends on the previous read, so the processor can't start early or guess ahead. This **pointer chasing** measures the true time of one read:

```text
  16 KB of data:    2 ns per read     ← fits in the L1 cache (this Mac reports 64 KB)
   1 MB of data:    5.5 ns per read   ← fits in the L2 cache (this Mac reports 4 MB)
 256 MB of data:  105 ns per read     ← mostly in main memory
```

| Where the data is | Time per read | In the kitchen |
|---|---|---|
| **L1 cache**: tiny, inside each core | ~2 ns | the shelf by the stove |
| **L2 cache**: bigger, shared by several cores | ~5.5 ns | the cupboard |
| **main memory** (DRAM) | ~105 ns | the warehouse |

In 105 ns, a processor running at 3.5 GHz could have done about 370 steps of work. A read from main memory is like waiting a whole minute for an answer you could have had in a second.

### Cache lines: memory comes in blocks

The cache never fetches a single byte from memory. It always fetches a whole **cache line**: 128 bytes on this Mac (`sysctl hw.cachelinesize`), and 64 bytes on most Intel and AMD processors. So reading one `u32` also brings the next 31 into the cache, for free.

The processor also notices patterns. When it sees reads going forward through memory, it **prefetches**: it fetches the next lines before they're even asked for.

### In order vs random order

Both versions add up the same 16 million numbers (64 MB):

```text
in order:          3–6 ms
random order:     ~60 ms     (10–18× slower, the same numbers)
```

In order, every cache line brings 32 useful numbers, and the prefetcher stays ahead. In random order, almost every number costs a trip to main memory.

### Row by row vs column by column

A 4096 × 4096 grid is stored **row after row**: Rust, C and most languages do it this way. Adding it up row by row reads memory in order. Column by column jumps 4096 × 4 = 16 KB between reads:

```rust
for row in 0..side { for column in 0..side { total += grid[row * side + column]; } }   // fast
for column in 0..side { for row in 0..side { total += grid[row * side + column]; } }   // slow
```

```text
row by row:          6 ms
column by column:   60 ms    (10× slower, the same sum)
```

Swapping two lines of code makes the same calculation ten times slower.

**A trap when measuring this:** in the first version of this program, the grid's size, 4096, was written directly in the code. The compiler then rearranged the column loop by itself, and column by column came out *faster*: 1.4 ms. Only after hiding the size from the compiler, with `std::hint::black_box(side)`, did the program measure what it was meant to. Always question a benchmark result that seems too good. The compiler may not be running the code you wrote.

## What this means for your programs

| Do | Because |
|---|---|
| prefer `Vec` and arrays: items side by side | every cache line brings neighbours you're about to need |
| go through data in the order it's stored | the prefetcher keeps up; nothing waits for memory |
| keep data that's used together, together | one cache line instead of several |
| be careful with linked lists and pointer-heavy structures for big data | each step can be a 100 ns trip to main memory |

This is why [lesson 2](../02-memory-and-addresses/)'s simple rule, "array elements sit side by side", matters so much, and why the [data processing](../../data-processing/) course streams through data in order.

## A bit of history

| When | What happened |
|---|---|
| **1965** | Maurice Wilkes describes a small, fast **"slave memory"** holding copies of the most-used parts of main memory: the idea of the cache |
| **1968** | IBM's **System/360 Model 85** is the first commercial computer with a cache |
| **1980s–90s** | Processors get faster much more quickly than memory. In 1994, Wulf and McKee call the growing gap the **"memory wall"** |
| **today** | Several levels of cache (L1, L2, often L3) take up a large part of every processor chip, to hide that gap |

## Run it

```bash
cargo run --release      # timings: only meaningful in a release build
cargo test
```

Previous: [Lesson 5: Real machine code](../05-real-machine-code/) · Next: [Lesson 7: Virtual memory](../07-virtual-memory/)
