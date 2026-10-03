<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 10: Advanced lifetimes

## The idea in one sentence

The default lifetime rules from lesson 5 are right most of the time. This lesson covers the cases where they're **too strict or simply wrong**, and how to say what you really mean.

Remember the key fact: **lifetime annotations never change how long anything lives.** They describe which references are connected, so the compiler can check your code. More precise annotations just let the compiler accept more correct programs.

## 1. A method's result can come from the data, not from `self`

```rust
struct Excerpt<'text> { part: &'text str }

impl<'text> Excerpt<'text> {
    fn part(&self) -> &str        { self.part }   // elided: tied to `self`
    fn part(&self) -> &'text str  { self.part }   // precise: tied to the text
}
```

By elision rule 3 (lesson 5), `-> &str` gets **`self`'s** lifetime: "the result lives only as long as this `Excerpt`." But the text actually lives in the original string, which usually lives much longer. With the elided version, this doesn't compile:

```rust
fn first_sentence(text: &str) -> &str {
    Excerpt::new(text).part()      // the Excerpt is a temporary, dropped right here
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

With one lifetime for both (`Found<'a>`), the compiler must assume each field could be either, so the result is only usable while **both** sources are alive. Two parameters keep them independent. In the demo, the key is dropped and the found value is still used afterwards. With a single `'a`, that fails:

```text
error[E0597]: `key` does not live long enough
```

It only compiles because `value` is linked to the table alone.

**Rule of thumb:** start with one lifetime. Add a second only when the compiler stops you from using the result in a place where you know it's still valid.

## 3. Trait objects that borrow: `dyn Trait + 'a`

`Box<dyn Trait>` silently means `Box<dyn Trait + 'static>`: whatever is inside can't borrow anything temporary. A closure that captures a reference breaks that:

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
fn make_labeler<'a>(prefix: &'a str) -> Box<dyn Fn(&str) -> String + 'a>
```

## 4. What a returned `impl Trait` borrows (Rust 2024)

In the 2024 edition, a function returning `impl Trait` is assumed to borrow **every** reference parameter, including `&self`. That's usually right, but not always:

```rust
fn multiples(&self, count: u32) -> impl Iterator<Item = u32> {
    let step = self.step;                 // copies the number…
    (0..count).map(move |i| i * step)     // …so the iterator doesn't need `self`
}

let multiples = counter.multiples(4);
counter.step = 5;                         // ✗ the signature says `multiples` borrows `counter`
```

```text
error[E0506]: cannot assign to `counter.step` because it is borrowed
```

`+ use<>` lists exactly what the result borrows. Empty means nothing:

```rust
fn multiples(&self, count: u32) -> impl Iterator<Item = u32> + use<>
```

## 5. `T: 'a`

```rust
struct Labelled<'a, T: Display + 'a> {
    value: &'a T,
}
```

`T: 'a` means *"any references inside `T` must live at least as long as `'a`."* It's needed for `&'a T` to make sense: you can't have a reference that's valid longer than the data inside the thing it points to. Rust usually adds it for you, but you'll see it in library signatures. `T: 'static` (lesson 5) is the same idea with the longest possible lifetime.

## 6. Closures that take references: `for<'a>`

```rust
fn total_by<F: Fn(&str) -> usize>(words: &[String], measure: F) -> usize
```

The closure is called with a different, short-lived `&str` each time. So `F` mustn't work for just *one* lifetime, it must work for **every** lifetime. In full, the bound is written:

```rust
F: for<'x> Fn(&'x str) -> usize
```

That's a **higher-ranked trait bound**. Rust adds the `for<'x>` automatically whenever a closure bound has reference parameters, so you'll rarely write it, but you'll see it in error messages and documentation.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 9: Borrowing in patterns](../09-borrowing-in-patterns/) · Next: [Lesson 11: Cow and the borrowing traits](../11-cow-and-borrowing-traits/)
