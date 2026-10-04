<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: The stack and the heap

## The idea in one sentence

Every value lives either on the **stack**, which is fast and automatic but needs its size known at compile time, or on the **heap**, which holds data of any size for as long as needed, reached through a pointer. Types like `String`, `Vec` and `Box` are a small value on the stack that owns data on the heap.

## Two places for data

| | Stack | Heap |
|---|---|---|
| what goes there | local variables, function arguments, return addresses | data whose size is only known while running, or that must outlive the function |
| size | must be known at **compile time** | anything, decided at **run time** |
| speed | very fast: making room is just moving one pointer | slower: the allocator has to find a free block |
| freed | automatically, when the function returns | when its owner is dropped (in Rust); with `free` (in C) |
| limit | small: usually 8 MB for the main thread | as much as the computer has |

Think of the stack as a pile of plates: each function call puts a plate on top, and returning takes it off. Everything on a plate must have a known size. The heap is a big warehouse: you can ask for a shelf of any size, and you get back its address.

## How big is a value?

`std::mem::size_of` tells you the size of a value **on the stack**. These are real numbers from `cargo run`, on a 64-bit machine:

```text
    u8                                         1
    i32                                        4
    f64                                        8
    Point { x: f64, y: f64 }                  16
    [u8; 1000]                              1000
    &i32 (a reference)                         8
    &str (pointer + length)                   16
    String (pointer + length + capacity)      24
    Vec<u64> (the same three)                 24
    Box<[u8; 1000]> (just a pointer)           8
    Option<Box<i32>> (still one pointer)       8
```

Some things to notice:
- A **reference** is one pointer: 8 bytes on a 64-bit machine.
- A **`&str`** is a pointer **and** a length (16 bytes), because it has to know where the text ends. It's a *fat pointer*.
- A **`String`** is three numbers: where the text is on the heap, how long it is, and how much room is reserved. That's 24 bytes, whether the text is `"hi"` or a million characters.
- A **`Box`** is just one pointer, even when the value it points at is 1,000 bytes.
- **`Option<Box<T>>`** is still 8 bytes. A `Box` can never be null, so Rust uses the null value to mean `None`, and no extra space is needed. This is called the **niche optimisation**.

## A String is two parts

```text
   stack: the String (24 bytes)              heap: the text
  ┌──────────────────────────────┐          ┌─────────────────────┐
  │ pointer ─────────────────────┼────────► │ x x x x x … (1 MB)  │
  │ length   1000000             │          └─────────────────────┘
  │ capacity 1000000             │
  └──────────────────────────────┘
```

```text
"hi":        24 bytes on the stack, 2 bytes of text on the heap
a million x: 24 bytes on the stack, 1000000 bytes of text on the heap
```

## What a move really copies

Passing a `String` to a function **moves** it ([ownership lesson 1](../../ownership-and-borrowing/01-ownership-and-moves/)). Moving copies only the 24-byte stack part. The text on the heap doesn't move at all, and the program shows it:

```text
heap address before the move: 0x8fac00000
heap address after the move:  0x8fac00000
the same: true
```

(The addresses differ on every run and every machine. What matters is that they're the same as each other.) That's why moving a `String` with a million characters is just as cheap as moving one with two. After the move, the old variable can't be used any more, so there's always exactly one owner of the heap data, and exactly one place that frees it.

`clone()` is different: it asks the heap for a new block and copies the text, so the clone's address is different. That's why it's an explicit call, so you can see where the expensive copies happen.

## The stack is small

```rust
let big = [0u8; 100_000_000];       // 100 MB, on the stack
```

```text
thread 'main' has overflowed its stack
fatal runtime error: stack overflow, aborting
```

The main thread's stack is usually 8 MB, so a 100 MB array doesn't fit. The same data on the heap is no problem: `vec![0u8; 100_000_000]`. Large or growing data belongs on the heap. Deep recursion, where every call adds a plate to the stack, can overflow it too.

## Where this goes next

The rest of this course is about the types that own heap memory, the **smart pointers**:

| Lesson | Type | In short |
|---|---|---|
| 2 | `Box<T>` | one owner, a value on the heap |
| 3 | `Rc<T>`, `Weak<T>` | several owners, in one thread |
| 4 | `Arc<T>` | several owners, across threads |
| 5 | your own | how `Deref` and `Drop` make a smart pointer |
| 6 | `*const T`, `*mut T` | raw pointers: no owner, no checks, `unsafe` |

## Run it

```bash
cargo run
cargo test
```

Back to the [course overview](../) · Next: [Lesson 2: Box](../02-box/)
