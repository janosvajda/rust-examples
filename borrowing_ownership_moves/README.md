# Rust Ownership Moves Example

This example shows the difference between **copying** and **moving** a value in Rust.

## Example Description

1. **Copy types:** A stack-allocated `u32` is copied when it is assigned to another variable, so both `x` and `y` can still be used afterwards.

2. **Move semantics:** A heap-allocated `Box<i32>` is *moved* when it is assigned to another variable. After `let b = a;` the original variable `a` can no longer be used.

3. **Moving into a function:** Passing `b` to `destroy_box` transfers ownership to the function. The box is dropped and its memory freed when the function returns, so `b` cannot be used afterwards either.

The lines that would not compile are left in the source as comments. Uncomment them to see the compiler's "borrow of moved value" errors.

## Run the example

```bash
cargo run
```
