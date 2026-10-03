<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: References

## The idea in one sentence

A **reference** lets you use a value **without taking ownership** of it. Making a reference is called **borrowing**.

## Why we need it

In lesson 1, passing a `String` to a function moved it away for good. But most functions only need to *look at* a value, or change it a little, and then give it back. A reference is a way to lend the value instead of giving it away.

```text
  name ──owns──► "Ferris the crab"
                       ▲
  count_vowels(&name)  │  borrows it: can read it, doesn't own it,
                       │  and the borrow ends when the function returns
```

## The two kinds of reference

| | Written | Can read | Can change | How many at once |
|---|---|---|---|---|
| **Shared reference** | `&value` | ✓ | ✗ | as many as you like |
| **Mutable reference** | `&mut value` | ✓ | ✓ | only one (lesson 3) |

```rust
fn count_vowels(text: &String) -> usize { ... }   // borrows to read
fn double(number: &mut i32) { *number *= 2; }      // borrows to change

let name = String::from("Ferris");
count_vowels(&name);        // lend it; `name` is still ours afterwards

let mut score = 21;
double(&mut score);         // score is now 42
```

- **`&` in a type** (`&String`) means "a reference to a String". **`&` in an expression** (`&name`) means "borrow `name`".
- **`*` follows a reference** to the value it points at (*dereferencing*). `*number *= 2` changes the number, not the reference. Rust adds `*` automatically in many places, like method calls (`text.chars()`) and printing.
- **To lend something mutably, the owner must be declared `mut`:**

  ```text
  error[E0596]: cannot borrow `fixed` as mutable, as it is not declared as mutable
  ```

- **A shared reference is read-only.** You can't change anything through it:

  ```text
  error[E0596]: cannot borrow `*name` as mutable, as it is behind a `&` reference
  ```

## Methods: `&self`, `&mut self`, `self`

A method's first parameter says how it uses the value it's called on:

| Method | Means | Example |
|---|---|---|
| `fn summary(&self)` | borrows to **read** | `book.summary()` |
| `fn rename(&mut self, …)` | borrows to **change** | `book.rename("Animal Farm")` |
| `fn into_title(self)` | **takes ownership**; the caller loses the value | `let title = book.into_title();` |

You don't write `&book` or `&mut book` when calling a method: Rust borrows automatically, in the way the method asks for. After `into_title`, `book` has been moved:

```text
error[E0382]: borrow of moved value: `book`
```

By convention, methods that take `self` and turn the value into something else are named `into_…`.

## Facts worth knowing precisely

- **A reference is a pointer** to the original value, not a copy of it. A test checks that the reference's address is the value's address.
- **References are never null.** In safe Rust, a reference always points at a valid value of the right type. Lesson 5 shows how the compiler guarantees the value is still alive.
- **Shared references are `Copy`.** `let r3 = r1;` just makes another reference to the same value.
- **Mutable references are not `Copy`.** There may only ever be one usable at a time, so `let b = a;` *moves* a `&mut`.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: Ownership and moves](../01-ownership-and-moves/) · Next: [Lesson 3: The borrowing rules](../03-borrowing-rules/)
