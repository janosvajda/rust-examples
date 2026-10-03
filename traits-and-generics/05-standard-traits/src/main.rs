// Lesson 5: the standard traits.
//
// A small set of traits from the standard library shows up everywhere:
// printing, copying, comparing, hashing, defaults, conversions and operators.
// Implementing (or deriving) them makes your types work with the rest of Rust.

use std::collections::{BTreeSet, HashMap};
use std::fmt;
use std::ops::{Add, Mul, Neg};

// ---- 1. Debug and Display: two ways to print ----------------------------------------

/// `Debug` is for developers (`{:?}`): derive it on almost everything.
/// `Display` is for users (`{}`): you write it yourself, because only you
/// know how your type should look to a person.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct Money {
    cents: i64,
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sign = if self.cents < 0 { "-" } else { "" };
        write!(f, "{sign}€{}.{:02}", self.cents.abs() / 100, self.cents.abs() % 100)
    }
}

// ---- 2. Operators are traits: Add, Mul, Neg -------------------------------------------

/// `a + b` is just `a.add(b)`. Implement `Add` and `+` works on your type.
impl Add for Money {
    type Output = Money;
    fn add(self, other: Money) -> Money {
        Money { cents: self.cents + other.cents }
    }
}

/// The right-hand side can be a different type: Money * quantity.
impl Mul<i64> for Money {
    type Output = Money;
    fn mul(self, quantity: i64) -> Money {
        Money { cents: self.cents * quantity }
    }
}

impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        Money { cents: -self.cents }
    }
}

// ---- 3. From / Into / TryFrom: conversions --------------------------------------------

/// Implement `From` and you get `Into` for free, in the other direction.
impl From<i64> for Money {
    fn from(euros: i64) -> Money {
        Money { cents: euros * 100 }
    }
}

/// `TryFrom` is for conversions that can fail.
impl TryFrom<&str> for Money {
    type Error = String;

    fn try_from(text: &str) -> Result<Money, String> {
        let text = text.trim().trim_start_matches('€');
        let (euros, cents) = text.split_once('.').unwrap_or((text, "0"));
        let euros: i64 = euros.parse().map_err(|_| format!("bad amount: {text:?}"))?;
        let cents: i64 = format!("{cents:0<2}")[..2]
            .parse()
            .map_err(|_| format!("bad cents: {text:?}"))?;
        Ok(Money { cents: euros * 100 + cents })
    }
}

// ---- 4. Comparing and hashing ----------------------------------------------------------

/// `PartialEq`/`Eq` give `==` and `!=`. `PartialOrd`/`Ord` give `<`, `>`,
/// sorting and BTreeSet/BTreeMap keys. `Hash` (with `Eq`) gives
/// HashMap/HashSet keys. Derived comparisons look at the fields in
/// declaration order: here `priority` first, then `title`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Task {
    priority: u8,
    title: String,
}

// Deriving Eq isn't possible for a struct containing an f64:
//     #[derive(PartialEq, Eq)] struct Point { x: f64 }
//     error[E0277]: the trait bound `f64: Eq` is not satisfied
// Because NaN != NaN, floats only have PartialEq.
//
// Deriving Copy isn't possible when a field owns heap data:
//     #[derive(Clone, Copy)] struct User { name: String }
//     error[E0204]: the trait `Copy` cannot be implemented for this type
//
// A HashMap key without Hash:
//     error[E0277]: the trait bound `Id: Hash` is not satisfied

// ---- 5. Default -------------------------------------------------------------------------

/// `Default` gives a type a sensible "empty" or starting value. Derived, it
/// uses each field's default (0, "", false, empty Vec…), or you write your own.
#[derive(Debug)]
struct Settings {
    volume: u8,
    theme: String,
    notifications: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { volume: 50, theme: String::from("dark"), notifications: true }
    }
}

fn main() {
    println!("1. Debug vs Display");
    let price = Money { cents: 1999 };
    println!("    Debug:   {price:?}");
    println!("    Display: {price}");

    println!("\n2. Operators");
    let total = price * 3 + Money::from(5);
    println!("    3 × {price} + €5 = {total}");
    println!("    refund: {}", -total);

    println!("\n3. Conversions");
    let ten: Money = 10.into(); // Into comes free with From
    println!("    10.into() = {ten}");
    for text in ["€12.5", "7", "€3.99", "lots"] {
        println!("    Money::try_from({text:?}) = {:?}", Money::try_from(text).map(|m| m.to_string()));
    }

    println!("\n4. Comparing, sorting, hashing");
    let mut tasks = vec![
        Task { priority: 2, title: String::from("write tests") },
        Task { priority: 1, title: String::from("fix bug") },
        Task { priority: 2, title: String::from("deploy") },
    ];
    tasks.sort(); // needs Ord: by priority, then title
    for task in &tasks {
        println!("    {} {}", task.priority, task.title);
    }
    let unique: BTreeSet<Task> = tasks.iter().cloned().collect(); // needs Ord
    let mut hours: HashMap<Task, u32> = HashMap::new(); // needs Hash + Eq
    hours.insert(tasks[0].clone(), 3);
    println!("    {} unique tasks; \"{}\" takes {} hours", unique.len(), tasks[0].title, hours[&tasks[0]]);

    println!("\n5. Default");
    let settings = Settings { volume: 80, ..Default::default() }; // override one field
    println!(
        "    volume {} (set), theme {} and notifications {} (defaults)",
        settings.volume, settings.theme, settings.notifications
    );
    println!("    Money::default() = {}", Money::default());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_formats_money() {
        assert_eq!(Money { cents: 105 }.to_string(), "€1.05");
        assert_eq!(Money { cents: -50 }.to_string(), "-€0.50");
    }

    #[test]
    fn operators() {
        let a = Money::from(2);
        assert_eq!(a + a, Money { cents: 400 });
        assert_eq!(a * 3, Money::from(6));
        assert_eq!(-a, Money { cents: -200 });
    }

    #[test]
    fn try_from_parses_or_fails() {
        assert_eq!(Money::try_from("€3.99"), Ok(Money { cents: 399 }));
        assert_eq!(Money::try_from("12.5"), Ok(Money { cents: 1250 }));
        assert_eq!(Money::try_from("4"), Ok(Money { cents: 400 }));
        assert!(Money::try_from("abc").is_err());
    }

    #[test]
    fn derived_ordering_uses_field_order() {
        let a = Task { priority: 1, title: String::from("z") };
        let b = Task { priority: 2, title: String::from("a") };
        assert!(a < b); // priority decides before title
    }

    #[test]
    fn default_with_overrides() {
        let s = Settings { theme: String::from("light"), ..Default::default() };
        assert_eq!((s.volume, s.notifications), (50, true));
    }
}
