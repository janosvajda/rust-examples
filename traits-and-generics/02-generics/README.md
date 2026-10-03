<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Generics

## The idea in one sentence

**Generics** let you write a function or type once and use it with many types. `T` is a placeholder for "some type", and **trait bounds** say what that type must be able to do.

## A generic function

```rust
fn largest<T: PartialOrd>(items: &[T]) -> Option<&T>
```

Read it as: *"for any type `T` that can be compared with `<` and `>`, take a slice of `T`s and maybe return a reference to one of them."* The same function then works for numbers, floats, characters and strings. The demo calls it with all four.

### Why the bound is needed

Without `: PartialOrd`, this doesn't compile:

```text
error[E0369]: binary operation `>` cannot be applied to type `&T`
```

The compiler checks generic code **once, for every possible `T`**, not just for the types you happen to call it with. Without a bound, `T` could be a type with no `>` at all, so the compiler refuses. The bound is a promise: "only types with `>` allowed", and it lets you use `>`. The same applies to everything else, for example printing:

```text
error[E0277]: `T` doesn't implement `std::fmt::Display`
```

That's fixed with `T: Display`.

## Writing bounds

```rust
fn describe<T: Display + PartialOrd>(a: T, b: T) -> String   // `+` combines bounds

fn describe<T>(a: T, b: T) -> String                         // the same, with `where`
where
    T: Display + PartialOrd,
```

`where` means exactly the same thing. It just keeps long signatures readable. `impl Trait` in a parameter (lesson 1) is a shorthand for a simple bound: `fn f(x: impl Display)` is `fn f<T: Display>(x: T)`.

## Generic structs, enums and methods

```rust
struct Pair<T> { first: T, second: T }

impl<T> Pair<T> {                          // for EVERY Pair<T>
    fn swap(self) -> Pair<T> { … }
}

impl<T: PartialOrd + Display> Pair<T> {    // ONLY when T can be compared and printed
    fn bigger(&self) -> &T { … }
}
```

A method block can have its own, stricter bounds. `Pair<Vec<i32>>` is a perfectly valid type, it just doesn't get a `bigger()` method, because `Vec` isn't `Display`.

You've been using generic enums all along: `Option<T>` and `Result<T, E>` are generic enums from the standard library. The demo's `Measurement<T>` works the same way.

## Several type parameters

```rust
fn lookup<K: PartialEq, V>(pairs: &[(K, V)], key: &K) -> Option<&V>
```

`K` and `V` are independent: keys can be country names while values are cities, or keys can be port numbers while values are labels. Each parameter only needs the bounds its own code uses. `V` needs none, because `lookup` never looks inside a value.

## How it works: monomorphisation

For every concrete type a generic function is called with, the compiler generates a **separate copy** specialised for that type: `largest::<i32>`, `largest::<f64>`, `largest::<char>`, and so on. This is called **monomorphisation**.

- **Generics cost nothing at runtime.** Each copy is as fast as if you'd written it by hand for that type.
- **The trade-off is compile time and program size:** more types means more copies. Lesson 3 shows the alternative, trait objects, which use one copy and decide at runtime instead.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: Traits](../01-traits/) · Next: [Lesson 3: Trait objects](../03-trait-objects/)
