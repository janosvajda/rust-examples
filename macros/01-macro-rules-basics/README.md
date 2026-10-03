<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: `macro_rules!` basics

## The idea in one sentence

A macro is **code that writes code**: it takes pieces of code as input and replaces itself with new code, before the compiler checks types.

## Why macros exist

You've used macros since your first Rust program: `println!`, `vec!`, `format!`, `assert_eq!`. The `!` marks a macro call. They do things ordinary functions can't:

| A function can't… | …but a macro can | Example |
|---|---|---|
| take any number of arguments | ✓ | `println!("{} {} {}", a, b, c)` |
| check things at compile time | ✓ | `println!("{}")` with no argument is a **compile** error |
| take code, not just values | ✓ | a block of code to time, a name to define |
| create new items | ✓ | define a function, struct or `impl` |

## Anatomy of a macro

```rust
macro_rules! square {
    ($x:expr) => { $x * $x };
//   ───┬───      ────┬────
//   pattern      template
}

square!(7)   →   7 * 7
```

- **Pattern** (`($x:expr)`): what the call must look like. `$x` names the matched piece, and `expr` says what kind of code it must be.
- **Template** (`$x * $x`): the code that replaces the call, with `$x` filled in.

A macro can have **several rules**, tried top to bottom like the arms of a `match`:

```rust
macro_rules! greet {
    () => { … };                                // greet!()
    ($name:expr) => { … };                      // greet!("Ferris")
    ($greeting:literal to $name:expr) => { … }; // greet!("Good morning" to "Ana")
}
```

Words in the pattern that aren't `$`-variables, like `to`, must appear literally in the call. If no rule matches, you get a compile error pointing at the token that didn't fit:

```text
error: no rules expected `,`
error: unexpected end of macro invocation
```

## Fragment types

| Fragment | Matches | Example |
|---|---|---|
| `expr` | an expression | `2 + 3`, `foo(x)`, `vec![1]` |
| `ident` | a name | `answer`, `my_var` |
| `ty` | a type | `u32`, `Vec<String>`, `&'static str` |
| `literal` | a literal value | `42`, `"text"`, `true` |
| `block` | a `{ … }` block | `{ (1..=10).sum() }` |
| `pat` | a pattern | `Some(x)`, `0..=9` |
| `tt` | any single token tree, the most flexible | anything |

With `ident` and `ty`, a macro can create things:

```rust
macro_rules! make_getter {
    ($name:ident, $type:ty, $value:expr) => {
        fn $name() -> $type { $value }
    };
}
make_getter!(answer, u32, 42);     // defines: fn answer() -> u32 { 42 }
```

## Rust macros aren't text substitution

C's `#define` pastes text, which causes famous bugs. Rust macros work on **syntax**, and that has two effects you can see in the demo.

**An `expr` stays one grouped expression:**

```rust
macro_rules! double { ($x:expr) => { $x * 2 }; }
double!(1 + 1)    // 4, i.e. (1 + 1) * 2, not 1 + 1 * 2 = 3 as a C macro would give
```

**Hygiene: a macro's variables can't clash with yours.**

```rust
macro_rules! set_x_to_ten { () => { let x = 10; }; }

let x = 1;
set_x_to_ten!();
println!("{x}");    // still 1
```

The `x` inside the macro is a *different* variable from yours, even with the same name. Trying to use it from outside fails:

```text
error[E0425]: cannot find value `x` in this scope
```

## Order matters

A `macro_rules!` macro must be **defined before it's used** in the file. The compiler reads macros top to bottom:

```text
error: cannot find macro `shout` in this scope
```

## Run it

```bash
cargo run
cargo test
```

Next: [Lesson 2: Repetition](../02-repetition/)
