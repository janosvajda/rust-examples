# Factory Design Pattern Example

This example demonstrates the Factory design pattern in Rust. It implements a Robot Factory that can create three types of robots: Robot A, Robot B, and Robot C. Each robot type has different capabilities:

- Robot A (Dumbly): It can move its hands.
- Robot B (Pumbly): It can move its hands and walk.
- Robot C (Wumbly): It can move its hands, walk, and speak.

## How to Run

To run the example, execute the following command:

```bash
$ cargo run
```

This will create instances of each robot type and invoke their respective methods to demonstrate their capabilities.

## How to Test

To run the tests, execute the following command:

```bash
$ cargo test
```

This will run the test cases defined in the code, which validate the behavior of each robot type.
