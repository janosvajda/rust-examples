<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 9: Borrowing in patterns

## The idea in one sentence

`match`, `if let`, `let` destructuring and closure parameters can either **move** values out or **borrow** them, and **what you match on decides which**.

## Match on a value: it moves

```rust
let nickname: Option<String> = Some(String::from("Ferris"));
match nickname {
    Some(name) => …,     // `name` is a String, moved out of `nickname`
    None => {}
}
println!("{nickname:?}");  // ✗
```

```text
error[E0382]: borrow of partially moved value: `nickname`
```

The `String` inside the `Option` has moved into `name`. *Partially moved* means part of the value was taken out, so the whole can't be used any more.

## Match on a reference: it borrows

```rust
match &nickname {
    Some(name) => …,     // `name` is a &String
    None => {}
}
println!("{nickname:?}");  // ✓ still ours
```

When you match a **reference** against a pattern that isn't a reference (`Some(name)`), Rust makes every binding inside a reference too. This is called **match ergonomics**:

| You match on | `Some(name)` gives `name: …` |
|---|---|
| `Option<String>` | `String` (moved) |
| `&Option<String>` | `&String` (borrowed) |
| `&mut Option<String>` | `&mut String` (borrowed, can change it) |

```rust
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
fn display_name(user: &User) -> &str {
    user.nickname.as_deref().unwrap_or(&user.name)
}
```

## `ref` and `ref mut`

Before match ergonomics existed, you asked for a borrow inside a pattern with `ref`. It's still useful when you match on a value but want to borrow **part** of it:

```rust
let pair = (String::from("key"), 42);
let (ref key, value) = pair;   // borrow the String, copy the number
println!("{pair:?}");          // ✓ pair is still whole
```

## Partial moves out of structs

```rust
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

Moving `names[0]` out would leave a hole in the middle of the `Vec`. Choose instead:
- `&names[0]` to borrow it;
- `names[0].clone()` to copy it;
- `names.remove(0)` or `names.swap_remove(0)` to really take it out;
- `std::mem::take(&mut names[0])` to take it and leave an empty `String` behind.

**Anything behind a reference:**

```text
error[E0507]: cannot move out of `*shared` which is behind a shared reference
```

Calling `.unwrap()` on a `&Option<String>` would move the `String` out of a value you only borrowed. Use `.as_ref().unwrap()` to get a `&String` instead.

## Patterns in closures and loops

```rust
numbers.iter().filter(|&&n| n > 2)   // filter gives `&&i32`; `&&n` copies the number out
numbers.iter().map(|&n| n * 10)      // map gives `&i32`; `&n` copies it out
for (index, &n) in numbers.iter().enumerate() { … }
```

A `&` in a **pattern** does the opposite of `&` in an expression: it *removes* a layer of reference. It only works for `Copy` types, because it copies the value out.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 8: Threads and borrowing](../08-threads-and-borrowing/) · Next: [Lesson 10: Advanced lifetimes](../10-advanced-lifetimes/)
