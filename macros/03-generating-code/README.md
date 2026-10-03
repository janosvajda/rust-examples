<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Macros that generate code

## The idea in one sentence

The most useful macros write **repetitive definitions** for you: the same `impl` for many types, a family of similar structs, or a batch of test functions. You write the pattern once and stamp it out many times.

## 1. The same `impl` for many types

```rust
macro_rules! impl_describe_for_numbers {
    ( $( $t:ty ),* ) => {
        $(
            impl Describe for $t { … }
        )*
    };
}

impl_describe_for_numbers!(u8, i32, u64, f32, f64);    // five impl blocks
```

The repetition wraps a **whole `impl` block**, so one line generates five implementations. The standard library does exactly this internally: much of what's implemented for every number type is written once, inside a macro.

`stringify!($t)` turns the type's tokens into a string at compile time (`"u8"`, `"f64"`), which is handy for messages.

## 2. A family of similar types

```rust
units! {
    Metres => "m",
    Seconds => "s",
    Kilograms => "kg",
}
```

Each line generates a struct, a `Display` implementation and a `From<f64>` conversion. Three units cost three lines instead of about 60.

The payoff is **type safety for free**: `Metres` and `Seconds` are now different types, so `speed(time, distance)` with the arguments swapped doesn't compile. (This is the newtype pattern from the traits course, generated in bulk.)

## 3. Keeping things in sync

```rust
listed_enum!(Weekday { Monday, Tuesday, Wednesday, Thursday, Friday });

Weekday::ALL          // every variant, in order
Weekday::Friday.name()   // "Friday"
```

The macro generates the enum **and** a list of all its variants **and** a `name()` method. Add a variant and all three update together. Without the macro, it's easy to add a variant and forget to update the list.

## 4. Generating tests

```rust
speed_tests! {
    walking: 100.0, 80.0 => 1.25,
    running: 400.0, 50.0 => 8.0,
    standing_still: 0.0, 10.0 => 0.0,
}
```

Each line becomes its own `#[test]` function, named after the case. `cargo test` lists `walking`, `running` and `standing_still` separately, so a failure points at exactly the case that broke. A loop over test data inside one test would only say "the test failed".

## When to use a macro, and when not to

**Reach for a function or a generic first.** They're easier to read, give better error messages, and show up properly in documentation and editor tooling. Use a macro when:
- the repeated thing is a **definition** (an `impl`, a struct, a test), not a calculation;
- you need **names** made from input (`$name:ident`);
- you need a **variable number** of arguments, or a small custom syntax like `key => value`.

**Seeing what a macro expands to:** the `cargo expand` tool (installed with `cargo install cargo-expand`) prints your code with every macro expanded. It's the best way to debug a macro.

## Run it

```bash
cargo run
cargo test     # note the three separately named speed tests
```

Previous: [Lesson 2: Repetition](../02-repetition/) · Next: [Lesson 4: A derive macro](../04-derive-macro/)
