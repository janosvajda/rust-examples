fn main() {
    // Annotated closure with an integer parameter
    let closure_annotated = |i: i32| -> i32 { i + 1 };

    // Inferred closure with a string parameter
    let closure_inferred = |s| {
        return format!("{}{}{}", "Hello", " ", s);
    };

    // Inferred closure with two strings parameter, using join function to add strings.
    let closure_inferred2 = |s, s2| {
        let result = [s, s2].join(" ");
        return result;
    };

    let i = 1;

    println!("closure_annotated: {}", closure_annotated(i));

    println!("closure_inferred: {}", closure_inferred("world!"));

    println!(
        "closure_inferred2: {}",
        closure_inferred2("Hello", "world!")
    );
}
