# no_std Rust Application

This project demonstrates a `no_std` Rust application that leverages the `libc` crate for interactions with the C standard library. The main functionality of the application is to search for a specified substring within a given string and print whether or not the substring was found.

## Code Structure

The code is structured into one module and a primary function:

1. **no_std_specific Module:** This module is comprised of the `main` function and the `panic_handler` for non-test environments. The `main` function invokes the `call_strstr` function to determine if a specific substring is present within a string and prints the outcome accordingly.

2. **call_strstr Function:** This function does not take any arguments and returns an `Option<&'static [u8]>`. It leverages the `libc::strstr` function to search for a specific substring ("World!\n\0") within a given string ("Hello World!\n\0"). If the substring is found, it returns the substring along with any characters that follow. If not found, it returns `None`.

## Running the Application

In order to compile and run the application, use the following command within the root directory of your project:

```bash
cargo run
```

## Running Tests

The code also includes tests that are runnable within the standard Rust testing environment. These tests are housed within the `tests` module.

To run these tests, use the following command:

```bash
cargo test
```

This command compiles the code in test mode and runs all the tests, displaying the results in the terminal.
