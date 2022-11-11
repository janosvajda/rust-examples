fn main() {
    // Creating a raw pointer is safe.
    let raw_p: *const u32 = &99;

    // The raw pointers dereferencing is not checked by the compile-time checks process so they must be wrapped by unsafe{}.
    // This works, but it is not the best practice as the unsafe part in the code should be as short as it can be.
    unsafe {
        if *raw_p == 99 {
            println!("{}", *raw_p);
        }
    }

    // So, this does the same, and it is a better practice.
    let p = unsafe {*raw_p};
    if p == 99 {
        println!("{}", p);
    }
}
