// 1. Using clone() to create an independent copy of the vector
fn clone_example(numbers: &Vec<i32>) -> Vec<i32> {
    numbers.clone()
}

// 2. Using borrowing to create an immutable reference
fn borrowing_example(numbers: &Vec<i32>) -> &Vec<i32> {
    numbers
}

// 3. Using functional methods (map and filter) on the original vector
fn map_example(numbers: &Vec<i32>) -> Vec<i32> {
    numbers.iter().map(|&x| x * x).collect()
}

// 4. Using filter() to create a new vector with even numbers
fn filter_example(numbers: &Vec<i32>) -> Vec<i32> {
    numbers.iter().filter(|&x| x % 2 == 0).copied().collect()
}

// 5. Using slicing to create an immutable view of the vector
fn split_first_example(numbers: &Vec<i32>) -> Option<(i32, &[i32])> {
    if let Some((&first, rest)) = numbers.split_first() {
        Some((first, rest))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clone_example() {
        let numbers = vec![1, 2, 3, 4, 5];
        let cloned_numbers = clone_example(&numbers);
        assert_eq!(cloned_numbers, numbers);
    }

    #[test]
    fn test_borrowing_example() {
        let numbers = vec![1, 2, 3, 4, 5];
        let numbers_ref = borrowing_example(&numbers);
        assert_eq!(numbers_ref, &numbers);
    }

    #[test]
    fn test_map_example() {
        let numbers = vec![1, 2, 3, 4, 5];
        let squared_numbers = map_example(&numbers);
        assert_eq!(squared_numbers, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn test_filter_example() {
        let numbers = vec![1, 2, 3, 4, 5];
        let even_numbers = filter_example(&numbers);
        assert_eq!(even_numbers, vec![2, 4]);
    }

    #[test]
    fn test_split_first_example() {
        let numbers = vec![1, 2, 3, 4, 5];
        let (first, rest) = split_first_example(&numbers).unwrap();
        assert_eq!(first, 1);
        assert_eq!(rest, &[2, 3, 4, 5]);
    }
}

fn main() {
    // Create a vector of integers
    let numbers = vec![1, 2, 3, 4, 5];

    // 1. Using clone() to create an independent copy of the vector
    let cloned_numbers = clone_example(&numbers);
    println!("1. Cloned Numbers: {:?}", cloned_numbers);

    // 2. Using borrowing to create an immutable reference
    let numbers_ref = borrowing_example(&numbers);
    println!("2. Borrowing Reference: {:?}", numbers_ref);

    // 3. Using functional methods (map and filter) on the original vector
    let squared_numbers = map_example(&numbers);
    println!("3. Squared Numbers: {:?}", squared_numbers);

    // 4. Using filter() to create a new vector with even numbers
    let even_numbers = filter_example(&numbers);
    println!("4. Even Numbers: {:?}", even_numbers);

    // 5. Using slicing to create an immutable view of the vector
    if let Some((first, rest)) = split_first_example(&numbers) {
        println!("5. First Element: {}", first);
        println!("   Rest of the Elements: {:?}", rest);
    } else {
        println!("5. The vector is empty.");
    }
}

