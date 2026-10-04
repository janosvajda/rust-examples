<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 9: Borrowing in patterns

## The idea in one sentence

Patterns can **move, copy, borrow or ignore** parts of a value. Both the pattern and the matched type determine which happens.

Opening a parcel can mean taking a book out, inspecting it in place or ignoring it. Patterns make the same distinction.

## Binding a non-`Copy` field by value moves it

```rust
let nickname: Option<String> = Some(String::from("Ferris"));
match nickname {
    Some(name) => println!("{name}"), // `name` is a String, moved out
    None => {}
}
println!("{nickname:?}");  // ✗
```

```text
error[E0382]: borrow of partially moved value: `nickname`
```

This example **does not compile** because the `String` moves into `name`, then we try to print the whole `Option`. *Partially moved* means a field was taken out; the whole value cannot be used while that field is missing.

Merely writing `match value` does not force a move:

```rust
let text = String::from("Ferris");
match text { _ => {} } // ignores the String without moving it
println!("{text}");   // still usable
```

## Match on a reference: it borrows

```rust
let nickname = Some(String::from("Ferris"));
match &nickname {
    Some(name) => println!("{name}"), // `name` is a &String
    None => {}
}
println!("{nickname:?}");  // ✓ still ours
```

For `Some(name)`, matching a reference to the `Option` changes the default binding mode so `name` borrows the inner value automatically. This is **match ergonomics**. Explicit reference patterns and nested reference types can affect the binding mode; this table describes the particular `Some(name)` pattern:

| You match on | `Some(name)` gives `name: …` |
|---|---|
| `Option<String>` | `String` (moved) |
| `&Option<String>` | `&String` (borrowed) |
| `&mut Option<String>` | `&mut String` (borrowed, can change it) |

```rust
let mut maybe_list = Some(vec![1, 2]);
if let Some(list) = &mut maybe_list {
    list.push(3);        // changes the Vec inside the Option
}
```

## `Option` helper methods do the same thing

| Method | Turns | Into | Use it to |
|---|---|---|---|
| `.as_ref()` | `&Option<T>` | `Option<&T>` | look inside without moving |
| `.as_mut()` | `&mut Option<T>` | `Option<&mut T>` | change the inside |
| `.as_deref()` | `&Option<String>` | `Option<&str>` | get a `&str` directly |
| `.take()` | `&mut Option<T>` | `Option<T>` | move the value out, leaving `None` |

```rust
struct User { name: String, nickname: Option<String> }
fn display_name(user: &User) -> &str {
    user.nickname.as_deref().unwrap_or(&user.name)
}
```

## `ref` and `ref mut`

`ref` explicitly borrows a binding's matched value; `ref mut` requests a mutable borrow. This remains useful when matching a value and borrowing **part** of it:

```rust
let pair = (String::from("key"), 42);
let (ref key, value) = pair;   // borrow the String, copy the number
println!("{pair:?}");          // ✓ pair is still whole
```

In Rust 2024, explicit `ref` or `ref mut` must appear where the default binding mode is by value. Use `Some(name)` when matching `&nickname`; adding `ref` there is unnecessary and rejected in this edition.

## Partial moves out of structs

This example **does not compile** at the last line:

```rust
#[derive(Debug)]
struct Book { title: String, pages: u32 }
let book = Book { title: String::from("Dune"), pages: 412 };
let title = book.title;        // moves only this field out
println!("{}", book.pages);    // ✓ the other fields are still fine
println!("{book:?}");          // ✗ the struct as a whole is incomplete
```

```text
error[E0382]: borrow of partially moved value: `book`
```

Rust tracks moves **per field**. You can keep using the fields that weren't moved, but not the whole struct. (This doesn't work if the struct implements `Drop`, since dropping needs the whole value.)

## Two things you can't move out of

**An element of a `Vec`, by index:**

```text
error[E0507]: cannot move out of index of `Vec<String>`
```

Moving a non-`Copy` `String` from `names[0]` by indexing would leave a hole. Reading an `i32` this way copies it instead. For a `Vec<String>`, choose:

- `&names[0]` to borrow it;
- `names[0].clone()` to copy it;
- `names.remove(0)` or `names.swap_remove(0)` to really take it out;
- `std::mem::take(&mut names[0])` to take it and leave an empty `String` behind.

**A non-`Copy` value behind a reference, without a replacement:**

```text
error[E0507]: cannot move out of `*shared` which is behind a shared reference
```

Calling `.unwrap()` on a `&Option<String>` would move the `String` out of a value you only borrowed. Use `.as_ref().unwrap()` to get a `&String` instead.

## Patterns in closures and loops

```rust
let numbers = [1, 3, 4];
let big: Vec<i32> = numbers.iter().filter(|&&n| n > 2).copied().collect();
let tens: Vec<i32> = numbers.iter().map(|&n| n * 10).collect();
for (index, &n) in numbers.iter().enumerate() { println!("{index}: {n}"); }
assert_eq!(big, [3, 4]);
assert_eq!(tens, [10, 30, 40]);
```

`filter` passes `&&i32` to its predicate, while `map` passes `&i32`. A `&` in a **pattern** matches and removes a reference layer; the plain `n` binding then copies the `i32`.

The reference pattern itself is not limited to `Copy` types. Its inner binding may borrow:

```rust
let owned = String::from("Ferris");
let shared = &owned;
let &ref name = shared; // matches the & layer, then borrows the String
assert_eq!(name, "Ferris");
println!("{owned}");    // no String was moved out
```

## Run it

```bash
cargo run
cargo test
```

Precise rules: [patterns and binding modes](https://doc.rust-lang.org/reference/patterns.html).

Previous: [Lesson 8: Threads and borrowing](../08-threads-and-borrowing/) · Next: [Lesson 10: Advanced lifetimes](../10-advanced-lifetimes/)
