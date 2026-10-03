<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Rust Box and Arc Example

This example shows two of Rust's smart pointers: `Box<T>` and `Arc<T>`.

## Example Description

### Box<T>
A `Box` owns a value on the heap. The heap memory is freed when the `Box` goes out of scope.

1. **Recursive types:** An `Expr` enum (a tiny expression tree) can contain other `Expr` values only through a `Box`. Without one the type would have infinite size.

2. **Trait objects:** `Vec<Box<dyn Shape>>` stores a `Circle` and a `Rectangle` in the same vector, even though they are different types with different sizes.

### Arc<T>
An `Arc` (Atomically Reference Counted) lets several owners share one heap value. Cloning an `Arc` only increments a counter, and the value is dropped when the last `Arc` is dropped. Because the counter is atomic, an `Arc` can be shared between threads (its single-threaded sibling `Rc<T>` can't).

3. **Reference counting:** `Arc::strong_count` shows the counter going up when the `Arc` is cloned and down when a clone is dropped.

4. **Read-only data shared between threads:** Several threads sum different parts of one `Arc<Vec<i32>>` without copying it.

5. **Mutable data shared between threads:** `Arc<Mutex<T>>` lets several threads safely increment the same counter.

## Run the example

```bash
cargo run
```

## Run the tests

```bash
cargo test
```
