<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: Ownership and moves

## The idea in one sentence

An ordinary owned value has **one owner at a time**; moving it hands it over, and dropping it runs its cleanup.

Imagine Ferris handing Ana a book. Ana now owns that book. A number written on a second piece of paper, however, can be copied without taking Ferris's paper away.

## The three rules

1. **An ordinary owned value has one owner.** That can be a variable, struct field or collection.
2. **Non-`Copy` values move.** Assignment or a by-value function argument transfers ownership. The old location cannot be read unless you put a new value there. `Copy` values follow the different rule below.
3. **Owned values are normally dropped when their owner leaves scope.** A `String` releases its heap buffer; a file handle closes its file. Assignment can drop the old value, and `drop(value)` can run cleanup early.

These examples need no garbage collector or manual `free`. Dropping is type-specific cleanup: it does not mean that every byte is erased. Rust also permits intentional leaks, for example with `mem::forget`, so automatic cleanup is not an unconditional promise in every program.

Later, `Rc` and `Arc` let several handles share ownership of one heap allocation. Each handle still has its own owner; the shared data lasts until the last strong handle is dropped.

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

**What really happens in memory:** a `String` has two parts. A small header (pointer, length, capacity) lives in the variable, and the text itself lives on the heap. A move transfers that header to `b`, and the compiler treats `a` as uninitialised. It is not an empty `String`. The compiler may optimise away the physical header copy. The text on the heap is **not** copied. A test checks that `b` points to the same heap memory `a` did.

```text
  before:  a ──► "hello" (heap)          after:  a   (unusable)
                                                 b ──► "hello" (same heap memory)
```

Why forbid using `a`? If both `a` and `b` could be used, both would try to free the same memory when they go out of scope. Safe Rust prevents this *double free*. An incorrect `unsafe` implementation can still cause memory errors.

## Copy

```rust
let x = 5;
let y = x;            // x is copied, not moved
println!("{x} {y}");  // ✓ both work
```

Types that implement the **`Copy`** trait are duplicated instead of moved. Familiar examples include:

- whole numbers, floats, `bool`, `char`;
- shared references (`&T`, see lesson 2);
- tuples and arrays made only of `Copy` types.

`String`, `Vec` and `Box` are **not** `Copy`: duplicating their ownership bits would not create a separately owned allocation.

Size does not decide this. A large array of `Copy` elements can be `Copy`, while a tiny custom struct moves unless you implement or derive the trait:

```rust
#[derive(Clone, Copy)]
struct Point { x: i32, y: i32 }

let a = Point { x: 1, y: 2 };
let b = a;
assert_eq!(a.x + b.y, 3); // both are usable
```

## Clone

```rust
let original = String::from("data");
let duplicate = original.clone();   // ✓ a full, independent copy
```

`.clone()` explicitly asks a type to create another value. **For `String`**, that copies the text into an independent buffer. Other types choose different behaviour: `Rc::clone` and `Arc::clone` create another handle to the **same** data. Cloning is not a universal promise of an independent deep copy.

Think of cloning a `String` as photocopying a book; cloning an `Rc` is making another membership card for the same shared library book.

## Functions take and give ownership

```rust
fn consume(text: String) -> usize { text.len() }   // takes ownership; drops `text` at the end
fn shout(mut text: String) -> String { text.push('!'); text }   // takes it, then gives it back

let message = String::from("goodbye");
consume(message);      // `message` moves into the function
println!("{message}"); // ✗ error[E0382]: borrow of moved value: `message`
```

Passing a non-`Copy` value by value moves it, like `let b = a`. Passing an `i32` copies it. Returning an owned `String` transfers ownership to the caller.

Moving ownership in and out just to *look* at a value would be clumsy. That's what **borrowing** solves, in lesson 2.

## Drop

```rust
{
    let _first = Noisy("first");
    let _second = Noisy("second");
}   // prints: dropping second, dropping first
```

`Noisy` is defined in the runnable lesson and implements `Drop` to print a message. Local variables leaving the same scope are dropped in **reverse declaration order**. That happens to match reverse creation order above, but a variable can be declared before its value is created.

Struct fields follow a different rule: they are dropped in **declaration order**. A `Vec` owns its remaining elements and drops them when it is dropped; do not rely on a particular element drop order for `Vec`.

## Summary

| You write | For `String`, `Vec`, `Box`… | For `i32`, `bool`, `char`… |
|---|---|---|
| `let b = a;` | **move**: `a` can't be used any more | **copy**: both usable |
| `f(a)` | **move** into the function | **copy** |
| `a.clone()` | `String` copies its text; `Vec` and `Box` clone their elements or pointee | same as a copy |

## Run it

```bash
cargo run     # each step prints what's happening
cargo test
```

The failing lines are comments in `src/main.rs`. Uncomment an example to see its diagnostic.

For the precise rules: [Copy](https://doc.rust-lang.org/std/marker/trait.Copy.html), [Clone](https://doc.rust-lang.org/std/clone/trait.Clone.html) and [drop order](https://doc.rust-lang.org/reference/destructors.html).

Next: [Lesson 2: References](../02-references/)
