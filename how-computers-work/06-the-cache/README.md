<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: The cache

## The idea in one sentence

A CPU keeps copies of useful data in small, fast **caches**, so the order in which a program visits memory can change its running time dramatically.

## Imagine…

Ferris's kitchen has a **shelf by the stove**, a **cupboard**, and a **warehouse down the road**.

Ingredients on the shelf are quick to reach. The warehouse holds much more, but a trip takes longer. When Ferris fetches flour, he brings a whole bag, hoping the next spoonful will be needed soon.

| Kitchen picture | Computer idea |
|---|---|
| Shelf | The small, very fast **L1** (level 1) cache |
| Cupboard | A larger, slightly slower cache, **L2** (level 2) |
| Warehouse | Main memory, usually DRAM |
| Bag | A **cache line**, a block of neighbouring bytes |

For these examples, the processor manages caching while the program uses ordinary memory accesses. It doesn't ask for a particular cache level.

At a particular cache level, finding the data is a **cache hit**; not finding it is a **cache miss**. An L1 miss can still be an L2 hit! A miss needs a request further along the memory hierarchy; the processor may do other work while it waits.

## First, know what we're measuring

Run this lesson with **`cargo run --release`**. A debug build ([lesson 1](../01-bits-and-bytes/)) adds so many extra checks that it mostly measures itself. The results below come from one run on an Apple M2 Mac. They are observations, not promises: your numbers will differ.

A **nanosecond**, or ns, is one billionth of a second. A **millisecond**, or ms, is one thousandth of a second. Times this small change with the compiler's choices, other running programs, the processor's current speed, which core runs the program, and what happens to be in the caches already.

The sizes are powers of two: **KiB** is 1,024 bytes and **MiB** is 1,048,576 bytes ([lesson 2](../02-memory-and-addresses/) explains the difference from KB and MB).

## A treasure trail through memory

The program builds a table where each entry tells it which entry to visit next:

```text
slot 0 says 7 → slot 7 says 3 → slot 3 says 12 → …
```

The next read's address depends on the value returned by the current read. That limits how much work can overlap. Hardware may still guess or prefetch, and a needed line may already be cached. This pattern, called **pointer chasing**, estimates the average time per dependent read, including the loop's work. The visiting order is random, so the reads jump all over the table.

```text
1. The time of one memory read, depending on how much memory is in use
     16 KiB of data:    2.2 ns per read
      1 MiB of data:    5.3 ns per read
    256 MiB of data:  101.2 ns per read
```

The program does exactly the same work each time; only the table size changes. On the tested Mac, macOS reports an L1 data cache of 128 KiB and an L2 cache of 16 MiB for the fast cores:

| Table size | Time per read | Likely explanation, not a measured cache-hit count |
|---|---:|---|
| 16 KiB | about 2 ns | Small enough to fit on the shelf (L1). |
| 1 MiB | about 5 ns | Too big for that L1, but small enough for the cupboard (L2). |
| 256 MiB | about 100 ns | Much bigger than those caches, so random visits often need main memory. |

Each time also includes the loop's own work, and translating addresses ([lesson 7](../07-virtual-memory/)). The program doesn't count where each read came from, so these numbers show the pattern, not exact cache speeds.

For perspective, at 3.5 GHz ([lesson 4](../04-a-tiny-cpu/) explains clock cycles), 100 ns is about **350 clock cycles**: time the processor spends mostly waiting.

## A cache line brings neighbours

For ordinary cacheable memory, a missed load normally brings in a **whole cache line** containing the requested byte.

Memory is divided into lines of equal size, each starting at a multiple of the line size (**aligned**, as in [lesson 2](../02-memory-and-addresses/)). The tested Mac uses **128-byte** lines, so one line holds **32 `u32` values**. A miss brings in the whole line that contains the requested value: its neighbours on both sides, not necessarily the 31 values *after* it.

```text
one line: [0][1][2] … [20] … [31]
                       ↑
reading element 20 brings in this whole line, including earlier neighbours
```

If that line stays in the cache, later reads of its neighbours can be hits. Many Intel and AMD processors use 64-byte lines instead; the sizes depend on the processor. Each core usually has its own L1 cache, while bigger caches may be shared between cores. The M2 has two kinds of cores, fast and energy-saving, with different cache sizes.

