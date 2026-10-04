<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 10: Advanced lifetimes

## The idea in one sentence

The default signature rules are useful, but sometimes connect lifetimes more tightly than your implementation needs. Explicit annotations can describe those relationships more precisely.

An excerpt is like a bookmark card: the card can disappear while the original book still exists. A returned passage can depend on the book without needing the card.

Remember the key fact: **lifetime annotations never change how long anything lives.** They describe which references are connected, so the compiler can check your code. More precise annotations just let the compiler accept more correct programs.

## 1. A method's result can come from the data, not from `self`

```rust
struct Excerpt<'text> { part: &'text str }
impl<'text> Excerpt<'text> {
    fn new(text: &'text str) -> Self {
        Self { part: text.split('.').next().unwrap_or("") }
    }
    fn part_elided(&self) -> &str { self.part } // tied to this borrow of self
    fn part(&self) -> &'text str { self.part } // tied to the original text
}
fn first_sentence(text: &str) -> &str {
    Excerpt::new(text).part() // the temporary Excerpt may be dropped
}
```

By elision rule 3, the output of `part_elided` is tied to **the reference borrowing `self`**. `part` explicitly ties its output to `'text` instead.

Add this function to the example above: it **does not compile**, because it uses the elided method on a temporary:

```rust
fn too_short(text: &str) -> &str {
    Excerpt::new(text).part_elided()
}
```

```text
error[E0515]: cannot return value referencing temporary value
```

Writing `-> &'text str` tells the truth: the result borrows from the text, not from the `Excerpt`, so the temporary can go away.

## 2. Different lifetimes for different references

```rust
struct Found<'key, 'data> {
    key: &'key str,      // borrowed from what we searched for
    value: &'data str,   // borrowed from the table we searched in
}
```

With one lifetime parameter shared by both inputs and fields, the API ties their required validity together. That does not make a field physically switch between sources; it limits what the caller can prove about its lifetime. Two parameters let the field lifetimes vary independently. In the demo, the key is dropped and the found value is still used afterwards. With a single `'a`, that fails:

```text
error[E0597]: `key` does not live long enough
```

It only compiles because `value` is linked to the table alone.

**Rule of thumb:** start with one lifetime. Add a second only when the compiler stops you from using the result in a place where you know it's still valid.

## 3. Trait objects that borrow: `dyn Trait + 'a`

In a return signature such as the one below, `Box<dyn Fn(&str) -> String>` defaults to `Box<dyn Fn(&str) -> String + 'static>`. The closure cannot hold a shorter-lived borrow. This example **does not compile**:

```rust
fn make_labeler(prefix: &str) -> Box<dyn Fn(&str) -> String> {
    Box::new(move |text| format!("{prefix}{text}"))     // captures `prefix`
}
```

```text
error: lifetime may not live long enough
```

Say how long the box may live, and it compiles:

```rust
fn make_labeler<'a>(prefix: &'a str) -> Box<dyn Fn(&str) -> String + 'a> {
    Box::new(move |text| format!("{prefix}{text}"))
}
```

This object lifetime bound describes borrows stored **inside the closure**, independently of its `&str` call arguments. `'static` does not force an owned closure to remain alive forever.

Local expression contexts can infer a shorter object lifetime. This works:

```rust
let prefix = String::from("[note] ");
let label: Box<dyn Fn()> = Box::new(|| println!("{prefix}"));
label(); // the closure and prefix are both valid here
```

## 4. What a returned `impl Trait` may capture (Rust 2024)

In Rust 2024, a return-position `impl Trait` **implicitly captures all in-scope generic parameters**, including the lifetimes of reference inputs such as `&self`. A captured lifetime allows the hidden return type to depend on it, and can restrict the caller even if the implementation does not store that borrow.

This example **does not compile** at the assignment:

```rust
struct Counter { step: u32 }
impl Counter {
    fn multiples(&self, count: u32) -> impl Iterator<Item = u32> {
        let step = self.step;
        (0..count).map(move |i| i * step)
    }
}
let mut counter = Counter { step: 2 };
let multiples = counter.multiples(4);
counter.step = 5; // the output type captures the self borrow's lifetime
println!("{:?}", multiples.collect::<Vec<_>>()); // keeps the iterator in use
```

```text
error[E0506]: cannot assign to `counter.step` because it is borrowed
```

A `use<...>` bound lists captured **generic parameters**: lifetimes, types and const parameters. It is not a list of runtime values held by a closure. This non-generic method can use `use<>` to exclude all generic captures, including the receiver lifetime.

Replace the method above with this method excerpt:

```rust,ignore
fn multiples(&self, count: u32) -> impl Iterator<Item = u32> + use<> {
    let step = self.step;
    (0..count).map(move |i| i * step)
}
```

The assignment then works. The iterator still owns its copied `step`; the empty capture list does not mean it contains no data. The demo uses small numbers; larger `i * step` products need an overflow policy.

An iterator borrowing another input can exclude `self` while retaining the input lifetime. This method excerpt appears in the runnable lesson:

```rust,ignore
fn labelled<'a>(&self, items: &'a [&'a str])
    -> impl Iterator<Item = String> + use<'a>
{
    items.iter().map(|item| format!("<{item}>"))
}
```

It keeps the items borrowed while allowing the counter to change. Generic APIs have additional capture-list requirements; consult the linked edition guide when adding type or const parameters.

## 5. `T: 'a`

```rust
use std::fmt::Display;

struct Labelled<'a, T: Display + 'a> {
    value: &'a T,
}
```

`T: 'a` means *"any references inside `T` must live at least as long as `'a`."* It's needed for `&'a T` to make sense: you can't have a reference that's valid longer than the data inside the thing it points to. Rust usually adds it for you, but you'll see it in library signatures. `T: 'static` (lesson 5) is the same idea with the longest possible lifetime.

## 6. Closures that take references: `for<'a>`

```rust
fn total_by<F: Fn(&str) -> usize>(words: &[String], measure: F) -> usize {
    words.iter().map(|w| measure(w)).sum()
}
```

The closure is called with a different, short-lived `&str` each time. So `F` mustn't work for just *one* lifetime, it must work for **every** lifetime. In full, the bound is written:

Bound excerpt:

```text
F: for<'x> Fn(&'x str) -> usize
```

That's a **higher-ranked trait bound**. In this `Fn(&str)` bound, the elided argument lifetime introduces an implicit `for<'x>`. An explicitly named lifetime, such as `Fn(&'a str)`, instead names that particular lifetime; it does not automatically mean "every lifetime".

## Run it

```bash
cargo run
cargo test
```

Precise rules: [elision and trait-object defaults](https://doc.rust-lang.org/reference/lifetime-elision.html) and [Rust 2024 capture rules](https://doc.rust-lang.org/edition-guide/rust-2024/rpit-lifetime-capture.html).

Previous: [Lesson 9: Borrowing in patterns](../09-borrowing-in-patterns/) · Next: [Lesson 11: Cow and the borrowing traits](../11-cow-and-borrowing-traits/)
