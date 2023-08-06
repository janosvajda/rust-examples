// Define the add_one function
fn add_one(x: i32) -> i32 {
    x + 1
}

// Define the square function
fn square(x: i32) -> i32 {
    x * x
}

// Define the double function
fn double(x: i32) -> i32 {
    x * 2
}

// Define a function for function composition
fn compose<F, G, T>(f: F, g: G) -> impl Fn(T) -> T
where
    F: Fn(T) -> T,
    G: Fn(T) -> T,
{
    move |x| g(f(x))
}

fn main() {
    let num = 5;

    // Using nested function calls for function composition
    let result_nested = square(add_one(num));
    println!("Nested: {}", result_nested);

    // Using function composition with closures
    let add_one_and_square = |x| square(add_one(x));
    let result_closure = add_one_and_square(num);
    println!("Closure: {}", result_closure);

    // Using the compose function for function composition
    let add_one_and_square = compose(add_one, square);
    let result_composed = add_one_and_square(num);
    println!("Composed: {}", result_composed);

    // Chaining function calls
    let result_chained = num.add_one().square();
    println!("Chained: {}", result_chained);

    // Call the double function
    let result_double = double(num);
    println!("Double: {}", result_double);
}

trait AddOne {
    fn add_one(self) -> i32;
}

trait Square {
    fn square(self) -> i32;
}

impl AddOne for i32 {
    fn add_one(self) -> i32 {
        add_one(self)
    }
}

impl Square for i32 {
    fn square(self) -> i32 {
        square(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compose() {
        let num = 5;

        // Using the compose function for function composition
        let add_one_and_square = compose(add_one, square);
        let result_composed = add_one_and_square(num);
        assert_eq!(result_composed, 36);
    }

    #[test]
    fn test_chained() {
        let num = 5;

        // Chaining function calls
        let result_chained = num.add_one().square();
        assert_eq!(result_chained, 36);
    }

    #[test]
    fn test_double() {
        let num = 5;

        // Call the double function
        let result_double = double(num);
        assert_eq!(result_double, 10);
    }
}
