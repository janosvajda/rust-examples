<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: Traits

## The idea in one sentence

A **trait** is a set of methods a type promises to have. It describes what a type can **do**, so code can work with any type that keeps the promise.

## An everyday picture

A power socket doesn't care whether you plug in a lamp, a laptop or a kettle. It only cares that the plug has the right shape. The plug shape is the trait. The lamp, laptop and kettle are types that implement it, each doing something completely different once plugged in.

## Defining and implementing a trait

```rust
trait Summary {
    fn author(&self) -> String;                 // required: every type must write this

    fn summarise(&self) -> String {             // default: a type gets this for free…
        format!("(Read more from {}…)", self.author())
    }
}

impl Summary for Article {
    fn author(&self) -> String { self.author.clone() }
    fn summarise(&self) -> String { … }         // …or replaces it with its own version
}

impl Summary for Post {
    fn author(&self) -> String { format!("@{}", self.username) }
    // no summarise: Post uses the default
}
```

- A **required** method has no body. Forgetting it is a compile error:

  ```text
  error[E0046]: not all trait items implemented, missing: `author`
  ```

- A **default** method has a body. It can call the required methods, so a trait can offer lots of functionality while asking implementors for very little. Rust's `Iterator` asks for one method and gives you dozens.

- You can implement **your** trait for types you didn't write. The demo implements `Summary` for `String`.

## Using a trait: accept anything that implements it

```rust
fn print_feed_item(item: &impl Summary) { … }    // any type that implements Summary
fn notify<T: Summary>(item: &T) -> String { … }  // the same, in "trait bound" syntax
```

Pass a type that doesn't implement the trait and the compiler stops you before the program runs:

```text
error[E0277]: the trait bound `Tag: Summary` is not satisfied
```

## Returning a trait: `-> impl Summary`

```rust
fn featured_post() -> impl Summary { Post { … } }
```

The caller knows it gets "something that implements `Summary`" and can call its methods, without knowing it's a `Post`. You can change the concrete type later without breaking any callers. (Every `return` in the function must give back the **same** concrete type. To return different types, see trait objects in lesson 3.)

## `derive`: let the compiler write it

```rust
#[derive(Debug, Clone, PartialEq)]
struct Tag { name: String }
```

For common standard traits, `#[derive(…)]` generates a sensible implementation automatically: `Debug` prints all fields, `Clone` clones all fields, `PartialEq` compares all fields. Lesson 5 covers the standard traits in detail.

## Traits compared with other languages

| If you know… | A trait is like… | Differences |
|---|---|---|
| Java / C# | an interface | can have default methods *and* be implemented for types you didn't write |
| Go | an interface | implemented explicitly with `impl`, not automatically by having the right methods |
| C++ | an abstract base class / concept | no inheritance of data; no runtime cost unless you use `dyn` (lesson 3) |

## Run it

```bash
cargo run
cargo test
```

Next: [Lesson 2: Generics](../02-generics/)
