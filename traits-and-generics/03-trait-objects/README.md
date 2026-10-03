<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Trait objects

## The idea in one sentence

A **trait object** (`dyn Trait`) lets the **concrete type be chosen while the program runs**, so one collection can hold different types and one function can return different types.

## The problem generics can't solve

A `Vec<T>` holds values of **one** type. What if you want circles, squares and triangles in the same list? With generics, `T` must be decided at compile time, and it can only be one type. A trait object says *"some type that implements `Shape`; which one is decided at runtime"*:

```rust
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { radius: 1.0 }),
    Box::new(Square { side: 2.0 }),
    Box::new(Triangle { base: 3.0, height: 4.0 }),
];
for shape in &shapes {
    println!("{}", shape.describe());    // each one runs its own area()
}
```

A trait object always lives **behind a pointer**: `Box<dyn Shape>`, `&dyn Shape`, `Rc<dyn Shape>`. That's because circles and triangles have different sizes, and a pointer is always the same size.

## How it works: the vtable

```text
  &dyn Shape  (2 pointers = 16 bytes)
  ┌──────────────┬──────────────┐
  │ data pointer │ vtable ptr   │
  └──────┬───────┴──────┬───────┘
         ▼              ▼
   Square { 2.0 }    vtable for Square-as-Shape
                     ┌───────────────────────┐
                     │ name    → Square::name│
                     │ area    → Square::area│
                     │ describe→ …           │
                     │ size, alignment, drop │
                     └───────────────────────┘
```

A trait object is a **fat pointer**: one pointer to the data, and one to a **vtable**, a small table of function pointers for that type's implementation of the trait. Calling `shape.area()` looks up `area` in the vtable and jumps there. The demo prints the sizes (8 bytes for `&Square`, 16 for `&dyn Shape`), and a test checks them.

## Static vs dynamic dispatch

| | Generics `<S: Shape>` | Trait objects `&dyn Shape` |
|---|---|---|
| Type decided | at **compile** time | at **run** time |
| Called through | a direct call (can be inlined) | the vtable (one indirect call) |
| Copies of the function | one per type (*monomorphisation*) | one for all types |
| Mixed types in one collection | ✗ | ✓ |
| Speed | fastest | very slightly slower; usually irrelevant |

**Rule of thumb:** use generics by default. Use `dyn Trait` when you need **different types together**: a list of plugins, UI widgets, shapes, or a function that picks the type from runtime input like `parse_shape("circle 2")`.

## `impl Trait` vs `Box<dyn Trait>` as a return type

```rust
fn parse_shape(text: &str) -> Option<Box<dyn Shape>> {
    match … {
        ("circle", …) => Some(Box::new(Circle { … })),
        ("square", …) => Some(Box::new(Square { … })),   // a different type: fine with dyn
        …
    }
}
```

`-> impl Shape` means *one* hidden concrete type, so every branch must return the same type. When different branches return different types, you need `Box<dyn Shape>`.

## Not every trait can be a trait object

```rust
trait Duplicate { fn duplicate(&self) -> Self; }
let items: Vec<Box<dyn Duplicate>> = …;
```

```text
error[E0038]: the trait `Duplicate` is not dyn compatible
```

A trait can be used as `dyn Trait` only if every method can be called without knowing the concrete type. Two common things break that:
- **returning `Self`**: the caller would need to know the concrete type's size;
- **generic methods** (`fn convert<T>(&self)`): there'd be no single function to put in the vtable.

The escape hatch: add `where Self: Sized` to such a method. It then can't be called on a trait object, but the rest of the trait can still be used as `dyn Trait`. That's how `Iterator` stays dyn compatible despite methods like `collect`.

(The older name for "dyn compatible" is *object safe*. You'll still see it in documentation.)

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: Generics](../02-generics/) · Next: [Lesson 4: Associated types and constants](../04-associated-types/)
