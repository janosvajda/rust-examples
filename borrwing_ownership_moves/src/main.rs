// This function takes ownership of the heap allocated memory.
fn destroy_box(c: Box<i32>) {
    println!("Destroying a box that contains {}", c);
    // `c` is destroyed and the memory freed
}

// This function shows the type of the variable.
fn print_type_of<T>(_: &T) {
    println!("Type of the variable: {}", std::any::type_name::<T>())
}

fn main() {
    // _Stack_ allocated integer
    let x = 5u32;
    print_type_of(&x);

    // *Copy* `x` into `y` - no resources are moved
    let y = x;
    print_type_of(&y);

    // Both values can be independently used
    println!("x is {}, and y is {}", x, y);

    // `a` is a pointer to a _heap_ allocated integer
    let a = Box::new(5i32);
    print_type_of(&a);

    println!("a contains: {}", a);

    // *Move* `a` into `b` after this point `a` does not exist anymore
    let b = a;

    // This println won't work. Compile will show an error as `a` is destroyed: value borrowed here after move
    // println!("b contains: {}", a);

    print_type_of(&b);
    destroy_box(b);

    // This println won't work. Compile will show an error as `b` is destroyed.
    // println!("b contains: {}", b);

}
