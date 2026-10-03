<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Traits and Generics

Traits and generics are how Rust lets you write code that works with many types, **without** inheritance, and usually without any runtime cost. Traits describe what a type can do; generics let code work with any type that can do it.

Every compiler error quoted in these READMEs is the real output of Rust 1.99.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [Traits](01-traits/) | defining and implementing traits, default methods, `impl Trait`, `derive` |
| 2 | [Generics](02-generics/) | generic functions, structs and enums; trait bounds and `where`; monomorphisation |
| 3 | [Trait objects](03-trait-objects/) | `dyn Trait`, vtables, static vs dynamic dispatch, dyn compatibility |
| 4 | [Associated types and constants](04-associated-types/) | `type Item`, `const`, and when to use a generic parameter instead |
| 5 | [The standard traits](05-standard-traits/) | `Debug`, `Display`, `Clone`, `Copy`, `Eq`, `Ord`, `Hash`, `Default`, `From`, `TryFrom`, operators |
| 6 | [Advanced traits](06-advanced-traits/) | supertraits, blanket impls, the orphan rule, newtypes, extension traits, disambiguation |

## The errors you'll meet

| Error | In plain words | Lesson |
|---|---|---|
| **E0046** not all trait items implemented | you forgot a required method | 1 |
| **E0277** the trait bound `X: Trait` is not satisfied | this type doesn't implement the trait you need | 1, 5 |
| **E0369** binary operation cannot be applied to type `T` | add a bound like `T: PartialOrd` | 2 |
| **E0277** `T` doesn't implement `Display` | add `T: Display` | 2 |
| **E0038** the trait is not dyn compatible | a method returns `Self` or is generic | 3 |
| **E0204** the trait `Copy` cannot be implemented | a field owns heap data | 5 |
| **E0117** only traits defined in the current crate… | the orphan rule: use a newtype | 6 |
| **E0034** multiple applicable items in scope | name the trait: `Trait::method(&x)` | 6 |

## Run a lesson

```bash
cd traits-and-generics/03-trait-objects
cargo run
cargo test
```

Or from the repository root: `cargo run -p trait-objects`. The package names are in each lesson's `Cargo.toml`.
