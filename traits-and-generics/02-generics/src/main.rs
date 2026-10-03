// Lesson 2: generics.
//
// Generics let you write code once and use it with many types. `T` is a
// placeholder for "some type, chosen by whoever uses this code". Trait
// bounds (`T: PartialOrd`) say what that type must be able to do.

use std::fmt::Display;

// ---- 1. A generic function, and why it needs a bound -------------------------------

// Without a bound this doesn't compile: the compiler can't know that every
// possible T supports `>`.
//
// fn largest<T>(items: &[T]) -> &T { … if item > best … }
// error[E0369]: binary operation `>` cannot be applied to type `&T`

/// `T: PartialOrd` = "T can be any type that can be compared with < and >".
fn largest<T: PartialOrd>(items: &[T]) -> Option<&T> {
    let mut best = items.first()?;
    for item in items {
        if item > best {
            best = item;
        }
    }
    Some(best)
}

// ---- 2. Several bounds, and `where` for readability ----------------------------------

/// `+` combines bounds. When a signature gets long, a `where` clause keeps
/// it readable. These two mean exactly the same thing:
///     fn describe<T: Display + PartialOrd>(a: T, b: T) -> String
fn describe<T>(a: T, b: T) -> String
where
    T: Display + PartialOrd,
{
    if a > b {
        format!("{a} is bigger than {b}")
    } else {
        format!("{a} is not bigger than {b}")
    }
}

// ---- 3. Generic structs and enums ------------------------------------------------------

/// A pair of two values of the same type.
#[derive(Debug)]
struct Pair<T> {
    first: T,
    second: T,
}

/// Methods available for EVERY Pair<T>.
impl<T> Pair<T> {
    fn new(first: T, second: T) -> Self {
        Pair { first, second }
    }

    fn swap(self) -> Pair<T> {
        Pair { first: self.second, second: self.first }
    }
}

/// Methods only available when T can be compared and printed.
/// `Pair<Vec<i32>>` still exists, it just doesn't get `bigger()`.
impl<T: PartialOrd + Display> Pair<T> {
    fn bigger(&self) -> &T {
        if self.first >= self.second { &self.first } else { &self.second }
    }
}

/// Generic enums: Option<T> and Result<T, E> from the standard library are
/// exactly this.
#[derive(Debug, PartialEq)]
enum Measurement<T> {
    Exact(T),
    Range { low: T, high: T },
    Unknown,
}

impl<T: Copy + std::ops::Add<Output = T> + std::ops::Div<Output = T> + From<u8>> Measurement<T> {
    /// The best single estimate: the value itself, or the middle of a range.
    fn estimate(&self) -> Option<T> {
        match *self {
            Measurement::Exact(value) => Some(value),
            Measurement::Range { low, high } => Some((low + high) / T::from(2)),
            Measurement::Unknown => None,
        }
    }
}

// ---- 4. Two type parameters ------------------------------------------------------------

/// Keys and values can be different types: `K` and `V` are independent.
fn lookup<'a, K: PartialEq, V>(pairs: &'a [(K, V)], key: &K) -> Option<&'a V> {
    pairs.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

fn main() {
    println!("1. One function, many types");
    println!("    largest number: {:?}", largest(&[3, 17, 8]));
    println!("    largest float:  {:?}", largest(&[2.5, -1.0]));
    println!("    largest char:   {:?}", largest(&['r', 'u', 's', 't']));
    println!("    largest word:   {:?}", largest(&["borrow", "checker"]));
    println!("    empty slice:    {:?}", largest::<i32>(&[]));

    println!("\n2. Several bounds");
    println!("    {}", describe(10, 3));
    println!("    {}", describe("apple", "banana"));

    println!("\n3. Generic structs and enums");
    let numbers = Pair::new(4, 9);
    println!("    {numbers:?}, bigger: {}", numbers.bigger());
    let words = Pair::new("left", "right").swap();
    println!("    swapped: {words:?}, bigger: {}", words.bigger());
    let lists = Pair::new(vec![1], vec![2, 3]); // no `bigger()`: Vec isn't Display
    println!("    {lists:?}");
    println!("    estimates: {:?}, {:?}, {:?}",
        Measurement::Exact(20.5).estimate(),
        Measurement::Range { low: 10, high: 20 }.estimate(),
        Measurement::<f64>::Unknown.estimate());

    println!("\n4. Two type parameters");
    let capitals = [("Hungary", "Budapest"), ("Austria", "Vienna")];
    let ports = [(8080, "dev"), (443, "https")];
    println!("    {:?}", lookup(&capitals, &"Austria"));
    println!("    {:?}", lookup(&ports, &443));

    println!("\n5. What the compiler does with generics: monomorphisation");
    // For each concrete type used, the compiler generates a separate copy of
    // the function: largest::<i32>, largest::<f64>, largest::<char>, …
    // So generic code runs exactly as fast as code written for one type.
    println!("    this program contains separate, fully optimised copies of `largest`");
    println!("    for i32, f64, char and &str: one per type it was called with");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn largest_works_for_many_types() {
        assert_eq!(largest(&[1, 5, 3]), Some(&5));
        assert_eq!(largest(&["a", "c", "b"]), Some(&"c"));
        assert_eq!(largest::<u8>(&[]), None);
    }

    #[test]
    fn methods_with_extra_bounds() {
        assert_eq!(*Pair::new(2, 7).bigger(), 7);
        let swapped = Pair::new(1, 2).swap();
        assert_eq!((swapped.first, swapped.second), (2, 1));
    }

    #[test]
    fn generic_enum() {
        assert_eq!(Measurement::Range { low: 10, high: 20 }.estimate(), Some(15));
        assert_eq!(Measurement::Exact(1.5).estimate(), Some(1.5));
        assert_eq!(Measurement::<i32>::Unknown.estimate(), None);
    }

    #[test]
    fn two_type_parameters() {
        let pairs = [('a', 1), ('b', 2)];
        assert_eq!(lookup(&pairs, &'b'), Some(&2));
        assert_eq!(lookup(&pairs, &'z'), None);
    }
}
