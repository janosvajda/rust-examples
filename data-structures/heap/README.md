# Binary Heap in Rust

## What is a Binary Heap?

A binary heap is a specialized binary tree-based data structure that satisfies the heap property. The heap property states that the key of each node in the tree must be either greater than or equal to (max heap) or less than or equal to (min heap) the keys of its children. In a binary max heap, the root node is always the maximum value in the heap, while in a binary min heap, the root node is always the minimum value.

Binary heaps are commonly used for priority queues and can efficiently support the insertion, deletion, and retrieval of the minimum or maximum element in logarithmic time (O(log n)), where 'n' is the number of elements in the heap.

## Code Functionality

The Rust example in this repository implements a simple binary heap data structure, represented by the `BinaryHeap` struct. The implementation includes methods to create a new empty heap, insert elements into the heap, and pop the minimum element from the heap.

### BinaryHeap Methods

1. `new()`: Creates a new empty binary heap.

2. `insert(&mut self, value: i32)`: Inserts a new value into the binary heap and maintains the heap property by bubbling up the value to its correct position.

3. `pop(&mut self) -> Option<i32>`: Removes and returns the minimum value (top element) from the binary heap and maintains the heap property by bubbling down the top element to its correct position.

### Running the Code

The main function of the Rust program demonstrates the usage of the binary heap data structure. It creates a new binary heap, inserts a series of elements, and then pops the elements one by one, printing the minimum value until the heap is empty.

To run the code and see the output, execute the following command:

```bash
cargo run

### Running the Code

```bash
cargo test