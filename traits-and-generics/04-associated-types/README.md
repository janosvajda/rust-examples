<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Associated types and constants

## The idea in one sentence

A trait can also contain **types** and **constants** that each implementor chooses, and knowing when to use an associated type instead of a generic parameter is the key decision.

## Associated types

```rust
trait Container {
    type Item;                                     // each implementor picks this
    fn get(&self, index: usize) -> Option<&Self::Item>;
}

impl Container for Shelf       { type Item = String; … }
impl Container for Thermometer { type Item = f64;    … }
```

`type Item;` is a placeholder for a type that each implementor fills in, **exactly once**. A `Shelf` holds `String`s, full stop. Inside the trait, `Self::Item` means "whatever type this implementor chose".

### You already use one: `Iterator::Item`

```rust
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    // …plus dozens of default methods: map, filter, sum, collect…
}

impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> { … }
}
```

Because `Item` is fixed for `Countdown`, every iterator method automatically knows it's dealing with `u32`s. `Countdown(5).sum::<u32>()` and `.collect::<Vec<_>>()` just work.

## Associated type or generic parameter?

The two can look similar, but they say different things:

| | Associated type `trait Container { type Item; }` | Generic parameter `trait ConvertTo<T>` |
|---|---|---|
| Implementations per type | **one**: a `Shelf` has exactly one `Item` | **many**: `Celsius` implements `ConvertTo<Fahrenheit>` **and** `ConvertTo<Kelvin>` |
| Who picks the type | the implementor, once | the caller, at each use (`let f: Fahrenheit = c.convert();`) |
| Use when | there's one natural answer per type | a type can sensibly do it in several ways |

```rust
let today = Celsius(25.0);
let f: Fahrenheit = today.convert();   // picks ConvertTo<Fahrenheit>
let k: Kelvin     = today.convert();   // picks ConvertTo<Kelvin>
```

The standard library uses both. `Iterator` has an associated `Item`, because an iterator produces one kind of thing. `From<T>` is generic, because `String` can be created *from* `&str`, from `char`, from `Box<str>`, and more. `Add<Rhs>` uses **both**: a generic right-hand side and an associated `Output` (lesson 5).

## Associated constants

```rust
trait Vehicle {
    const WHEELS: u32;
    const MAX_SPEED_KMH: u32;
}

impl Vehicle for Bicycle { const WHEELS: u32 = 2; const MAX_SPEED_KMH: u32 = 40; }
impl Vehicle for Car     { const WHEELS: u32 = 4; const MAX_SPEED_KMH: u32 = 180; }
```

Each implementor provides its own fixed value, known at compile time. They're used through the type, not a value: `Car::WHEELS`, or `V::WHEELS` in generic code like `total_wheels::<Car>(3)`.

Typical uses: the size of a fixed block, a version number, a default limit. For example, every integer type has `u32::MAX`, `i64::MIN` and so on.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: Trait objects](../03-trait-objects/) · Next: [Lesson 5: The standard traits](../05-standard-traits/)
