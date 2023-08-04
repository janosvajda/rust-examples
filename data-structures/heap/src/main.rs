// Define the main function
fn main() {
    // Create a new BinaryHeap instance called 'heap'
    let mut heap = BinaryHeap::new();

    // Insert elements into the heap
    heap.insert(5);
    heap.insert(2);
    heap.insert(10);
    heap.insert(1);

    // Pop elements from the heap and print the minimum value until the heap is empty
    while let Some(min) = heap.pop() {
        println!("Min: {}", min);
    }
}

// Define a struct BinaryHeap to represent the binary heap data structure
struct BinaryHeap {
    data: Vec<i32>,
}

impl BinaryHeap {
    // Implement a new method to create a new empty BinaryHeap
    fn new() -> BinaryHeap {
        BinaryHeap { data: Vec::new() }
    }

    // Implement an insert method to add a new value to the heap
    fn insert(&mut self, value: i32) {
        // Add the value to the end of the data vector
        self.data.push(value);
        // Bubble up the value to its correct position in the heap
        self.bubble_up(self.data.len() - 1);
    }

    // Implement a pop method to remove and return the minimum value from the heap
    fn pop(&mut self) -> Option<i32> {
        // If the heap is empty, return None
        if self.data.is_empty() {
            return None;
        }

        // Swap the first (minimum) element with the last element
        let len = self.data.len();
        self.data.swap(0, len - 1);
        // Pop the last (minimum) element
        let min = self.data.pop();

        // If there is still more than one element in the heap, bubble down the first element
        if len > 1 {
            self.bubble_down(0);
        }

        // Return the minimum value that was removed from the heap
        min
    }

    // Implement a helper method to bubble up a value to its correct position in the heap
    fn bubble_up(&mut self, mut index: usize) {
        while index > 0 {
            // Calculate the index of the parent node
            let parent_index = (index - 1) / 2;
            // If the parent node is smaller or equal to the current node, stop bubbling up
            if self.data[parent_index] <= self.data[index] {
                break;
            }

            // Swap the current node with its parent node
            self.data.swap(parent_index, index);
            // Update the current index to be the parent index for the next iteration
            index = parent_index;
        }
    }

    // Implement a helper method to bubble down a value to its correct position in the heap
    fn bubble_down(&mut self, mut index: usize) {
        let len = self.data.len();
        let mut child_index = 2 * index + 1;

        while child_index < len {
            // Check if the right child exists and is smaller than the left child
            if child_index + 1 < len && self.data[child_index + 1] < self.data[child_index] {
                // If so, choose the right child as the one to compare and swap
                child_index += 1;
            }

            // If the current node is smaller or equal to the child node, stop bubbling down
            if self.data[child_index] >= self.data[index] {
                break;
            }

            // Swap the current node with its smallest child node
            self.data.swap(child_index, index);
            // Update the current index to be the child index for the next iteration
            index = child_index;
            child_index = 2 * index + 1;
        }
    }
}

// Implement some tests for the BinaryHeap struct
#[cfg(test)]
mod tests {
    use super::*;

    // Test the BinaryHeap data structure with a series of insertions and pops
    #[test]
    fn test_binary_heap() {
        // Create a new BinaryHeap instance called 'heap'
        let mut heap = BinaryHeap::new();

        // Insert elements into the heap
        heap.insert(5);
        heap.insert(2);
        heap.insert(10);
        heap.insert(1);

        // Test that the pop method returns the minimum value correctly
        assert_eq!(heap.pop(), Some(1));
        assert_eq!(heap.pop(), Some(2));
        assert_eq!(heap.pop(), Some(5));
        assert_eq!(heap.pop(), Some(10));
        assert_eq!(heap.pop(), None); // The heap should be empty now
    }
}
