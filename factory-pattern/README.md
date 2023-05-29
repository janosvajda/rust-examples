# Factory Design Pattern Example

This example demonstrates the Factory design pattern in Rust. The project is divided into multiple files each representing a separate module:

- `main.rs`: This is the entry point of our program. It contains the main function and a separate function `run` that carries out the main logic of creating robot instances and displaying their names. Unit tests are also included in this file.

- `robot.rs`: This file contains the definition of the Robot trait, which defines the common behaviors of all robots. It also includes the definitions of RobotA, RobotB, and RobotC structs along with their specific implementations of the Robot trait. 

- `robot_factory.rs`: This file includes the definition of the RobotFactory struct, which has methods to create instances of each type of robot.

Each robot type has different capabilities:

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
