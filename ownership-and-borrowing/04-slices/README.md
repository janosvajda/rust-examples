<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Slices

## The idea in one sentence

A **slice** is a reference to a **part** of a collection: a run of items that sit next to each other in memory.

## The two slice types you'll use all the time

| Type | A slice of… | Borrowed from |
|---|---|---|
| `&str` | text | a `String`, a string literal (`"hi"`), or another `&str` |
| `&[T]` | items of type `T` | a `Vec<T>`, an array `[T; N]`, or another slice |
| `&mut [T]` | items you can change | a `Vec<T>` or array you can borrow mutably |

```rust
let sentence = String::from("hello wonderful world");
let hello = &sentence[0..5];    // "hello"
let world = &sentence[16..];    // "world"
```

```text
  sentence ──► h e l l o   w o n d e r f u l   w o r l d     (one String on the heap)
               └───┬───┘                         └───┬───┘
  hello ───────────┘                                 └────── world
```

Nothing is copied. A slice is just **a pointer to the first item plus a length**. A test checks that `first_word` returns a slice starting at exactly the same address as the original text.

Range syntax: `[a..b]` is from `a` up to but **not including** `b`. `[..b]` starts at the beginning, `[a..]` goes to the end, and `[..]` is everything.

## Why functions should take `&str` and `&[T]`

```rust
fn first_word(text: &str) -> &str
fn average(numbers: &[i32]) -> Option<f64>
```

A function that takes `&str` works with a `String`, a literal, or part of either. A function that takes `&String` only works with a whole `String`. Rust converts `&String` to `&str` (and `&Vec<T>` to `&[T]`) automatically when you pass it, so taking the slice type costs nothing and accepts more.

This is why clippy suggested `&str` in lesson 2.

## A slice is a borrow, so the rules apply

```rust
let mut text = String::from("hello world");
let word = first_word(&text);   // `word` borrows `text`
text.clear();                    // ✗ would empty the text `word` points into
println!("{word}");
```

```text
error[E0502]: cannot borrow `text` as mutable because it is also borrowed as immutable
```

Without this rule, `word` would still say "hello" after the text it points into had been erased. Once `word` is no longer used, `text.clear()` is fine.

## Text slices are measured in bytes

Rust strings are UTF-8. Letters like `a` take 1 byte, but `ő` takes 2. Slice positions count **bytes**, not characters:

```rust
let city = "Győr";       // 4 characters, 5 bytes
&city[0..3]              // ✗ panics at runtime: byte 3 is in the middle of "ő"
city.get(0..3)           // ✓ returns None instead of panicking
city.get(0..4)           // ✓ Some("Győ")
```

Slicing can never give you half a character. It either panics, or with `.get()` returns `None`. Unlike the borrowing rules, this is checked **when the program runs**, because the compiler doesn't know the text in advance.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: The borrowing rules](../03-borrowing-rules/) · Next: [Lesson 5: Lifetimes](../05-lifetimes/)
