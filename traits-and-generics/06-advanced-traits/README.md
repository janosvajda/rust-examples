<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: Advanced traits

## The idea in one sentence

Traits can build on other traits, be implemented for whole families of types at once, and extend types you don't own, within one important rule about who may implement what.

## 1. Supertraits: "to be an X, you must also be a Y"

```rust
trait Printable: Display {
    fn print_framed(&self) -> String {
        let text = self.to_string();     // allowed: every Printable is also Display
        …
    }
}
```

`trait Printable: Display` means **every type implementing `Printable` must also implement `Display`**. In return, `Printable`'s methods can use `Display`'s. It's not inheritance of data, only a requirement. The standard library does this a lot: `Eq: PartialEq`, `Ord: Eq + PartialOrd`, `Copy: Clone`.

## 2. Blanket implementations: one `impl` for many types

```rust
impl<T: Display> Shout for T {
    fn shout(&self) -> String { … }
}
```

This implements `Shout` for **every type that implements `Display`**: `&str`, numbers, your own `Badge`, all at once. It's exactly how the standard library gives every `Display` type a `.to_string()` method: there's one blanket `impl<T: Display> ToString for T`.

## 3. The orphan rule

You may implement a trait for a type only if **the trait or the type is defined in your crate**:

| Trait | Type | Allowed? |
|---|---|---|
| yours | yours | ✓ |
| yours (`StrExt`) | someone else's (`str`) | ✓ |
| someone else's (`Display`) | yours (`Badge`) | ✓ |
| someone else's (`Display`) | someone else's (`Vec<i32>`) | ✗ |

```text
error[E0117]: only traits defined in the current crate can be implemented for types defined outside of the crate
```

**Why:** if two crates could both implement `Display` for `Vec<i32>`, a program using both would have two conflicting implementations and no way to choose. The rule guarantees there's at most one.

**The workaround, a newtype:** wrap the foreign type in your own struct, which is now *your* type:

```rust
struct CommaList(Vec<i32>);
impl Display for CommaList { … }     // ✓ CommaList is yours
```

A newtype costs nothing at runtime: it has exactly the same size and layout as the `Vec` inside it.

## 4. Extension traits: new methods for types you don't own

```rust
trait StrExt {
    fn word_count(&self) -> usize;
    fn is_shouting(&self) -> bool;
}
impl StrExt for str { … }

"Rust is fun".word_count()       // 3, once StrExt is in scope
```

Your own trait, implemented for a standard type. That's allowed by the orphan rule. The new methods are only available where the trait is imported with `use`, so they can't surprise other code. Many crates use this, for example `itertools` adds dozens of methods to every iterator this way.

## 5. Two traits with the same method name

```rust
person.fly()                       // the type's own method (impl Person) wins
Pilot::fly(&person)                // pick a trait explicitly
<Person as Wizard>::fly(&person)   // fully qualified: type AND trait
```

If a type implements two traits that both have `fly`, and has no `fly` of its own, `person.fly()` is ambiguous:

```text
error[E0034]: multiple applicable items in scope
```

Name the trait to say which one you mean. The fully qualified form `<Type as Trait>::method` is needed when even the trait name isn't enough, for example for associated functions without `self`.

## The course in one table

| You want to… | Use |
|---|---|
| describe what types can do | a trait (lesson 1) |
| write code for any type that can do it | generics with bounds (lesson 2) |
| mix different types at runtime | `dyn Trait` (lesson 3) |
| let each implementor pick a type or constant | associated types / consts (lesson 4) |
| make your type print, compare, hash, convert, add | the standard traits (lesson 5) |
| require one trait for another | a supertrait |
| implement a trait for many types at once | a blanket `impl<T: …>` |
| implement a foreign trait for a foreign type | a newtype |
| add methods to a foreign type | an extension trait |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 5: The standard traits](../05-standard-traits/) · Back to the [course overview](../)
