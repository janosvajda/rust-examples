# Rust Math Example

This example demonstrates the usage of the `libm` crate to perform math calculations in Rust. It calculates the cosine of an angle and the ceiling value of the cosine result.

## Description

The example showcases how to use the `libm` crate, which provides a comprehensive set of math functions compatible with C's math library. It calculates the cosine of a given angle and then determines the ceiling value of the cosine result.

The example code demonstrates two key aspects:
- Importing and using functions from the `libm` crate
- Differentiating between the main application code and test code using conditional compilation attributes

## Usage

To run the example, use the following command:

```bash
$ cargo run
```

This will execute the `main` function, which calculates the cosine of an angle and prints the results to the console.

## Testing

The example includes a test case to validate the correctness of the cosine and ceiling calculations. To run the tests, use the following command:

```bash
$ cargo test
```