// Lesson 4: associated types and associated constants.
//
// A trait can contain more than methods:
//   associated types   a type each implementor chooses, like Iterator's `Item`
//   associated consts  a value each implementor chooses
// And it can be generic itself (`trait Convert<T>`). Knowing when to use an
// associated type and when to use a generic parameter is the key idea here.

// ---- 1. An associated type: each implementor picks ONE --------------------------------

/// A container that hands out items. Each container decides what its items
/// are, and there's exactly one answer per container type.
trait Container {
    type Item;

    fn get(&self, index: usize) -> Option<&Self::Item>;
    fn first(&self) -> Option<&Self::Item> {
        self.get(0)
    }
}

struct Shelf {
    books: Vec<String>,
}

impl Container for Shelf {
    type Item = String; // a Shelf holds Strings, and only Strings

    fn get(&self, index: usize) -> Option<&String> {
        self.books.get(index)
    }
}

struct Thermometer {
    readings: [f64; 3],
}

impl Container for Thermometer {
    type Item = f64;

    fn get(&self, index: usize) -> Option<&f64> {
        self.readings.get(index)
    }
}

// ---- 2. Your own iterator: Iterator uses an associated type ----------------------------

/// Counts down from a number to 1. `type Item = u32` tells everyone what
/// `next()` produces, so `sum`, `collect` and the rest know the type too.
struct Countdown(u32);

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.0 == 0 {
            None
        } else {
            self.0 -= 1;
            Some(self.0 + 1)
        }
    }
}

// ---- 3. Associated constants ---------------------------------------------------------------

trait Vehicle {
    /// Each vehicle type has its own fixed value, known at compile time.
    const WHEELS: u32;
    const MAX_SPEED_KMH: u32;

    fn describe() -> String {
        format!("{} wheels, up to {} km/h", Self::WHEELS, Self::MAX_SPEED_KMH)
    }
}

struct Bicycle;
struct Car;

impl Vehicle for Bicycle {
    const WHEELS: u32 = 2;
    const MAX_SPEED_KMH: u32 = 40;
}

impl Vehicle for Car {
    const WHEELS: u32 = 4;
    const MAX_SPEED_KMH: u32 = 180;
}

/// Constants can be used in generic code, too.
fn total_wheels<V: Vehicle>(count: u32) -> u32 {
    V::WHEELS * count
}

// ---- 4. A generic trait: one type, MANY implementations ---------------------------------

/// Unlike an associated type, a generic parameter lets one type implement
/// the trait several times: a Celsius value can convert into Fahrenheit AND
/// into Kelvin. With an associated type it could only pick one.
trait ConvertTo<T> {
    fn convert(&self) -> T;
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Celsius(f64);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Fahrenheit(f64);
#[derive(Debug, Clone, Copy, PartialEq)]
struct Kelvin(f64);

impl ConvertTo<Fahrenheit> for Celsius {
    fn convert(&self) -> Fahrenheit {
        Fahrenheit(self.0 * 9.0 / 5.0 + 32.0)
    }
}

impl ConvertTo<Kelvin> for Celsius {
    fn convert(&self) -> Kelvin {
        Kelvin(self.0 + 273.15)
    }
}

fn main() {
    println!("1. Associated types: each container picks its item type");
    let shelf = Shelf { books: vec![String::from("Dune"), String::from("Emma")] };
    let thermometer = Thermometer { readings: [21.5, 22.0, 19.8] };
    println!("    shelf.first():       {:?}", shelf.first());
    println!("    thermometer.get(2):  {:?}", thermometer.get(2));

    println!("\n2. Iterator's Item is an associated type");
    let launch: Vec<u32> = Countdown(5).collect();
    println!("    countdown: {launch:?}, sum: {}", Countdown(5).sum::<u32>());

    println!("\n3. Associated constants");
    println!("    bicycle: {}", Bicycle::describe());
    println!("    car:     {}", Car::describe());
    println!("    wheels on 3 cars: {}", total_wheels::<Car>(3));

    println!("\n4. A generic trait: one type, several implementations");
    let today = Celsius(25.0);
    // The type annotation picks WHICH implementation to use.
    let f: Fahrenheit = today.convert();
    let k: Kelvin = today.convert();
    println!("    {today:?} = {f:?} = {k:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn associated_type_per_implementor() {
        let shelf = Shelf { books: vec![String::from("A")] };
        assert_eq!(shelf.first(), Some(&String::from("A")));
        let t = Thermometer { readings: [1.0, 2.0, 3.0] };
        assert_eq!(t.get(5), None);
    }

    #[test]
    fn custom_iterator_works_with_adapters() {
        assert_eq!(Countdown(3).collect::<Vec<_>>(), vec![3, 2, 1]);
        assert_eq!(Countdown(4).filter(|n| n % 2 == 0).count(), 2);
        assert_eq!(Countdown(0).next(), None);
    }

    #[test]
    fn associated_constants() {
        assert_eq!(Bicycle::WHEELS, 2);
        assert_eq!(total_wheels::<Bicycle>(5), 10);
        assert_eq!(Car::describe(), "4 wheels, up to 180 km/h");
    }

    #[test]
    fn generic_trait_has_several_impls() {
        let f: Fahrenheit = Celsius(100.0).convert();
        let k: Kelvin = Celsius(0.0).convert();
        assert_eq!(f, Fahrenheit(212.0));
        assert_eq!(k, Kelvin(273.15));
    }
}