A processor may also **prefetch**: when it notices a pattern, such as reading line after line in order, it fetches the next lines before they're asked for. Like Ferris noticing he's baking a whole batch, and bringing the next bag before the first one is empty. See [Intel's optimisation manual](https://www.intel.com/content/dam/doc/manual/64-ia-32-architectures-optimization-manual.pdf) for one processor family's cache and prefetch mechanisms.

## Same numbers, different visiting order

The next experiment adds up **16,777,216 `u32` values**, occupying **64 MiB**, much bigger than the caches. First in order, then in a random order:

```text
2. Reading 64 MiB: in order, or in random order
    in order:         5.8 ms
    random order:    55.0 ms   (9× slower, the same numbers)
```

In order, each line brought in serves 32 values, and prefetching fetches the next lines early. Random visits usually reuse nearby values less effectively and are harder for ordinary sequential prefetchers to predict. They can still hit cached lines, and several independent reads may overlap. This program doesn't count misses, so we can't conclude that each value caused one.

To be fair, the random version does a little more work: it also reads the list of positions to visit (another 64 MiB, read in order). The comparison includes this extra index-reading work. It demonstrates two access strategies; it doesn't isolate the cost of visiting order alone.

Both sums must agree. Timing differences don't change the answer.

## A bookshelf-sized grid

We store a 4096 × 4096 grid in **one flat vector, row after row**, using:

```text
position = row × side + column
```

For a smaller example:

```text
drawing:         flat storage:
A B C            [A B C D E F G H I]
D E F
G H I
```

Rows visit A, B, C, D, E, F… Columns visit A, D, G, B, E, H…

The actual summing functions use:

```rust
// Rows first:
for row in 0..side {
    for column in 0..side {
        total += u64::from(grid[row * side + column]);
    }
}

// Columns first:
for column in 0..side {
    for row in 0..side {
        total += u64::from(grid[row * side + column]);
    }
}
```

These are function-body excerpts; `grid`, `side` and `total` come from the surrounding function.

`u64::from` turns each `u32` into a `u64`, so the total can grow beyond the largest `u32`. This grid's sum fits in `u64`; even a `u64` could overflow for a sufficiently large different calculation.

Row by row, the next value is the very next 4 bytes. Column by column, with side 4096, the next value is **4096 × 4 = 16,384 bytes** further on: a different line every time.

```text
3. A 4096 × 4096 grid: row by row, or column by column
    row by row:           6.4 ms
    column by column:    59.4 ms   (9× slower, the same sum)
```

Storing rows one after another is *our* choice for this program. Some libraries store grids column after column instead, and then columns are the fast direction. The rule is: visit the data in the order it's stored.

## Be a benchmark detective

A benchmark can lie. In a release build, the compiler's **optimiser** looks for shortcuts. If it knew `side` was always 4096, it could rearrange the column loop into the fast order, or even work out parts of the answer in advance, and we would no longer be measuring what we think. The program passes `side` through `std::hint::black_box`, which asks the compiler to treat the value as unknown. It's a **best-effort hint**, not a guarantee. See [Rust's `black_box` documentation](https://doc.rust-lang.org/std/hint/fn.black_box.html).

When a result is surprisingly fast, suspect the optimiser or the caches before celebrating. Check the generated assembly ([lesson 5](../05-real-machine-code/) shows how) before claiming the compiler did something. Run the measurement several times, and always check that the answers agree as well as comparing the times: this program asserts that both sums are equal.

**Fun fact:** changing only the order of two loops can change how much travelling the kitchen needs, even though the recipe's final answer stays identical!

## What can you use in real programs?

| Useful starting point | Why it can help |
|---|---|
| Use arrays or `Vec` for a sequence | Neighbouring elements share cache lines. |
| Visit data in storage order | Reuse and prefetching often improve. |
| Keep frequently used data together | Fewer lines may be needed. |
| Measure structures full of pointers | Following pointers to values scattered around memory, as in a linked list, can mean a miss at every step, like the treasure trail above. |

These are starting points, not rules that win every time.

## Words to remember

| Word | Meaning |
|---|---|
| **cache** | a small, fast memory on the processor chip, holding copies of recently used memory |
| **L1, L2, …** | the cache levels: L1 is the smallest and fastest, the next levels bigger and slower |
| **cache hit / miss** | the data was already in the cache / it had to be fetched from further away |
| **cache line** | the block of neighbouring bytes (often 64 or 128) that the cache brings in at once |
| **prefetch** | fetching lines early, before the program asks for them |
| **pointer chasing** | each read supplies the next read's location, limiting overlapping work |
| **benchmark** | a program that measures speed |

First choose a correct representation, then measure the work your program actually does.

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1968** | IBM describes the cache in the **System/360 Model 85**, among the first commercial computers with one. The paper's title calls it **the cache**, a name that stuck. Ordinary loads can benefit from this hidden store without explicitly choosing it. [The original IBM paper](https://doi.org/10.1147/sj.71.0015) |
| **1989** | Intel's **80486**, a chip with more than a million transistors, includes a cache on the processor chip itself. The cache and other improvements increased performance without relying only on a higher clock rate. [Computer History Museum, 1989](https://www.computerhistory.org/timeline/1989/) |
| **1994 / 1995** | Wulf and McKee described the growing processor–memory speed gap as the **“memory wall”** in a 1994 report, published in a journal in 1995. Processors were getting faster much more quickly than memory, so more and more time would be spent waiting for data, however fast the arithmetic became. Caches help, alongside better locality, prefetching and other techniques, but they don't make the memory-speed problem disappear. [Their paper](https://libraopen.lib.virginia.edu/downloads/4b29b598d) |

## Run it

From this lesson's directory:

```bash
cargo run --release
cargo test
```

The largest table is **256 MiB**, so the program needs a few hundred MiB of memory and a few seconds to build and shuffle its tables. The tests check that the visiting orders are correct and that the sums agree; they don't check any timings, because those depend on the computer.

**Try it:** with a row-major 4 × 4 grid of `u32` values, how far apart are consecutive values in one column?

<details>
<summary>Show the answer</summary>

**4 × 4 = 16 bytes.** Consecutive values in a row are only four bytes apart.

</details>

Previous: [Lesson 5: Real machine code](../05-real-machine-code/) · Next: [Lesson 7: Virtual memory](../07-virtual-memory/)
