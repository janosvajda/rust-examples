<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: Closures

## The idea in one sentence

A **closure** is an anonymous function you can store in a variable and pass around, and that can **capture** variables from where it was written.

## Syntax

```rust
let add_one = |x: i32| -> i32 { x + 1 };   // fully written out
let double  = |x| x * 2;                    // types inferred, single expression
let greet   = || String::from("hello");     // no parameters
```

Parameters go between `| |`. Types can usually be left out, and the compiler works them out from how the closure is used. Once a closure has been called with one type, that's its type for good.

## What makes a closure different from a function: capturing

```rust
let tax_rate = 0.27;
let with_tax = |price: f64| price * (1.0 + tax_rate);   // uses tax_rate from outside
```

A closure can use variables from the surrounding code. A function (`fn`) can't, even when it's written inside another function:

```text
error[E0434]: can't capture dynamic environment in a fn item
```

## Three ways to capture

A closure captures each variable in the **least powerful way that works**, exactly like the borrowing rules (ownership course, lesson 6):

| The closure… | captures by | Example |
|---|---|---|
| only reads the variable | shared borrow `&` | `\|\| names.len()` |
| changes the variable | mutable borrow `&mut` | `\|entry\| log.push(…)` |
| is marked `move` | taking ownership | `move \|\| owned.len()` |

Because they're borrows, the usual rules apply. While a closure that mutably borrows `log` is still going to be used, nothing else can touch `log`. Once its last use is past, `log` is free again.

## Every closure has its own type

```rust
let mut list = vec![|x: i32| x + 1];
list.push(|x: i32| x + 1);           // ✗ identical code, but a different type
```

```text
error[E0308]: mismatched types
  = note: no two closures, even if identical, have the same type
```

Every closure gets a unique, compiler-generated type that holds whatever it captured. That's why you can't write a closure's type by name, and why lesson 3 needs `impl Fn` and `Box<dyn Fn>` to store and return them.

## The exception: closures that capture nothing

A closure that captures **nothing** can turn into a plain **function pointer**, with the type `fn(i32) -> i32`. Since they all become the same type, they *can* share a `Vec`:

```rust
type Operation = (&'static str, fn(i32) -> i32);    // a short name for a long type

let operations: Vec<Operation> = vec![
    ("double", |x| x * 2),
    ("square", |x| x * x),
    ("negate", |x| -x),
];
```

Only non-capturing closures can do this. A closure that captures something carries that data with it, so it doesn't fit in a plain function pointer.

## Where you'll use closures most

As arguments, to say **how** something should be done:

```rust
words.sort_by_key(|w| w.to_lowercase());          // how to sort
words.iter().filter(|w| w.len() > 5)              // what to keep
thread::spawn(move || { … })                      // what to run
```

## Run it

```bash
cargo run
cargo test
```

Next: [Lesson 2: Fn, FnMut and FnOnce](../02-fn-traits/)
