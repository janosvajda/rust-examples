<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: Borrowing in practice

## The idea in one sentence

Real programs lend fields, collection elements and captured variables. These examples show how to keep the borrowed regions and their required lifetimes clear.

If two friends decorate different halves of a cake, each can work on their own half. Borrowing separate fields or using `split_at_mut` is Rust's way of making that separation explicit.

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

## 3. Two mutable parts of one collection: `split_at_mut` and `get_disjoint_mut`

```rust
let (morning, afternoon) = temperatures.split_at_mut(3);
morning[0] += afternoon[0];    // ✓ both halves mutable at once
```

Writing `&mut temperatures[..3]` and `&mut temperatures[3..]` at the same time doesn't compile, even though the ranges don't overlap. The compiler doesn't reason about index ranges; it just sees two mutable borrows of `temperatures`. `split_at_mut` is a standard-library function that *guarantees* the halves don't overlap, so it can safely give you both.

When the parts aren't neat halves, ask for exactly the positions you need with `get_disjoint_mut`:

```rust
let mut scores = [10, 20, 30, 40];
if let Ok([first, last]) = scores.get_disjoint_mut([0, 3]) {
    std::mem::swap(first, last);                // two &mut into one array at once
}
```

It checks at runtime that the indexes are different and in bounds. The same index twice gives `Err(OverlappingIndices)`, and an index past the end gives `Err(IndexOutOfBounds)`, so two `&mut` to the same element can never exist.

`HashMap` has the same method, which solves a classic problem: changing two entries of one map at once.

```rust
use std::collections::HashMap;

fn transfer(balances: &mut HashMap<&str, u32>, from: &str, to: &str, amount: u32) -> Result<(), String> {
    if from == to {
        return Err(String::from("choose two different accounts"));
    }
    let [Some(source), Some(target)] = balances.get_disjoint_mut([from, to]) else {
        return Err(format!("unknown account: `{from}` or `{to}`"));
    };
    let new_source = source.checked_sub(amount)
        .ok_or_else(|| format!("`{from}` can't pay {amount}"))?;
    let new_target = target.checked_add(amount)
        .ok_or_else(|| format!("`{to}` can't hold another {amount}"))?;
    *source = new_source;
    *target = new_target;
    Ok(())
}
```

Two simultaneous `balances.get_mut(...)` calls would conflict: both borrow the map mutably. `HashMap::get_disjoint_mut` safely obtains the separate entries together. It returns `None` for missing keys and panics for overlapping entries; our wrapper rejects equal account names before calling it.

The arithmetic checks matter too. Both new balances are calculated **before either account changes**. If Ana cannot pay, Bob's balance would overflow, or an account is missing, the function returns `Err` with balances unchanged. For example, sending one coin to Bob when he already has `u32::MAX` coins must fail safely.

## 4. Three ways to loop over a collection

| Loop | Same as | Each item is | Afterwards |
|---|---|---|---|
| `for x in &v` | `v.iter()` | `&T`: shared access | the `Vec` remains usable; ordinary elements are read-only |
| `for x in &mut v` | `v.iter_mut()` | `&mut T`: may change it in place | `v` remains usable |
| `for x in v` | `v.into_iter()` | `T`: owned, moved out | a non-`Copy` `Vec` has moved |

This table describes a `Vec<T>`. Other collection types can behave differently; an array of `Copy` elements can itself be copied. Elements with interior mutability can also change through shared access (lesson 7). While an iterator still needs its borrow of the `Vec`, you cannot push to that vector.

## 5. Closures borrow what they use

A closure captures the variables it uses in the **least powerful way that works**:

| The closure… | It captures by | Example |
|---|---|---|
| only reads the variable | `&` shared borrow | `\|\| println!("{count}")` |
| changes the variable | `&mut` mutable borrow | `\|\| count += 1` |
| moves a non-`Copy` value out | taking ownership, even without `move` | `\|\| drop(greeting)` |
| is marked `move` | by value: moves or copies | `move \|\| greeting.to_uppercase()` |

A captured borrow must stay valid for every use that requires it. This can include dropping captured values. In the simple closure below, later calling `increment` keeps the mutable borrow active:

```rust
let mut increment = || count += 1;   // mutable borrow of `count` starts
println!("{count}");                 // ✗ can't read while the closure may still change it
increment();
```

```text
error[E0502]: cannot borrow `count` as immutable because it is also borrowed as mutable
```

Use `move` to capture by value when a closure must own its local data, for example when returning it or passing it to an unscoped thread. Capturing an existing reference by value still captures a reference: `move` does not extend the lifetime of the data it points to.

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

Library details: [slice disjoint borrows](https://doc.rust-lang.org/std/primitive.slice.html#method.get_disjoint_mut) and [HashMap disjoint entries](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.get_disjoint_mut).

Previous: [Lesson 5: Lifetimes](../05-lifetimes/) · Next: [Lesson 7: Interior mutability](../07-interior-mutability/)
