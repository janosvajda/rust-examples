<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: `Box<T>`

## The idea in one sentence

`Box::new(value)` puts a value on the heap and gives you its single owner: a pointer that's always the same small size, used just like the value itself, and freed automatically when it goes out of scope.

## What a Box is

```rust
let boxed = Box::new(41);
println!("{}", *boxed + 1);        // 42: `*` reaches the value inside
```

A `Box<T>` is one pointer on the stack, 8 bytes on a 64-bit machine, pointing at a `T` on the heap. It **owns** that value. There's exactly one owner, and when the `Box` is dropped, the value is dropped and its heap memory freed. No `free()` to forget, and no way to free twice.

You'll need a `Box` in three situations.

## 1. Recursive types

An expression like `(2 + 3) * 4` is a tree: a multiplication whose left side is an addition. Written without a `Box`:

```rust
enum Expr { Num(i64), Add(Expr, Expr) }
```

```text
error[E0072]: recursive type `Expr` has infinite size
```

To know an `Expr`'s size, Rust needs the size of the `Expr`s inside it, which need the size of the `Expr`s inside *them*, forever. A `Box` breaks the loop: whatever it points to, a `Box` itself is one pointer.

```rust
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}
```

```text
Mul(Add(Num(2), Num(3)), Num(4)) = 20
```

Linked lists and trees are built this way, as in the [linked list](../../data-structures/linear/linked-list/) and [binary search tree](../../data-structures/trees/binary-search-tree/) examples.

## 2. Different types in one list

A `Circle` and a `Rectangle` have different sizes, so a `Vec` can't hold both directly. Boxed and seen through a trait, they all have the same size:

```rust
let shapes: Vec<Box<dyn Shape>> = vec![
    Box::new(Circle { radius: 1.0 }),
    Box::new(Rectangle { width: 2.0, height: 3.0 }),
];
```

Each `Box<dyn Shape>` is a pointer to the data, plus a pointer to the right `area` and `name` functions for that type. [Traits lesson 3](../../traits-and-generics/03-trait-objects/) explains trait objects in detail.

## 3. Big values

```text
size of Image:      1000000 bytes
size of Box<Image>: 8 bytes
```

Passing a 1 MB `Image` by value copies a megabyte. Passing a `Box<Image>` copies 8 bytes. A `Box` also keeps big values off the stack, which is small ([lesson 1](../01-stack-and-heap/)).

One surprise: `Box::new(value)` **first builds the value, usually on the stack, and then moves it** to the heap. For one megabyte that's fine. For a value too big for the stack, build it on the heap from the start, for example with `vec![0u8; n].into_boxed_slice()`.

## When you don't need a Box

Most of the time you don't: `String`, `Vec` and `HashMap` already keep their contents on the heap. Reach for `Box` when you need one of the three situations above. And when **several** parts of the program need to own the same value, one owner isn't enough. That's what the next lesson is about.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: The stack and the heap](../01-stack-and-heap/) · Next: [Lesson 3: Rc and Weak](../03-rc-and-weak/)
