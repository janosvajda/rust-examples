// Lesson 2: Box<T>, one owner, a value on the heap.
//
// `Box::new(value)` moves the value to the heap and gives you a pointer to
// it. The Box owns the value: when the Box goes away, so does the value.
// Three situations need it, and this lesson shows each one.

// ---- 1. Recursive types ------------------------------------------------------------------------
//
//     enum Expr { Num(i64), Add(Expr, Expr) }
//     error[E0072]: recursive type `Expr` has infinite size
//
// An Expr that contains an Expr that contains an Expr… would be infinitely big.
// A Box has a fixed size (one pointer), so a Box<Expr> inside an Expr is fine.

/// A tiny expression tree: numbers, additions and multiplications.
#[derive(Debug)]
enum Expr {
    Num(i64),
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

fn eval(expr: &Expr) -> i64 {
    match expr {
        Expr::Num(n) => *n,
        Expr::Add(a, b) => eval(a) + eval(b),
        Expr::Mul(a, b) => eval(a) * eval(b),
    }
}

fn num(n: i64) -> Box<Expr> {
    Box::new(Expr::Num(n))
}

// ---- 2. Trait objects: different types in one list ---------------------------------------
//
// A Circle and a Rectangle have different sizes, so they can't sit side by
// side in a Vec. Boxed, each is just a pointer (plus a vtable pointer), all the
// same size. See traits lesson 3 for how `dyn` works.

trait Shape {
    fn area(&self) -> f64;
    fn name(&self) -> &'static str;
}

struct Circle {
    radius: f64,
}

struct Rectangle {
    width: f64,
    height: f64,
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }
    fn name(&self) -> &'static str {
        "circle"
    }
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }
    fn name(&self) -> &'static str {
        "rectangle"
    }
}

fn total_area(shapes: &[Box<dyn Shape>]) -> f64 {
    shapes.iter().map(|s| s.area()).sum()
}

// ---- 3. Big values: move a pointer, not the whole thing ---------------------------------

/// A big value: 1 MB. Moving it around copies 1 MB every time; and created
/// directly in a thread with a small stack, it could even overflow it.
struct Image {
    pixels: [u8; 1_000_000],
}

/// Takes a boxed image: only the 8-byte pointer is passed, never the megabyte.
fn brightness(image: Box<Image>) -> u8 {
    image.pixels.iter().max().copied().unwrap_or(0)
} // the Box is dropped here, and the 1 MB on the heap is freed with it

fn main() {
    println!("1. A recursive type");
    let expr = Expr::Mul(Box::new(Expr::Add(num(2), num(3))), num(4)); // (2 + 3) * 4
    println!("    {expr:?} = {}", eval(&expr));

    println!("\n2. Different types in one Vec");
    let shapes: Vec<Box<dyn Shape>> = vec![
        Box::new(Circle { radius: 1.0 }),
        Box::new(Rectangle {
            width: 2.0,
            height: 3.0,
        }),
    ];
    for shape in &shapes {
        println!("    {} has area {:.2}", shape.name(), shape.area());
    }
    println!("    total area: {:.2}", total_area(&shapes));

    println!("\n3. A big value, passed as a small pointer");
    // Box::new builds the value first (usually on the stack), then moves it to
    // the heap: for 1 MB that's fine. For values too big for the stack, build
    // them on the heap from the start, e.g. vec![0u8; n].into_boxed_slice().
    let mut image: Box<Image> = Box::new(Image {
        pixels: [0; 1_000_000],
    });
    image.pixels[123] = 200; // a Box is used just like the value it points to
    println!("    size of Image:      {} bytes", size_of::<Image>());
    println!("    size of Box<Image>: {} bytes", size_of::<Box<Image>>());
    println!("    brightest pixel: {}", brightness(image));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recursive_expressions() {
        let expr = Expr::Add(num(1), Box::new(Expr::Mul(num(2), num(3)))); // 1 + 2 * 3
        assert_eq!(eval(&expr), 7);
    }

    #[test]
    fn trait_objects_in_one_vec() {
        let shapes: Vec<Box<dyn Shape>> = vec![
            Box::new(Rectangle {
                width: 2.0,
                height: 3.0,
            }),
            Box::new(Rectangle {
                width: 1.0,
                height: 4.0,
            }),
        ];
        assert_eq!(total_area(&shapes), 10.0);
    }

    #[test]
    fn a_box_is_one_pointer_whatever_it_holds() {
        assert_eq!(size_of::<Box<Image>>(), size_of::<usize>());
        assert_eq!(size_of::<Box<u8>>(), size_of::<usize>());
    }

    #[test]
    fn a_box_derefs_to_its_value() {
        let boxed = Box::new(41);
        assert_eq!(*boxed + 1, 42); // `*` reaches the value inside
    }
}
