# Fibonacci Calculation example

This code calculates Fibonacci numbers using both imperative and declarative paradigms.

## What is Fibonacci?

The Fibonacci sequence is a series of numbers in which each number is the sum of the two preceding ones. It typically starts with 0 and 1.

## Paradigms

### Imperative way

The imperative way uses a recursive function to calculate Fibonacci numbers. It checks the input value and returns the corresponding Fibonacci number using conditional statements and function calls.

\```rust
fn fibonacci(n: u32) -> u32 {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
\```

### Declarative way

The declarative way uses the `fold` function to calculate Fibonacci numbers. It operates on a range of numbers and applies a closure repeatedly to calculate the Fibonacci sequence. The `fold` function accumulates the result by passing the previous two Fibonacci numbers and returning the next one.

```rust
(0..=10).for_each(|i| println!("Fibonacci({}) = {}", i, if i < 2 { i } else { (2..=i).fold((0, 1), |(a, b), _| (b, a + b)).1 }));
```

## Running the Code

To run the code, follow these steps:

1. Ensure you have Rust installed on your system.
2. Clone this repository or copy the code into a new Rust project.
3. Open a terminal and navigate to the project directory.
4. Run the following command to compile and execute the code:

    ```
   cargo run
    ```

5. The program will calculate and display the Fibonacci numbers for the range defined in the code.

## Running the Tests

This code includes unit tests to verify the correctness of the Fibonacci calculation. To run the tests, follow these steps:

1. Open a terminal and navigate to the project directory.
2. Run the following command:

   ```
   cargo test
   ```

3. The tests will execute, and the results will be displayed in the terminal.
