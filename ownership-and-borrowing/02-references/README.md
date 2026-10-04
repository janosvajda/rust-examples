<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: References

## The idea in one sentence

A **reference** lets you use a value **without taking ownership** of it. Making a reference is called **borrowing**.

## Why we need it

In lesson 1, passing a `String` by value transferred ownership to the function. But many functions only need to *look at* a value or change it a little. A reference is a way to lend the value instead of giving it away. Think of letting Ana read your book while you keep ownership of it.

```text
  name ──owns──► "Ferris the crab"
                       ▲
  count_vowels(&name)  │  borrows it: can read it, doesn't own it,
                       │  and the borrow ends when the function returns
```

## The two kinds of reference

| | Written | Can read | Can change | How many at once |
|---|---|---|---|---|
| **Shared reference** | `&value` | ✓ | no direct changes to ordinary data | as many as you like |
| **Mutable reference** | `&mut value` | ✓ | ✓ | only one (lesson 3) |

```rust
fn count_vowels(text: &String) -> usize {
    text.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
} // counts the listed English vowel letters
fn double(number: &mut i32) { *number *= 2; }      // borrows to change

let name = String::from("Ferris");
count_vowels(&name);        // lend it; `name` is still ours afterwards

let mut score = 21;
double(&mut score);         // score is now 42
```

- **`&` in a type** (`&String`) means "a reference to a String". **`&` in an expression** (`&name`) means "borrow `name`".
- **`*` follows a reference** to the value it points at (*dereferencing*). `*number *= 2` changes the number, not the reference. Method calls such as `text.chars()` can automatically follow references. Formatting traits also let you print referenced values directly.
- **To borrow a directly owned local variable mutably, declare that binding `mut`:**

  ```text
  error[E0596]: cannot borrow `fixed` as mutable, as it is not declared as mutable
  ```

- **A shared reference cannot directly change ordinary data.** For example, `name.push_str("!")` cannot change a `String` through `&String`:

  ```text
  error[E0596]: cannot borrow `*name` as mutable, as it is behind a `&` reference
  ```

An immutable binding can still hold a mutable reference. Here, `mut` on `r` would control replacing the reference; `&mut` already gives permission to change its pointee:

```rust
let mut number = 1;
let r = &mut number; // `r` itself is not declared `mut`
*r += 1;
assert_eq!(number, 2);
```

Types such as `Cell` and `RefCell` provide **interior mutability**: controlled changes through shared references. Lesson 7 explains them. The table above describes ordinary data such as `String` and `i32`.

## Methods: `&self`, `&mut self`, `self`

A method's first parameter says how it uses the value it's called on:

| Method | Means | Example |
|---|---|---|
| `fn summary(&self)` | borrows to **read** | `book.summary()` |
| `fn rename(&mut self, …)` | borrows to **change** | `book.rename("Animal Farm")` |
| `fn into_title(self)` | **takes by value**; this non-`Copy` book moves | `let title = book.into_title();` |

You don't write `&book` or `&mut book` when calling a method: Rust borrows automatically, in the way the method asks for. After `into_title`, `book` has been moved:

```text
error[E0382]: borrow of moved value: `book`
```

By convention, methods that take `self` and turn the value into something else are named `into_…`.

## Facts worth knowing precisely

- **A reference is a pointer** to the original value, not a copy of it. A test checks that the reference's address is the value's address.
- **References are never null.** In safe Rust, a reference always points at a valid value of the right type. Lesson 5 shows how the compiler guarantees the value is still alive.
- **Shared references are `Copy`.** `let r3 = r1;` just makes another reference to the same value.
- **Mutable references are not `Copy`.** `let b = a;` moves an inferred `&mut` binding. A reborrow instead temporarily lends access and limits the original reference while the new borrow is active; lesson 3 shows how.

## Run it

```bash
cargo run
cargo test
```

The [Rust Book's reference chapter](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) explains ordinary borrowing. [Interior mutability](https://doc.rust-lang.org/std/cell/index.html) explains controlled changes through `&`.

Previous: [Lesson 1: Ownership and moves](../01-ownership-and-moves/) · Next: [Lesson 3: The borrowing rules](../03-borrowing-rules/)
