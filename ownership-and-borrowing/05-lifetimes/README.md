<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Lifetimes

## The idea in one sentence

A reference must **never outlive the value it points to**, and **lifetimes** are how the compiler checks that.

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

A **lifetime** is the stretch of code during which a reference is used. The compiler checks that every reference's lifetime fits **inside** the lifetime of the value it borrows.

The compiler works this out on its own almost all the time. You only write lifetimes when it can't tell how references relate. They look like `'a` (an apostrophe and a name).

> **Lifetime annotations never change how long anything lives.** They only *describe* how references are connected, so the compiler can check your code. Getting one wrong gives you a compile error, never a bug.

## When you have to write them

**A function that returns a reference, with more than one reference input:**

```rust
fn longest(a: &str, b: &str) -> &str { … }
```

```text
error[E0106]: missing lifetime specifier
```

The result comes from either `a` or `b`, so the compiler can't know which one the caller must keep alive. You tell it:

```rust
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str
```

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
fn after_prefix<'a>(text: &'a str, prefix: &str) -> &'a str
```

Now `prefix` can be dropped while the result is still in use. The demo and a test show this.

## When you don't: the elision rules

The compiler fills in lifetimes for you in three cases ("elision"):

1. Each reference parameter gets its own lifetime.
2. If there's **exactly one** reference parameter, the result gets its lifetime. So `fn first_line(text: &str) -> &str` needs no annotation.
3. If one of the parameters is `&self` or `&mut self`, the result gets `self`'s lifetime. That's why most methods returning references need no annotation.

If none of these rules decides the result's lifetime, you get `E0106` and must write it yourself.

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

A struct that holds a reference needs a lifetime parameter. It means *"a `Highlight` can't outlive the text it points into."* Creating one from a temporary `String` that's dropped before the `Highlight` is used gives:

```text
error[E0597]: `temporary` does not live long enough
```

This is useful for things like parsers, which point into the input instead of copying it. If the struct needs to live independently, store an owned `String` instead.

## `'static`

`'static` means **valid for the entire run of the program**. String literals are `&'static str`: they're stored inside the program file itself, so they never go away.

```rust
fn app_name() -> &'static str { "borrow-checker-demo" }
```

You'll also see `T: 'static` as a requirement, for example in `std::thread::spawn`. It doesn't mean "lives forever". It means "doesn't contain any borrowed data that could expire", so owned types like `String` satisfy it.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 4: Slices](../04-slices/) · Next: [Lesson 6: Borrowing in practice](../06-borrowing-in-practice/)
