<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: Ownership and moves

## The idea in one sentence

Every value in Rust has **exactly one owner**, and when the owner goes away, the value is cleaned up automatically.

## The three rules

1. **Every value has one owner.** The owner is usually a variable, but can also be a struct field or a collection.
2. **Ownership can move.** Assigning a value to another variable, or passing it to a function, hands it over. The old owner can't be used any more.
3. **When the owner goes out of scope, the value is dropped.** Its memory is freed, files are closed, and so on, at a point you can see in the code: the closing `}`.

There's no garbage collector and no manual `free`. Rule 3 does the cleanup, and rules 1 and 2 make sure it happens exactly once.

## Move

```rust
let a = String::from("hello");
let b = a;            // ownership moves from `a` to `b`
println!("{a}");      // ✗ compile error
```

```text
error[E0382]: borrow of moved value: `a`
  move occurs because `a` has type `String`, which does not implement the `Copy` trait
```

**What really happens in memory:** a `String` has two parts. A small header (pointer, length, capacity) lives in the variable, and the text itself lives on the heap. A move copies only the header into `b`, and the compiler then treats `a` as empty. The text on the heap is **not** copied. A test checks that `b` points to the same heap memory `a` did.

```text
  before:  a ──► "hello" (heap)          after:  a   (unusable)
                                                 b ──► "hello" (same heap memory)
```

Why forbid using `a`? If both `a` and `b` could be used, both would try to free the same memory when they go out of scope. That bug, a *double free*, is impossible in Rust.

## Copy

```rust
let x = 5;
let y = x;            // x is copied, not moved
println!("{x} {y}");  // ✓ both work
```

Small values that live entirely in the variable, with nothing on the heap, implement the **`Copy`** trait and are duplicated instead of moved:
- whole numbers, floats, `bool`, `char`;
- shared references (`&T`, see lesson 2);
- tuples and arrays made only of `Copy` types.

`String`, `Vec`, `Box` and anything that owns heap memory are **not** `Copy`.

## Clone

```rust
let original = String::from("data");
let duplicate = original.clone();   // ✓ a full, independent copy
```

`.clone()` is how you ask for a real copy, including the heap data. It's always written out explicitly, so expensive copies are visible in the code.

## Functions take and give ownership

```rust
fn consume(text: String) -> usize { text.len() }   // takes ownership; drops `text` at the end
fn shout(mut text: String) -> String { text.push('!'); text }   // takes it, then gives it back

let message = String::from("goodbye");
consume(message);      // `message` moves into the function
println!("{message}"); // ✗ error[E0382]: borrow of moved value: `message`
```

Passing a value to a function is a move, exactly like `let b = a`. Returning a value moves it out again.

Moving ownership in and out just to *look* at a value would be clumsy. That's what **borrowing** solves, in lesson 2.

## Drop

```rust
{
    let _first = Noisy("first");
    let _second = Noisy("second");
}   // prints: dropping second, dropping first
```

At the end of a scope, values are dropped in **reverse order of creation**. `Noisy` implements the `Drop` trait to print a message, so you can watch it happen. A value moved into a `Vec` is owned by the `Vec`, and is dropped when the `Vec` is.

## Summary

| You write | For `String`, `Vec`, `Box`… | For `i32`, `bool`, `char`… |
|---|---|---|
| `let b = a;` | **move**: `a` can't be used any more | **copy**: both usable |
| `f(a)` | **move** into the function | **copy** |
| `a.clone()` | full copy, both usable | same as a copy |

## Run it

```bash
cargo run     # each step prints what's happening
cargo test
```

The lines that don't compile are left in `src/main.rs` as comments, with the real compiler error underneath. Uncomment one to see the full message.

Next: [Lesson 2: References](../02-references/)
