# Raw Pointer Dereference in Rust

This project demonstrates how to safely dereference a raw pointer in Rust.

## Code Structure

The application contains a function and the `main`:

1. **get_value_from_raw_pointer Function:** This function takes a raw pointer of `u32` type as an argument and returns the dereferenced `u32` value. The dereferencing operation is done within an `unsafe` block to handle the potential undefined behavior.

2. **main Function:** This function creates a raw pointer to a `u32` value and calls the `get_value_from_raw_pointer` function to get the `u32` value. It then prints the value if it equals 99.

## Running the Application

To compile and run the application, use the following command in your terminal at the root directory of the project:

```bash
cargo run
```

## Running Tests

The code includes a test for the `get_value_from_raw_pointer` function, checking that it correctly dereferences a raw pointer.

To run the test, use the following command:

```bash
cargo test
```

The `cargo test` command will compile the code in test mode and run the tests, displaying the results in your terminal.
