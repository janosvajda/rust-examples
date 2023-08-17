# Rust Borrowing Example

This repository contains a simple Rust example that demonstrates the concept of borrowing in Rust. Borrowing is a fundamental concept in Rust's ownership system that allows you to work with data without transferring ownership. Instead of copying data, borrowing enables efficient and safe sharing of data between different parts of your code.

## Example Description

The example in this repository uses a `Book` struct to illustrate how borrowing works. The `Book` struct has three fields: `title`, `author`, and `pages`. The example demonstrates the following:

1. **Creating a Book:** It shows how to create a `Book` instance using the `new` constructor and initialize its fields.
   
2. **Immutable Borrowing:** It demonstrates how to borrow a `Book` instance immutably using the `print_info` method, which prints the book's information without changing its content.
   
3. **Mutable Borrowing:** It showcases how to borrow a `Book` instance mutably using the `change_title` method. This method allows you to change the title of the book without transferring ownership.


4. Run the example:

cargo run

5. To run the included tests:

cargo test
