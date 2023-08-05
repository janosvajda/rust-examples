// This is a pure function that filters even numbers from a vector and returns a new vector.
fn filter_even_numbers(input: &Vec<i32>) -> Vec<i32> {
    let even_numbers: Vec<i32> = input.iter().cloned().filter(|&x| x % 2 == 0).collect();
    even_numbers
}

fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Calling the pure function `filter_even_numbers` with `numbers` vector.
    let even_numbers = filter_even_numbers(&numbers);

    println!("Original vector: {:?}", numbers);
    println!("Even numbers: {:?}", even_numbers);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_even_numbers() {
        // Test case 1: Even numbers in the vector
        let input = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let expected_output = vec![2, 4, 6, 8, 10];
        assert_eq!(filter_even_numbers(&input), expected_output);

        // Test case 2: Only odd numbers in the vector
        let input = vec![1, 3, 5, 7, 9];
        let expected_output = Vec::<i32>::new(); // Empty vector as there are no even numbers
        assert_eq!(filter_even_numbers(&input), expected_output);

        // Test case 3: Only even numbers in the vector
        let input = vec![2, 4, 6, 8];
        let expected_output = vec![2, 4, 6, 8];
        assert_eq!(filter_even_numbers(&input), expected_output);

        // Test case 4: Empty vector
        let input = Vec::<i32>::new();
        let expected_output = Vec::<i32>::new();
        assert_eq!(filter_even_numbers(&input), expected_output);
    }
}