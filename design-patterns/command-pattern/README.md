
# Command Design Pattern Example - Mars Rover Kata

This repository contains a Rust implementation of the Mars Rover simulation using the Command Design Pattern. The Mars Rover is a simple 2D rover that can move and turn on a 5x5 grid.

# Command Design Pattern

The Command design pattern is a behavioral pattern that allows decoupling the sender of a request from its receiver, providing a way to parameterize objects with operations. It enables the encapsulation of a request as an object, allowing clients to parameterize different requests, queue or log requests, and perform undoable operations.

## Intent

The Command design pattern's main intent is to convert requests or simple operations into objects. By encapsulating a request in an object, we can parameterize clients with different requests, delay or queue a request's execution, and support undoable operations.

## Participants

The main participants in the Command design pattern are:

1. **Client**: Initiates the requests and holds references to the Command objects.
2. **Invoker**: Asks the command to carry out the request.
3. **Command**: Declares an interface for executing a specific operation.
4. **ConcreteCommand**: Implements the Command interface and holds the receiver object. It defines the binding between the action and the receiver.
5. **Receiver**: Knows how to perform the operations associated with the request.

## Structure

The Command design pattern follows a simple structure:

- The Client creates Command objects and sets their receivers.
- The Invoker asks the Command object to carry out the request.
- The Command object delegates the request to the appropriate method of the Receiver.
- The Receiver performs the action associated with the request.

## Benefits

The Command design pattern offers several advantages:

- **Decoupling**: It decouples the sender from the receiver, allowing different requests to be handled independently.
- **Flexibility**: Clients can be parameterized with different commands, allowing for easy extension and changes.
- **Undo/Redo**: By encapsulating requests as objects, it becomes possible to implement undo and redo operations.
- **Logging and Auditing**: Command objects can be logged or stored to enable auditing and tracking of operations.

## Use Cases

The Command design pattern is suitable in the following scenarios:

- Implementing callback functionality: It allows deferring a particular operation to be executed at a later time.
- Implementing undo/redo functionality: It provides an easy way to undo and redo operations by storing and executing command objects.
- Implementing queuing and scheduling systems: It enables queuing and scheduling of requests by encapsulating them as command objects.

## Conclusion

The Command design pattern provides a flexible and decoupled approach to handle requests and operations. By encapsulating requests as objects, it enables queuing, scheduling, undo/redo functionality, and easy extension of the application. It is a powerful pattern to implement complex command-based systems and provides a way to build robust and maintainable software applications.


## Purpose

The purpose of this example is to demonstrate the Command Design Pattern, a behavioral design pattern, which allows us to encapsulate a request as an object and decouples the sender (client) from the receiver (object that performs the action) and the specific actions (commands) that need to be executed.


## Functionality

The code simulates a Mars Rover capable of the following actions:
- Move Forward: Moves the rover one step forward in the current direction.
- Turn Left: Changes the rover's direction 90 degrees to the left.
- Turn Right: Changes the rover's direction 90 degrees to the right.

The Mars Rover starts at position (0, 0) in the North direction on a 5x5 grid. It can move within the grid, and if it reaches the grid boundaries, it wraps around to the other side (toroidal topology).

## How to run

    cargo run

## How to test

    cargo test

