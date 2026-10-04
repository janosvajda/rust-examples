<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Slices

## The idea in one sentence

A **borrowed slice** gives a view of a contiguous run of items, covering part or all of some data.

A bookmark points you to a passage in a book without photocopying the pages. A borrowed slice works similarly: the original data stays where it is.

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

Making these borrowed slices does not copy their contents. A slice reference stores **a pointer plus a length**: items for `&[T]`, bytes for `&str`. The unsized slice types themselves are `[T]` and `str`; `&[T]` and `&str` are references to them. A test checks that `first_word` returns a slice starting at exactly the same address as the original text.

Range syntax: `[a..b]` is from `a` up to but **not including** `b`. `[..b]` starts at the beginning, `[a..]` goes to the end, and `[..]` is everything.

## Why functions should take `&str` and `&[T]`

Signatures from the runnable lesson:

```text
fn first_word(text: &str) -> &str
fn average(numbers: &[i32]) -> Option<f64>
```

A function that takes `&str` works with a `String`, a literal, or part of either. A function that takes `&String` only works with a whole `String`. Rust converts `&String` to `&str` (and `&Vec<T>` to `&[T]`) automatically when you pass it, so taking the slice type costs nothing and accepts more.

This is why Clippy suggested `&str` in lesson 2.

Our small `first_word` helper defines a word as the text before the first **ASCII space**. Thus `first_word("hello world")` is `"hello"`, a leading space gives `""`, and a tab is not a separator here. A general whitespace-based helper could use `text.split_whitespace().next().unwrap_or("")`.

`average` returns `None` for an empty slice. It converts **each number** to `f64` before summing, so even `[i32::MAX, i32::MAX]` has an average of `2147483647.0` rather than overflowing an `i32` total. Floating-point answers are approximate in general.

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

`clear()` makes the `String`'s length zero; it does not promise to erase the old bytes or release the buffer. The important rule here is that the shared slice conflicts with `clear()`'s mutable borrow. A later operation could overwrite or reallocate the data. Once no further use needs `word`, `text.clear()` is fine.

## Text slices are measured in bytes

Rust strings are UTF-8. Letters like `a` take 1 byte, but `ő` takes 2. Slice positions count **bytes**, not characters:

```rust
let city = "Győr";       // 4 characters, 5 bytes
assert_eq!(city.get(0..3), None);          // ✓ no panic
assert_eq!(city.get(0..4), Some("Győ"));   // ✓ ends after the whole "ő"
let _ = &city[0..3];     // ✗ panics at runtime: byte 3 is in the middle of "ő"
```

Safe string slicing checks bounds and **Unicode scalar-value boundaries**. A Rust `char` is one Unicode scalar value. These checks happen at runtime, although a compiler may optimise a known check away.

A visible character can contain several scalar values. For example, `"e\u{301}"` displays an `e` with a combining accent:

```rust
let accented = "e\u{301}";
assert_eq!(accented.chars().count(), 2); // two Unicode scalar values
assert_eq!(&accented[..1], "e");        // valid UTF-8; the accent is separate
```

So a valid UTF-8 slice can still split a user-perceived character, called a **grapheme cluster**. The `Győr` example uses one scalar value per visible letter.

## Run it

```bash
cargo run
cargo test
```

For more detail: [Rust strings, bytes and characters](https://doc.rust-lang.org/book/ch08-02-strings.html).

Previous: [Lesson 3: The borrowing rules](../03-borrowing-rules/) · Next: [Lesson 5: Lifetimes](../05-lifetimes/)
