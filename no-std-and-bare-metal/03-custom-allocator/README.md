<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: A custom allocator

## The idea in one sentence

`Vec`, `String`, `Box` and friends live in the **`alloc`** crate, which works without `std`, as long as **you provide the heap allocator**. This lesson writes one.

## `alloc`: the middle layer

```text
  std     ← needs an operating system
  alloc   ← needs a heap allocator      ← this lesson: we supply one
  core    ← needs nothing
```

On a normal operating system, `std` provides the allocator. On a microcontroller, you choose one. Popular crates exist, like `embedded-alloc`, but writing a simple one shows exactly what an allocator does.

## A bump allocator

```text
  arena: [ used │ used │ used │ ← next │              free              ]
         0                       ▲                                     64 KB
                                 every new allocation starts here
```

The simplest allocator there is. It has one fixed block of memory (here 64 KB) and one number, the offset of the next free byte:
- **To allocate**, round the offset up to the alignment the type needs, hand out that address, and move the offset forward. That's a few instructions, so it's very fast.
- **To free**, do nothing. Memory is never reused.

That trade-off suits programs that allocate during start-up and then run forever, which is common in firmware, or programs that reset the whole arena between jobs.

## Plugging it in

```rust
extern crate alloc;     // alloc isn't in scope automatically, unlike core

#[global_allocator]
static HEAP: BumpAllocator<65_536> = BumpAllocator::new();
```

`#[global_allocator]` makes it **the** allocator for the whole program. From then on, `Vec::new()`, `String::from`, `format!`, `Box::new` and `BTreeMap` all get their memory from `HEAP`. An allocator is anything implementing the `GlobalAlloc` trait, with two methods:

| Method | Must |
|---|---|
| `alloc(layout)` | return memory of `layout.size()` bytes aligned to `layout.align()`, or **null** if there's none left |
| `dealloc(ptr, layout)` | take back memory it handed out (ours just counts it) |

## What the demo shows

```text
[heap:   189 of 65536 bytes used, 6 allocations, 78 bytes freed but not reusable] after building the Vec
[heap:   792 of 65536 bytes used, 11 allocations, 404 bytes freed but not reusable] after the Box and the BTreeMap
[heap:   950 of 65536 bytes used, 12 allocations, 708 bytes freed but not reusable] after dropping the Vec and the Box
try_reserve(100,000) refused: the arena only has 64 KB
```

- **Dropping the `Vec` and the `Box` doesn't lower "used".** Freed memory is counted but never reused: that's the bump trade-off, made visible.
- **"Used" grows even between steps** because printing each report calls `format!`, which allocates a temporary `String`, and that memory isn't reused either. A real bump-allocated program would avoid allocating in a loop like this.
- **Running out of memory is handled.** The allocator returns null, and `Vec::try_reserve` turns that into an `Err`. A plain `Vec::with_capacity(100_000)` would instead call the allocation-error handler, which panics, and our panic handler aborts.

## Why `UnsafeCell` and an atomic

The allocator is a `static`, so it's shared by the whole program, and Rust only allows shared things to be changed through `UnsafeCell` (or atomics). The "next free byte" offset is an `AtomicUsize`, updated with `compare_exchange`, so two threads allocating at the same time can never receive the same memory. That's concurrency course, lesson 4, in a real use.

## Run it

```bash
cargo run
cargo test     # tests the allocator directly, as an ordinary value
```

Like lesson 1, this crate is kept outside the repository's Cargo workspace, because it needs its own `panic = "abort"` profile.

Previous: [Lesson 2: A core-only library](../02-core-only-library/) · Next: [Lesson 4: Memory-mapped registers](../04-memory-mapped-registers/)
