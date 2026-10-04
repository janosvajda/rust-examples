<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Lifetimes

## The idea in one sentence

A reference must point to valid data whenever it is used, and **lifetimes** describe how long that access must remain valid.

Imagine a library lending you a book. A borrowing ticket cannot grant access after the book has been returned and removed. A lifetime annotation describes that relationship; it does not keep the book alive by itself.

## The bug lifetimes prevent: dangling references

```rust
let r;
{
    let x = 5;
    r = &x;       // r points at x
}                 // x is dropped here
println!("{r}");  // ✗ r would point at memory that no longer holds x
```

```text
error[E0597]: `x` does not live long enough
```

```text
  let r;          r: ┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄ used here ✗
  {                       ┌──────────────┐
      let x = 5;          │  x is alive  │
      r = &x;             │              │
  }                       └──────────────┘ ◄── x is dropped
```

A pointer to a value that no longer exists is called a *dangling reference*. In C it's a common source of crashes and security holes. In Rust, safe code can't create one.

## What a lifetime is

A reference's **lifetime** is the region of code for which it must remain valid. Uses through copies, returned references, captured values or destructors can require a longer region than the original reference's last visible use. The compiler checks that the borrowed data is valid throughout that region.

The compiler infers many lifetimes. In signatures, fixed **elision rules** fill in common relationships; otherwise you write the relationship explicitly. They look like `'a` (an apostrophe and a name).

> **Lifetime annotations never change how long anything lives.** They only *describe* how references are connected, so the compiler can check your code. Safe Rust checks that the implementation satisfies the relationship. Unsafe code must uphold additional safety requirements itself.

## When you have to write them

**When the signature's input lifetimes do not uniquely determine the output:**

This example **does not compile**:

```rust
fn longest(a: &str, b: &str) -> &str {
    if a.len() >= b.len() { a } else { b }
}
```

```text
error[E0106]: missing lifetime specifier
```

The result comes from either `a` or `b`, so the compiler can't know which one the caller must keep alive. You tell it:

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}
```

Here, "longest" means **most UTF-8 bytes**, and ties choose `a`. For example, `longest("éé", "abc")` returns `"éé"`: it has four bytes, even though it has fewer visible letters.

Read it as: *"there's some lifetime `'a`; both inputs live at least that long, and so does the result."* In practice, the result can only be used while **both** inputs are still alive:

```rust
let first = String::from("a long sentence");
let result;
{
    let second = String::from("short");
    result = longest(&first, &second);
}                        // second is dropped
println!("{result}");    // ✗ error[E0597]: `second` does not live long enough
```

Even though `result` happens to be `first` here, the *signature* says it could be `second`, and the compiler only trusts the signature.

**Link only what's really linked.** If the result can only come from one input, give only that input the lifetime:

```rust
fn after_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}
```

Now `prefix` can be dropped while the result is still in use. The demo and a test show this.

## When you don't: the elision rules

The rules for elided lifetimes in function and method signatures are:

1. Each elided lifetime in the input types becomes a distinct lifetime parameter.
2. If **exactly one lifetime** appears in the input types, that lifetime is assigned to elided output lifetimes. `fn first_line(text: &str) -> &str` is a common example.
3. For a method with a `&self` or `&mut self` receiver, elided output lifetimes use that receiver reference's lifetime.

Count lifetimes, rather than the number of arguments. For example, this signature is ambiguous despite having one reference argument:

```text
fn choose(pair: &(&str, &str)) -> &str // does not compile: several input lifetimes
```

If the rules cannot determine an elided output lifetime, the compiler asks you to specify it.

## Returning a reference to a local value: not possible

```rust
fn make_greeting(name: &str) -> &str {
    let greeting = format!("Hello, {name}!");
    &greeting        // ✗ greeting is dropped when the function returns
}
```

```text
error[E0515]: cannot return reference to local variable `greeting`
```

No lifetime annotation can fix this: the value really is gone. The fix is to **return the owned value** (`-> String`) and let the caller own it.

## Structs that hold references

```rust
struct Highlight<'a> {
    text: &'a str,
}
```

A struct that stores a borrow whose lifetime varies uses a lifetime parameter. Here it means *"a `Highlight` can't outlive the text it points into."* Creating one from a temporary `String` that's dropped before the `Highlight` is used gives:

```text
error[E0597]: `temporary` does not live long enough
```

This is useful for parsers that point into input instead of copying it. If the struct needs independent ownership of its text, store a `String` instead.

A fixed lifetime is another option. A struct storing only a string literal needs no lifetime parameter:

```rust
struct App { name: &'static str }
let app = App { name: "Ferris's library" };
assert_eq!(app.name, "Ferris's library");
```

## `'static`

For a reference, `'static` means **valid for the entire run of the program**. String literals are `&'static str`: their text is embedded in the program and remains available in memory throughout its run.

```rust
fn app_name() -> &'static str { "borrow-checker-demo" }
```

You'll also see `T: 'static` as a requirement, for example in `std::thread::spawn`. It doesn't mean "lives forever". It means "doesn't contain any borrowed data that could expire", so owned types like `String` satisfy it.

## Run it

```bash
cargo run
cargo test
```

Precise rules: [lifetime elision](https://doc.rust-lang.org/reference/lifetime-elision.html) and [the Rust Book's lifetime chapter](https://doc.rust-lang.org/book/ch10-03-lifetime-syntax.html).

Previous: [Lesson 4: Slices](../04-slices/) · Next: [Lesson 6: Borrowing in practice](../06-borrowing-in-practice/)
