# Functional Programming and Immutability

Functional programming is a programming paradigm that emphasizes the use of pure functions, immutability, and functional methods to build programs. One of the core principles of functional programming is immutability, which means that once data is assigned to a variable, it cannot be changed. Instead of modifying data in place, functional programming relies on creating new data structures with the desired changes.

In this Rust code example, we explore various aspects of immutability in functional programming using different techniques:

## Functions and Their Demonstrations:

### 1. Using `clone()` to Create an Independent Copy of the Vector

The function `clone_example(numbers: &Vec<i32>) -> Vec<i32>` demonstrates how to create an independent copy of the vector `numbers`. It uses the `clone()` method to create a new vector containing the same elements as the original vector. The new vector is a completely separate data structure, and any changes made to it will not affect the original vector. This showcases how cloning can be used to achieve immutability by avoiding modifications to the original data.

### 2. Using Borrowing to Create an Immutable Reference

The function `borrowing_example(numbers: &Vec<i32>) -> &Vec<i32>` shows how to use borrowing in Rust to create an immutable reference to the vector `numbers`. Borrowing allows the function to access the elements of the vector without taking ownership of it. Since it's an immutable reference (`&`), the function cannot modify the contents of the vector. This demonstrates how borrowing can be used to achieve immutability by restricting write access to the data.

### 3. Using Functional Methods (map and filter) on the Original Vector

The function `map_example(numbers: &Vec<i32>) -> Vec<i32>` demonstrates how functional methods like `map()` can be used to perform transformations on the elements of the vector `numbers`. It uses `map()` to square each element of the vector, returning a new vector with the squared values. This functional approach maintains immutability by creating a new vector and not modifying the original vector. The function showcases how functional methods provide a concise and immutable way to perform operations on data.

### 4. Using `filter()` to Create a New Vector with Even Numbers

The function `filter_example(numbers: &Vec<i32>) -> Vec<i32>` illustrates how to use the `filter()` method to create a new vector containing only the even numbers from the original vector `numbers`. The `filter()` method constructs a new vector by selecting elements that satisfy a given condition, in this case, being even. This showcases how filtering can be used to create a new collection without changing the original data, adhering to the principle of immutability.

### 5. Using Slicing to Create an Immutable View of the Vector

The function `split_first_example(numbers: &Vec<i32>) -> Option<(i32, &[i32])>` demonstrates slicing, a way to create an immutable view of the vector `numbers`. It uses `split_first()` to obtain a tuple containing the first element of the vector and a slice representing the rest of the elements. Slicing allows you to work with a portion of the original data while keeping it immutable. This function showcases how slicing enables you to view data without the need to create additional copies or modify the original data.

---

By exploring these different techniques in functional programming, you can see how immutability plays a crucial role in creating reliable and predictable code. Embracing immutability encourages writing pure functions that don't have side effects and leads to more robust, testable, and maintainable programs.
