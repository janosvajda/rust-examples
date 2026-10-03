// Using the derive macro from the `describe-derive` crate.
//
// `#[derive(Describe)]` runs the macro at compile time, which writes a
// `describe()` method and a `field_names()` function for each struct.

use describe_derive::Describe;

#[derive(Describe)]
struct User {
    name: String,
    age: u32,
    admin: bool,
}

#[derive(Describe)]
struct Point {
    x: f64,
    y: f64,
}

/// Generics work too: the macro copies the struct's generic parameters
/// onto the generated impl.
#[derive(Describe)]
struct Labelled<T: std::fmt::Debug> {
    label: &'static str,
    value: T,
}

// Using it on something it doesn't support gives the macro's own error:
//
//     #[derive(Describe)]
//     enum Colour { Red, Green }
//     error: Describe can only be derived for structs, not enums or unions
//
//     #[derive(Describe)]
//     struct Pair(u32, u32);
//     error: Describe needs a struct with named fields, like `struct S { a: u32 }`

fn main() {
    let user = User { name: String::from("Ferris"), age: 9, admin: true };
    let point = Point { x: 1.5, y: -2.0 };
    let tagged = Labelled { label: "scores", value: vec![10, 20] };

    println!("Generated describe():");
    println!("    {}", user.describe());
    println!("    {}", point.describe());
    println!("    {}", tagged.describe());

    println!("\nGenerated field_names():");
    println!("    User:  {:?}", User::field_names());
    println!("    Point: {:?}", Point::field_names());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn describe_lists_every_field() {
        let user = User { name: String::from("Ana"), age: 30, admin: false };
        assert_eq!(user.describe(), "User { name: \"Ana\", age: 30, admin: false }");
    }

    #[test]
    fn field_names_in_declaration_order() {
        assert_eq!(User::field_names(), ["name", "age", "admin"]);
        assert_eq!(Point::field_names(), ["x", "y"]);
    }

    #[test]
    fn works_with_generics() {
        let l = Labelled { label: "n", value: 5u8 };
        assert_eq!(l.describe(), "Labelled { label: \"n\", value: 5 }");
    }
}
