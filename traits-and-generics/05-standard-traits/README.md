<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: The standard traits

## The idea in one sentence

A small set of traits from the standard library connects your types to the rest of Rust. Implement or derive them, and printing, comparing, sorting, hashing, converting and even `+` work on your types.

## The cheat sheet

| Trait | Gives you | Derive? |
|---|---|---|
| `Debug` | `{:?}` printing, for developers | ✓ almost always |
| `Display` | `{}` printing and `.to_string()`, for users | ✗ write it yourself |
| `Clone` | `.clone()`, an explicit copy | ✓ |
| `Copy` | implicit copies on assignment (ownership lesson 1) | ✓ only if every field is `Copy` |
| `PartialEq`, `Eq` | `==` and `!=` | ✓ |
| `PartialOrd`, `Ord` | `<`, `>`, `.sort()`, `BTreeMap` keys | ✓ compares fields in order |
| `Hash` | `HashMap` / `HashSet` keys (together with `Eq`) | ✓ |
| `Default` | `Type::default()`, a starting value | ✓ or write your own |
| `From` / `Into` | infallible conversions | ✗ |
| `TryFrom` / `TryInto` | conversions that can fail | ✗ |
| `Add`, `Sub`, `Mul`, `Neg`, `Index`… | operators: `+`, `-`, `*`, unary `-`, `[]` | ✗ |

## Printing: `Debug` vs `Display`

```text
Debug:   Money { cents: 1999 }      ← derived, shows the structure
Display: €19.99                     ← written by hand, shows what a person expects
```

Derive `Debug` on nearly every type, since it makes debugging and tests easier. Write `Display` only for types that have a natural human-readable form. Implementing `Display` also gives you `.to_string()`.

## Operators are traits

`a + b` is literally `a.add(b)` from the `Add` trait:

```rust
impl Add for Money {
    type Output = Money;                       // an associated type (lesson 4)
    fn add(self, other: Money) -> Money { … }
}
impl Mul<i64> for Money { … }                  // the right-hand side can be another type
```

Now `price * 3 + Money::from(5)` works. Overload operators only where the meaning is obvious, like adding money or multiplying vectors.

## Conversions: `From`, `Into`, `TryFrom`

```rust
impl From<i64> for Money { … }       // Money::from(5)
let ten: Money = 10.into();          // Into comes for free, in the other direction

impl TryFrom<&str> for Money {       // can fail, so it returns a Result
    type Error = String;
    …
}
```

**Implement `From`, never `Into`:** the standard library automatically provides `Into` for every `From`. `From` is also what `?` uses to convert errors (error-handling course, lesson 3).

## Comparing, ordering, hashing

```rust
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Task { priority: u8, title: String }
```

Derived comparisons look at the fields **in the order they're declared**, so tasks sort by `priority` first, then `title`. Reorder the fields and you change the sort order.

Why both `PartialEq` and `Eq`, `PartialOrd` and `Ord`? The "partial" versions allow values that aren't comparable. The classic case is floating point: `NaN` isn't equal to anything, not even itself, so `f64` is only `PartialEq`:

```text
error[E0277]: the trait bound `f64: Eq` is not satisfied
```

`HashMap` keys need `Hash` **and** `Eq`, because the hash finds the bucket and `Eq` confirms the match:

```text
error[E0277]: the trait bound `Id: Hash` is not satisfied
```

## `Copy` has a strict rule

A type can only be `Copy` if copying its bytes is a complete copy, which means nothing owned on the heap:

```text
#[derive(Clone, Copy)] struct User { name: String }
error[E0204]: the trait `Copy` cannot be implemented for this type
```

`Money { cents: i64 }` can be `Copy`, and that's why the demo can use `price` several times without cloning.

## `Default` and struct update syntax

```rust
let settings = Settings { volume: 80, ..Default::default() };
```

`..Default::default()` fills every field you didn't mention with its default. That's handy for configuration structs with many fields.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 4: Associated types and constants](../04-associated-types/) · Next: [Lesson 6: Advanced traits](../06-advanced-traits/)
