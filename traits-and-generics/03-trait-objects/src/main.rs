// Lesson 3: trait objects.
//
// Generics pick the concrete type at COMPILE time. Trait objects (`dyn Trait`)
// pick it at RUN time. That's what lets one Vec hold circles, squares and
// triangles together, or a function return different types depending on input.

use std::f64::consts::PI;

trait Shape {
    fn name(&self) -> &'static str;
    fn area(&self) -> f64;

    fn describe(&self) -> String {
        format!("{} with area {:.2}", self.name(), self.area())
    }
}

struct Circle {
    radius: f64,
}
struct Square {
    side: f64,
}
struct Triangle {
    base: f64,
    height: f64,
}

impl Shape for Circle {
    fn name(&self) -> &'static str {
        "circle"
    }
    fn area(&self) -> f64 {
        PI * self.radius * self.radius
    }
}

impl Shape for Square {
    fn name(&self) -> &'static str {
        "square"
    }
    fn area(&self) -> f64 {
        self.side * self.side
    }
}

impl Shape for Triangle {
    fn name(&self) -> &'static str {
        "triangle"
    }
    fn area(&self) -> f64 {
        self.base * self.height / 2.0
    }
}

// ---- 1. A collection of different types ---------------------------------------------

/// A `Vec<T>` holds values of ONE type. `Vec<Box<dyn Shape>>` holds boxes
/// that each point to SOME type implementing Shape, all different.
fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|shape| shape.area()).sum()
}

// ---- 2. Static vs dynamic dispatch -----------------------------------------------------

/// Static dispatch: one copy of this function per concrete type, chosen at
/// compile time. `area()` is a direct call, and can even be inlined.
fn print_static<S: Shape>(shape: &S) -> String {
    shape.describe()
}

/// Dynamic dispatch: ONE copy of this function for all shapes. Which
/// `area()` runs is looked up at run time, through the trait object's vtable.
fn print_dynamic(shape: &dyn Shape) -> String {
    shape.describe()
}

// ---- 3. Returning different types from one function -----------------------------------

/// `-> impl Shape` would NOT work here: every branch must return the same
/// type for that. `Box<dyn Shape>` can hold any of them.
fn parse_shape(text: &str) -> Option<Box<dyn Shape>> {
    let mut parts = text.split_whitespace();
    let kind = parts.next()?;
    let numbers: Vec<f64> = parts.filter_map(|p| p.parse().ok()).collect();
    match (kind, numbers.as_slice()) {
        ("circle", [r]) => Some(Box::new(Circle { radius: *r })),
        ("square", [s]) => Some(Box::new(Square { side: *s })),
        ("triangle", [b, h]) => Some(Box::new(Triangle { base: *b, height: *h })),
        _ => None,
    }
}

// ---- 4. Not every trait can be a trait object --------------------------------------------
//
// trait Duplicate { fn duplicate(&self) -> Self; }
// let items: Vec<Box<dyn Duplicate>> = …;
// error[E0038]: the trait `Duplicate` is not dyn compatible
//
// `-> Self` needs to know the concrete type's size at compile time, which a
// trait object hides. Generic methods (`fn f<T>(&self)`) are ruled out too.

fn main() {
    println!("1. One Vec, three different types");
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Square { side: 2.0 }),
        Box::new(Triangle { base: 3.0, height: 4.0 }),
    ];
    for shape in &shapes {
        println!("    {}", shape.describe());
    }
    println!("    total area: {:.2}", total_area(&shapes));

    println!("\n2. Static and dynamic dispatch give the same answer");
    let square = Square { side: 3.0 };
    println!("    static:  {}", print_static(&square));
    println!("    dynamic: {}", print_dynamic(&square));

    println!("\n3. Choosing the type at run time");
    for text in ["circle 2", "triangle 6 2", "hexagon 1", "square"] {
        match parse_shape(text) {
            Some(shape) => println!("    {text:<14} → {}", shape.describe()),
            None => println!("    {text:<14} → not a shape I know"),
        }
    }

    println!("\n4. What a trait object is made of");
    println!("    &Square:    {} bytes (one pointer)", size_of::<&Square>());
    println!("    &dyn Shape: {} bytes (pointer to the data + pointer to the vtable)", size_of::<&dyn Shape>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_collection() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Square { side: 2.0 }),
            Box::new(Triangle { base: 2.0, height: 2.0 }),
        ];
        assert_eq!(total_area(&shapes), 6.0);
    }

    #[test]
    fn static_and_dynamic_agree() {
        let c = Circle { radius: 2.0 };
        assert_eq!(print_static(&c), print_dynamic(&c));
    }

    #[test]
    fn runtime_choice_of_type() {
        assert_eq!(parse_shape("square 3").unwrap().area(), 9.0);
        assert_eq!(parse_shape("triangle 4 5").unwrap().name(), "triangle");
        assert!(parse_shape("circle").is_none()); // missing the radius
        assert!(parse_shape("blob 1").is_none());
    }

    #[test]
    fn trait_object_is_a_fat_pointer() {
        assert_eq!(size_of::<&dyn Shape>(), 2 * size_of::<usize>());
        assert_eq!(size_of::<&Square>(), size_of::<usize>());
    }
}
