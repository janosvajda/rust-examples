# Pure Functions in Rust

## Concept of Pure Functions

In Rust, pure functions are functions that adhere to certain principles, making them predictable, easy to reason about, and free from side effects. A pure function is a function that:

1. **Deterministic**: For the same input, it always produces the same output, making it predictable and consistent.

2. **No Side Effects**: It does not modify any external state or variables outside its scope. This ensures that the function's behavior is isolated and doesn't affect the surrounding environment.

## Example: Filtering Even Numbers

Let's demonstrate the concept of pure functions using a simple example. We'll implement a pure function that takes a vector of integers and returns a new vector containing only the even numbers from the input vector. The original vector will remain unchanged, and no external state will be modified.

```rust
// This is a pure function that filters even numbers from a vector and returns a new vector.
fn filter_even_numbers(input: &Vec<i32>) -> Vec<i32> {
    let even_numbers: Vec<i32> = input.iter().cloned().filter(|&x| x % 2 == 0).collect();
    even_numbers
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Calling the pure function `filter_even_numbers` with `numbers` vector.
    let even_numbers = filter_even_numbers(&numbers);

    println!("Original vector: {:?}", numbers);
    println!("Even numbers: {:?}", even_numbers);
}
