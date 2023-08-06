# Function Composition in Rust

Function composition is a powerful concept in functional programming where you combine multiple functions to create a new function. In Rust, you can achieve function composition using higher-order functions, custom macros, or by chaining function calls.

## Example of Function Composition

In this example, we have three simple functions: `add_one`, `square`, and `double`, which perform basic arithmetic operations on an input integer. We demonstrate function composition using various techniques:

1. **Nested Function Calls:** We directly call `add_one` and then `square` in a nested manner to compose the two functions.

2. **Using Closures:** We use a closure `add_one_and_square` to combine the `add_one` and `square` functions.

3. **Using a Custom `compose` Function:** We define a `compose` function that takes two functions as arguments and returns a new function that applies them in sequence.

4. **Chaining Function Calls:** Rust allows chaining function calls by implementing trait methods. We define traits `AddOne` and `Square` and implement them for `i32`, enabling us to chain function calls.

All these approaches demonstrate different ways to achieve function composition in Rust, leading to more expressive and concise code.

## Function Composition vs. Chaining

While function composition and chaining might look similar, they are not the same:

- **Function Composition:** Function composition involves creating a new function by combining two or more functions. The result of one function becomes the input to the next function, forming a chain of operations.

- **Chaining Function Calls:** Chaining function calls, as shown in this example, allows you to call multiple methods in sequence on an object, where each method modifies the object and returns a new instance, providing a fluent API.

Chaining function calls is a useful technique, especially when dealing with method calls on objects, but it is not function composition in the pure functional programming sense.

## Pros and Cons of Function Composition

**Pros:**

- Readability: Function composition allows you to express complex operations concisely, making the code easier to read and understand.

- Reusability: By composing functions, you can create reusable building blocks for different parts of your program.

- Separation of Concerns: Function composition promotes separation of concerns, making the code more modular and maintainable.

**Cons:**

- Performance Overhead: Function composition may introduce some performance overhead due to the creation of intermediate functions.

- Complexity: Complex compositions of functions can sometimes be harder to debug and maintain.

- Learning Curve: Function composition may be unfamiliar to developers coming from an imperative programming background.

Overall, function composition and chaining are powerful techniques that can lead to more expressive and maintainable code. It is essential to understand their differences and use them judiciously based on the specific needs of your application.

Feel free to explore the example provided and experiment with different ways to use function composition and chaining in Rust!
