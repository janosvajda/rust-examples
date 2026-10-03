// Lesson 10: advanced lifetimes.
//
// Lesson 5 covered the basics. This lesson covers the situations where the
// default rules give the wrong answer, and how to say what you really mean.

use std::fmt::Display;

// ---- 1. A method's result can borrow from the data, not from `self` ---------

/// Points into some text, without copying it.
struct Excerpt<'text> {
    part: &'text str,
}

impl<'text> Excerpt<'text> {
    fn new(text: &'text str) -> Self {
        Excerpt {
            part: text.split('.').next().unwrap_or(""),
        }
    }

    /// Written as `fn part(&self) -> &str`, the elision rules would tie the
    /// result to `self`, the short-lived Excerpt. But the text really comes
    /// from the original string, so we say so with `'text`.
    fn part(&self) -> &'text str {
        self.part
    }
}

/// Creates a temporary Excerpt and returns text from it. This only compiles
/// because `part()` returns `&'text str`. With `-> &str`:
///     error[E0515]: cannot return value referencing temporary value
fn first_sentence(text: &str) -> &str {
    Excerpt::new(text).part()
}

// ---- 2. Two references with different lifetimes in one struct ---------------

/// A lookup result: the key we searched for, and the value we found.
/// They come from different places with different lifetimes, so each gets
/// its own parameter. With a single `'a` for both, the result would only be
/// usable while BOTH sources are alive.
struct Found<'key, 'data> {
    key: &'key str,
    value: &'data str,
}

fn find<'key, 'data>(key: &'key str, table: &'data [(String, String)]) -> Option<Found<'key, 'data>> {
    table
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| Found { key, value: v })
}

// ---- 3. Trait objects borrow too: `dyn Trait + 'a` ------------------------------

/// `Box<dyn Fn(&str) -> String>` means `Box<dyn Fn(&str) -> String + 'static>`
/// by default: the closure may not borrow anything temporary. This closure
/// borrows `prefix`, so we say the box only lives as long as `prefix`:
///     without `+ 'a`: error: lifetime may not live long enough
fn make_labeler<'a>(prefix: &'a str) -> Box<dyn Fn(&str) -> String + 'a> {
    Box::new(move |text| format!("{prefix}{text}"))
}

// ---- 4. What `impl Trait` borrows (Rust 2024) ------------------------------------

struct Counter {
    step: u32,
}

impl Counter {
    /// In Rust 2024 a returned `impl Trait` is assumed to borrow EVERY
    /// reference parameter, including `&self`. This iterator only copies
    /// `step`, so `+ use<>` says "I borrow nothing". Without it, changing
    /// the counter while the iterator exists would fail:
    ///     error[E0506]: cannot assign to `counter.step` because it is borrowed
    fn multiples(&self, count: u32) -> impl Iterator<Item = u32> + use<> {
        let step = self.step;
        (0..count).map(move |i| i * step)
    }

    /// This one really does borrow `items`, so the default (borrow
    /// everything) is right and nothing extra needs to be written.
    fn labelled<'a>(&self, items: &'a [&'a str]) -> impl Iterator<Item = String> {
        items.iter().map(|item| format!("<{item}>"))
    }
}

// ---- 5. `T: 'a`: a type that contains references must outlive the borrow ----

/// Holds a reference to a `T`. For that reference to be valid, every
/// reference *inside* T must live at least as long as `'a`. Rust usually
/// works this out on its own, but you'll see `T: 'a` in signatures.
struct Labelled<'a, T: Display + 'a> {
    label: &'a str,
    value: &'a T,
}

impl<'a, T: Display + 'a> Labelled<'a, T> {
    fn show(&self) -> String {
        format!("{} = {}", self.label, self.value)
    }
}

// ---- 6. Closures that take references: works for any lifetime ---------------------

/// `F: Fn(&str) -> usize` really means `F: for<'x> Fn(&'x str) -> usize`:
/// the closure must work for a reference of ANY lifetime, not one fixed one.
/// This is called a higher-ranked trait bound. Rust adds the `for<'x>` for you.
fn total_by<F: Fn(&str) -> usize>(words: &[String], measure: F) -> usize {
    words.iter().map(|w| measure(w)).sum()
}

fn main() {
    println!("1. Return data's lifetime, not self's");
    let text = String::from("Rust is fast. Rust is safe.");
    println!("    first sentence: \"{}\"", first_sentence(&text));

    println!("\n2. Two lifetimes in one struct");
    let table = vec![
        (String::from("lang"), String::from("Rust")),
        (String::from("year"), String::from("2015")),
    ];
    let value;
    {
        let key = String::from("lang"); // dropped at the end of this block
        let found = find(&key, &table).unwrap();
        println!("    {} → {}", found.key, found.value);
        value = found.value; // outlives `key`, because it only borrows `table`
    }
    println!("    still usable after the key is gone: {value}");

    println!("\n3. A boxed closure that borrows: dyn Fn + 'a");
    let prefix = String::from("[note] ");
    let label = make_labeler(&prefix);
    println!("    {}", label("borrowed prefix"));

    println!("\n4. impl Trait + use<>: an iterator that doesn't borrow");
    let mut counter = Counter { step: 2 };
    let multiples = counter.multiples(4);
    counter.step = 5; // allowed: `multiples` doesn't borrow `counter`
    println!("    {:?} (step is now {})", multiples.collect::<Vec<_>>(), counter.step);
    let words = ["a", "b"];
    println!("    {:?}", counter.labelled(&words).collect::<Vec<_>>());

    println!("\n5. T: 'a");
    let price = 9.99;
    println!("    {}", Labelled { label: "price", value: &price }.show());

    println!("\n6. Closures over references of any lifetime");
    let words = vec![String::from("borrow"), String::from("checker")];
    println!("    total letters: {}", total_by(&words, |w| w.len()));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn result_outlives_the_temporary_excerpt() {
        let text = String::from("One. Two.");
        let first = first_sentence(&text);
        assert_eq!(first, "One");
    }

    #[test]
    fn found_value_outlives_the_key() {
        let table = vec![(String::from("k"), String::from("v"))];
        let value = {
            let key = String::from("k");
            find(&key, &table).unwrap().value
        };
        assert_eq!(value, "v");
        assert!(find("missing", &table).is_none());
    }

    #[test]
    fn boxed_closure_can_borrow() {
        let prefix = String::from(">> ");
        let label = make_labeler(&prefix);
        assert_eq!(label("x"), ">> x");
    }

    #[test]
    fn use_nothing_iterator_does_not_borrow() {
        let mut counter = Counter { step: 3 };
        let it = counter.multiples(3);
        counter.step = 100; // allowed while `it` still exists
        assert_eq!(it.collect::<Vec<_>>(), vec![0, 3, 6]); // uses the old step
        assert_eq!(counter.step, 100);
    }

    #[test]
    fn higher_ranked_closure() {
        let words = vec![String::from("ab"), String::from("cde")];
        assert_eq!(total_by(&words, |w| w.len()), 5);
        assert_eq!(total_by(&words, |_| 1), 2);
    }
}
