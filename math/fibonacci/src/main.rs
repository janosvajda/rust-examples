fn fibonacci(n: u32) -> u32 {
    if n == 0 {
        0
    } else if n == 1 {
        1
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}

fn calculate_fibonacci(n: u32) -> u32 {
    if n < 2 {
        n
    } else {
        (2..=n).fold((0, 1), |(a, b), _| (b, a + b)).1
    }
}

fn main() {

    // Imperative way
    // In the imperative way, we calculate Fibonacci numbers using a recursive function.
    // The function checks if the input is 0 or 1 and returns the corresponding value.
    // For other values, it recursively calls itself with n - 1 and n - 2 and sums the results.
    let n = 10;
    for i in 0..=n {
        let result = fibonacci(i);
        println!("Fibonacci({}) = {}", i, result);
    }

    // Declarative way using fold function
    // In the declarative way, we use the `fold` function to calculate Fibonacci numbers.
    // The `fold` function takes an initial value (0, 1) and applies a closure repeatedly to each element in the range (2..=i).
    // The closure takes two parameters (a, b) representing the last two Fibonacci numbers and returns the next Fibonacci number.
    // The `1` after the `fold` function is used to extract the second element of the tuple, which is the Fibonacci number we need.

    //This is a packed solution. The cons of this is that it is not testable.
    (0..=10).for_each(|i| println!("Fibonacci({}) = {}", i, if i < 2 { i } else { (2..=i).fold((0, 1), |(a, b), _| (b, a + b)).1 }));

    //We can make it testable this way:
    let n = 10;

    for i in 0..=n {
        let result = calculate_fibonacci(i);
        println!("Fibonacci({}) = {}", i, result);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fibonacci() {
        assert_eq!(fibonacci(0), 0);
        assert_eq!(fibonacci(1), 1);
        assert_eq!(fibonacci(2), 1);
        assert_eq!(fibonacci(3), 2);
        assert_eq!(fibonacci(4), 3);
        assert_eq!(fibonacci(5), 5);
    }

    #[test]
    fn test_calculate_fibonacci() {
        assert_eq!(calculate_fibonacci(0), 0);
        assert_eq!(calculate_fibonacci(1), 1);
        assert_eq!(calculate_fibonacci(2), 1);
        assert_eq!(calculate_fibonacci(3), 2);
        assert_eq!(calculate_fibonacci(4), 3);
        assert_eq!(calculate_fibonacci(5), 5);
    }
}