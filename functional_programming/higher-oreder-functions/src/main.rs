// Using a higher-order function with a regular function as an argument
fn apply_function_regular<F>(num: i32, func: F) -> i32
where
    F: Fn(i32) -> i32,
{
    func(num)
}

// Using a higher-order function with a closure as an argument
fn apply_function_closure<F>(num: i32, func: F) -> i32
where
    F: Fn(i32) -> i32,
{
    func(num)
}

// Using a higher-order function with a function pointer as an argument
fn apply_function_pointer(num: i32, func: fn(i32) -> i32) -> i32 {
    func(num)
}

// Using trait to define a higher-order function with trait objects as arguments
trait Function {
    fn apply(&self, num: i32) -> i32;
}

struct Double;
impl Function for Double {
    fn apply(&self, num: i32) -> i32 {
        num * 2
    }
}

fn apply_function_trait_object(func: &dyn Function, num: i32) -> i32 {
    func.apply(num)
}

// Define a function that doubles the input value
fn double(x: i32) -> i32 {
    x * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_function_regular() {
        let num = 5;
        let result_regular = apply_function_regular(num, double);
        assert_eq!(result_regular, 10);
    }

    #[test]
    fn test_apply_function_closure() {
        let num = 5;
        let result_closure = apply_function_closure(num, |x| x * 2);
        assert_eq!(result_closure, 10);
    }

    #[test]
    fn test_apply_function_pointer() {
        let num = 5;
        let result_pointer = apply_function_pointer(num, double);
        assert_eq!(result_pointer, 10);
    }

    #[test]
    fn test_apply_function_trait_object() {
        let num = 5;
        let double = Double;
        let result_trait_double = apply_function_trait_object(&double, num);
        assert_eq!(result_trait_double, 10);
    }
}

fn main() {
    let num = 5;

    // Using a regular function as an argument to the higher-order function
    let result_regular = apply_function_regular(num, double);
    println!("Double of {} using regular function: {}", num, result_regular);

    // Using a closure as an argument to the higher-order function
    let result_closure = apply_function_closure(num, |x| x * 2);
    println!("Double of {} using closure: {}", num, result_closure);

    // Using a function pointer as an argument to the higher-order function
    let result_pointer = apply_function_pointer(num, double);
    println!("Double of {} using function pointer: {}", num, result_pointer);

    // Using trait objects as arguments to the higher-order function
    let double = Double;
    let result_trait_double = apply_function_trait_object(&double, num);
    println!("Double of {} using trait object: {}", num, result_trait_double);
}
