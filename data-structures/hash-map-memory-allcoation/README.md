# User Profile Memory Allocation Test in Rust by hash map

This Rust program is designed to test memory allocation and deallocation performance by creating a large number of user profile objects. Each `UserProfile` object contains fields like `name`, `age`, `email`, `hobbies`, and `attributes` to simulate a meaningful data structure. The program measures the time required to allocate and deallocate memory for a configurable number of objects.

## Features

- **Object Creation**: Generates user profiles with various fields such as name, age, email, and more.
- **Memory Allocation**: Measures the time it takes to allocate memory for a large number of user profile objects.
- **Memory Deallocation**: Measures the time it takes to deallocate the memory used by the objects.
- **Testing**: Includes unit tests to verify the correctness of profile creation and performance of memory operations.

## Structure of `UserProfile`

Each `UserProfile` object consists of:

- `name`: A `String` representing the user's name.
- `age`: A `u8` integer representing the user's age.
- `email`: A `String` representing the user's email address.
- `hobbies`: A `Vec<String>` representing the user's hobbies.
- `attributes`: A `HashMap<String, String>` storing additional user attributes, such as "Membership" and "Location".

## Code Overview

The main functionality of the program includes:

1. **Profile Creation**: The `create_user_profile` function generates a `UserProfile` object based on the input index.
2. **Memory Allocation**: In the `main` function, a large vector of `UserProfile` objects is created and populated. The time taken for allocation is measured.
3. **Memory Deallocation**: After the profiles are created, they are explicitly dropped, and the time taken for deallocation is measured.
4. **Testing**: A test suite is included to verify that `UserProfile` objects are correctly created and that memory allocation is efficient.

## Running the Program

You can adjust the number of user profiles generated by changing the `num_objects` variable in the `main` function. The program prints the time taken for allocation and deallocation.

### Prerequisites

- Rust only

### Building and Running the Program

1. Clone the repository https://github.com/janosvajda/rust-examples
2. Navigate to the directory in your terminal: ../rust-examples/data-structures/hash-map
3. Build and run the program using the following commands:

```bash
cargo build
cargo run

cargo test
