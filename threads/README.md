# Multi-Threaded Hello World

This is a simple Rust program that demonstrates multi-threading using threads from the `std::thread` module. The program prints messages concurrently from multiple threads to showcase parallel execution.

## What it Does

The program creates three threads:

1. **First Spawn Thread**: This thread prints messages with a red color prefix, starting from "Hello from first spawn: 1" up to "Hello from first spawn: 100". Each message is printed with a 1-second delay between them.

2. **Second Spawn Thread**: This thread prints messages with a green color prefix, starting from "Hello from second spawn: 1" up to "Hello from second spawn: 100". Each message is printed with a 10-millisecond delay between them.

3. **Main Thread**: This is the main thread of the program. It prints messages with a blue color prefix, starting from "Hello from main1" up to "Hello from main154". Each message is printed with a 1-millisecond delay between them.

## Running the Code

```bash
$ cargo run
```

You will see the messages printed by the program in your console.

## Running the Tests

The code also includes a test function that ensures the expected behavior of the multi-threaded program. To run the tests, follow these steps:

1. Open the `src/main.rs` file and add the provided test function at the end of the file, inside the `tests` module.

2. Save the changes.

3. Run the tests with Cargo:

```bash
$ cargo test
```

The test function will execute, and you can observe the console output to see the messages printed by the threads.

