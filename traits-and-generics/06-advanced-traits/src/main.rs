// Lesson 6: advanced traits.
//
// Supertraits, blanket implementations, the orphan rule and how to work
// around it, extension traits, and calling a method when two traits have one
// with the same name.

use std::fmt::{self, Display};

// ---- 1. Supertraits: "to be an X, you must also be a Y" -------------------------------

/// Anything `Printable` must also be `Display`, so `print_framed` can use
/// `to_string()` without asking for it separately.
trait Printable: Display {
    fn print_framed(&self) -> String {
        let text = self.to_string();
        let line = "─".repeat(text.chars().count() + 2);
        format!("┌{line}┐\n    │ {text} │\n    └{line}┘")
    }
}

struct Badge(&'static str);

impl Display for Badge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "★ {}", self.0)
    }
}

impl Printable for Badge {} // allowed only because Badge is Display

// ---- 2. Blanket implementations: implement for every type that … ------------------------

/// Implement `Shout` for EVERY type that implements Display, in one go.
/// (This is how the standard library gives every Display type `to_string()`.)
trait Shout {
    fn shout(&self) -> String;
}

impl<T: Display> Shout for T {
    fn shout(&self) -> String {
        format!("{}!", self.to_string().to_uppercase())
    }
}

// ---- 3. The orphan rule, and the newtype workaround ------------------------------------
//
// You can implement a trait for a type only if the trait OR the type is
// defined in your crate. Implementing a std trait for a std type fails:
//
//     impl fmt::Display for Vec<i32> { … }
//     error[E0117]: only traits defined in the current crate can be implemented
//                   for types defined outside of the crate
//
// Otherwise two crates could both implement Display for Vec<i32>, and Rust
// couldn't know which to use. The workaround: wrap the type in your own
// struct (a "newtype"), and implement the trait for that.

struct CommaList(Vec<i32>);

impl Display for CommaList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.0.iter().map(|n| n.to_string()).collect();
        write!(f, "{}", parts.join(", "))
    }
}

// ---- 4. Extension traits: add methods to types you don't own --------------------------

/// Your OWN trait, implemented for a std type: allowed by the orphan rule.
/// After `use`-ing the trait, every &str gets these methods.
trait StrExt {
    fn word_count(&self) -> usize;
    fn is_shouting(&self) -> bool;
}

impl StrExt for str {
    fn word_count(&self) -> usize {
        self.split_whitespace().count()
    }

    fn is_shouting(&self) -> bool {
        self.chars().any(char::is_alphabetic)
            && self
                .chars()
                .filter(|c| c.is_alphabetic())
                .all(char::is_uppercase)
    }
}

// ---- 5. Same method name in two traits ------------------------------------------------------

trait Pilot {
    fn fly(&self) -> String;
}

trait Wizard {
    fn fly(&self) -> String;
}

struct Person;

impl Pilot for Person {
    fn fly(&self) -> String {
        String::from("This is your captain speaking.")
    }
}

impl Wizard for Person {
    fn fly(&self) -> String {
        String::from("Up!")
    }
}

impl Person {
    fn fly(&self) -> String {
        String::from("*waving arms furiously*")
    }
}

fn main() {
    println!("1. Supertrait: Printable requires Display");
    println!("    {}", Badge("Rustacean").print_framed());

    println!("\n2. Blanket implementation: every Display type can shout");
    println!("    {}", "hello".shout());
    println!("    {}", 42.shout());
    println!("    {}", Badge("winner").shout());

    println!("\n3. The orphan rule and a newtype");
    println!("    {}", CommaList(vec![3, 1, 4, 1, 5]));

    println!("\n4. Extension trait: new methods on &str");
    for text in ["Rust is fun", "STOP SHOUTING", "1 2 3"] {
        println!(
            "    {text:?}: {} words, shouting: {}",
            text.word_count(),
            text.is_shouting()
        );
    }

    println!("\n5. Same method name in several traits");
    let person = Person;
    println!("    person.fly():          {}", person.fly()); // the type's own method wins
    println!("    Pilot::fly(&person):   {}", Pilot::fly(&person));
    println!("    Wizard::fly(&person):  {}", Wizard::fly(&person));
    println!(
        "    <Person as Wizard>::fly(&person): {}",
        <Person as Wizard>::fly(&person)
    );
    // Without the inherent `impl Person { fn fly }`, `person.fly()` would be ambiguous:
    //     error[E0034]: multiple applicable items in scope
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supertrait_default_uses_display() {
        assert!(Badge("x").print_framed().contains("★ x"));
    }

    #[test]
    fn blanket_impl_covers_many_types() {
        assert_eq!("hi".shout(), "HI!");
        assert_eq!(7.5.shout(), "7.5!");
        assert_eq!(String::from("a b").shout(), "A B!");
    }

    #[test]
    fn newtype_gets_its_own_display() {
        assert_eq!(CommaList(vec![1, 2]).to_string(), "1, 2");
        assert_eq!(CommaList(vec![]).to_string(), "");
    }

    #[test]
    fn extension_trait_methods() {
        assert_eq!("one two  three".word_count(), 3);
        assert!("HEY YOU!".is_shouting());
        assert!(!"Hey".is_shouting());
        assert!(!"123".is_shouting()); // no letters at all
    }

    #[test]
    fn disambiguation() {
        let p = Person;
        assert_eq!(Pilot::fly(&p), "This is your captain speaking.");
        assert_eq!(<Person as Wizard>::fly(&p), "Up!");
        assert_eq!(p.fly(), "*waving arms furiously*");
    }
}
