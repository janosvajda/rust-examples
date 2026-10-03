<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Closures and Iterators

Closures and iterators are everywhere in Rust: sorting, filtering, callbacks, threads, `Option` and `Result` helpers. This course explains how they really work, from capturing variables and the three `Fn` traits to writing your own iterator adapters.

Every compiler error and warning quoted in these READMEs is the real output of Rust 1.99.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [Closures](01-closures/) | syntax, capturing (borrow, mutable borrow, `move`), unique closure types, function pointers |
| 2 | [`Fn`, `FnMut` and `FnOnce`](02-fn-traits/) | which closure implements which trait, and which to ask for in a function |
| 3 | [Storing and returning closures](03-storing-and-returning-closures/) | `impl Fn`, `Box<dyn Fn>`, closures in struct fields, callbacks, memoisation |
| 4 | [The iterator toolbox](04-iterator-toolbox/) | laziness, the most useful adapters and consumers, `collect` into anything |
| 5 | [Writing your own iterators](05-writing-iterators/) | `from_fn`, `successors`, `impl Iterator`, `IntoIterator`, your own adapters |

## The errors you'll meet

| Error | In plain words | Lesson |
|---|---|---|
| **E0434** can't capture dynamic environment in a fn item | only closures can capture, not `fn` | 1 |
| **E0308** mismatched types · no two closures have the same type | two closures can't share a type; use `fn` pointers or `Box<dyn Fn>` | 1, 3 |
| **E0596** cannot borrow `f` as mutable | an `FnMut` closure must be stored in a `let mut` | 2 |
| **E0382** use of moved value | an `FnOnce` closure was called twice | 2 |
| **E0525** expected a closure that implements `Fn`, but it only implements `FnOnce` | the function needs more than the closure allows | 2 |
| **E0594** / **E0507** … captured variable in an `Fn` closure | the same, for a closure written directly in the call | 2 |
| **E0373** closure may outlive the current function | a returned closure needs `move` | 3 |
| unused `Map` that must be used | iterators are lazy; add a consumer | 4 |
| **E0283** type annotations needed | tell `collect` what to build | 4 |

## Run a lesson

```bash
cd closures-and-iterators/02-fn-traits
cargo run
cargo test
```

Or from the repository root: `cargo run -p fn-traits`. The package names are in each lesson's `Cargo.toml`.
