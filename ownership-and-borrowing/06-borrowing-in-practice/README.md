<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: Borrowing in practice

## The idea in one sentence

The rules from lesson 3 never change, but in real code you meet them in a handful of typical situations. Each one has a standard solution.

## 1. Struct fields can be borrowed separately

```rust
let title = &mut book.title;
let pages = &mut book.pages;   // ✓ a different field, so no conflict
```

The compiler knows `title` and `pages` are separate pieces of memory, so two mutable borrows of **different fields** are fine.

## 2. …but a method call borrows the whole struct

```rust
let title = &mut book.title;
let count = book.pages();      // ✗ the method borrows all of `book`
title.push('!');
```

```text
error[E0502]: cannot borrow `book` as immutable because it is also borrowed as mutable
```

Why the difference? The compiler only looks at a method's **signature**. `fn pages(&self)` says "I may read any field", including `title`, which is mutably borrowed. **Fix:** finish using the field borrow first, or read the field directly (`book.pages`) instead of calling a method.

## 3. Two mutable parts of one array or `Vec`: `split_at_mut`

```rust
let (morning, afternoon) = temperatures.split_at_mut(3);
morning[0] += afternoon[0];    // ✓ both halves mutable at once
```

Writing `&mut temperatures[..3]` and `&mut temperatures[3..]` at the same time doesn't compile, even though the ranges don't overlap. The compiler doesn't reason about index ranges; it just sees two mutable borrows of `temperatures`. `split_at_mut` is a standard-library function that *guarantees* the halves don't overlap, so it can safely give you both.

## 4. Three ways to loop over a collection

| Loop | Same as | Each item is | Afterwards |
|---|---|---|---|
| `for x in &v` | `v.iter()` | `&T`: read it | `v` unchanged and usable |
| `for x in &mut v` | `v.iter_mut()` | `&mut T`: change it in place | `v` changed and usable |
| `for x in v` | `v.into_iter()` | `T`: owned, moved out | `v` is gone (moved) |

While a loop over `&v` runs, `v` is borrowed, so you can't push to it inside the loop (lesson 3).

## 5. Closures borrow what they use

A closure captures the variables it uses in the **least powerful way that works**:

| The closure… | It captures by | Example |
|---|---|---|
| only reads the variable | `&` shared borrow | `\|\| println!("{count}")` |
| changes the variable | `&mut` mutable borrow | `\|\| count += 1` |
| is marked `move` | taking ownership | `move \|\| greeting.to_uppercase()` |

The borrow lasts as long as **the closure** is used:

```rust
let mut increment = || count += 1;   // mutable borrow of `count` starts
println!("{count}");                 // ✗ can't read while the closure may still change it
increment();
```

```text
error[E0502]: cannot borrow `count` as immutable because it is also borrowed as mutable
```

Use `move` when the closure must outlive the current scope, for example when it's returned from a function or sent to another thread.

## 6. Taking a value out from behind a `&mut`: `mem::take`

```rust
fn take_items(order: &mut Order) -> Vec<String> {
    order.items                     // ✗
}
```

```text
error[E0507]: cannot move out of `order.items` which is behind a mutable reference
```

You only *borrowed* the order. Its owner still expects a valid `Vec` in that field when you're done. **Fix:** put something in its place while taking the old value out:

- `std::mem::take(&mut order.items)` takes the value and leaves the type's default (an empty `Vec`);
- `std::mem::replace(&mut x, new)` takes the value and leaves `new`;
- `option.take()` takes the value out of an `Option`, leaving `None`.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 5: Lifetimes](../05-lifetimes/) · Next: [Lesson 7: Interior mutability](../07-interior-mutability/)
