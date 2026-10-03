# Rust Closures (Anonymous Functions) Example

This example shows how to define and call closures in Rust.

## Example Description

1. **Annotated closure:** `|i: i32| -> i32 { i + 1 }` declares the parameter and return types explicitly.

2. **Inferred closure:** `|s| format!(...)` lets the compiler infer the parameter type from how the closure is called.

3. **Closure with two parameters:** Joins two string slices with a space using `[s, s2].join(" ")`.

## Run the example

```bash
cargo run
```
