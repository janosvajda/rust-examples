# Rust Borrowing Example

This Rust example demonstrates various aspects of borrowing in Rust, including mutable and immutable references, borrowing across function calls, borrowing struct fields, using slices, and borrowing with iterators.

## Main Code Explanation

The `main.rs` file contains the main code that demonstrates different aspects of borrowing in Rust:

- **Mutable and Immutable References in the Same Scope**: The code demonstrates the "one mutable reference XOR any number of immutable references" rule by attempting to borrow a value mutably while there's an active immutable reference.

- **Dangling References**: The code showcases the prevention of creating dangling references, which are references to data that no longer exists.

- **Borrowing Across Function Calls**: The code shows how borrowing can be used across different function calls. It demonstrates how ownership and borrowing interact with function parameters and return values.

- **Borrowing Struct Fields**: This part illustrates borrowing of individual fields within a struct. It demonstrates that you can have multiple borrows of different fields in the same struct at the same time.

- **Slices**: The code demonstrates how to create and work with slices, which are references to a contiguous sequence of elements in a collection, like an array or a vector.

- **Borrowing and Iterators**: This section showcases how borrowing works with iterators, allowing you to iterate over a collection while still being able to access the collection itself.

## Tests

The code includes tests to validate the functionality of different aspects:

- `test_borrow_across_functions`: This test validates the borrowing of mutable references across function calls, modifying the value.

- `test_borrow_struct_fields`: This test demonstrates borrowing of struct fields, modifying the title of a book.

- `test_slices`: This test confirms the functionality of slices, ensuring that the sliced elements are correct.

## Running the Code

To run the main code, navigate to the project directory and use the following command:

```bash
cargo run
