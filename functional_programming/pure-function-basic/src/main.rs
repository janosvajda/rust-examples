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