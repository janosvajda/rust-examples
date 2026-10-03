// Lesson 11: Cow and the borrowing traits.
//
// Four standard-library tools that let code work with borrowed and owned
// data interchangeably:
//   Cow<T>   "clone on write": holds borrowed data, and only makes an owned
//            copy if it actually needs to change it.
//   Borrow   why a HashMap<String, _> can be searched with a &str.
//   AsRef    accept anything that can be viewed as a &str, &Path, &[u8]...
//   Deref    why a String, Box or Vec can be used like the thing inside it.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Deref;
use std::path::Path;

// ---- 1. Cow: borrow when you can, own when you must -------------------------

/// Replaces tabs with spaces. Most text has no tabs, so most of the time we
/// can return the input unchanged (borrowed, no allocation). Only text that
/// really changes gets a new String.
fn normalise_tabs(text: &str) -> Cow<'_, str> {
    if text.contains('\t') {
        Cow::Owned(text.replace('\t', "    "))
    } else {
        Cow::Borrowed(text)
    }
}

// Clippy usually prefers `&str` over `&Cow<str>`, but here we need the Cow
// itself, to see which of its two variants it is.
#[allow(clippy::ptr_arg)]
fn describe(cow: &Cow<str>) -> &'static str {
    match cow {
        Cow::Borrowed(_) => "borrowed, no copy made",
        Cow::Owned(_) => "owned, a new String was made",
    }
}

// ---- 2. AsRef: accept many kinds of input --------------------------------------

/// Accepts &str, String, &String, &Path, PathBuf... anything that can be
/// viewed as a `&Path`.
fn file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref()
        .extension()
        .map(|ext| ext.to_string_lossy().into_owned())
}

/// Accepts anything that can be viewed as a `&str`.
fn shout<S: AsRef<str>>(text: S) -> String {
    text.as_ref().to_uppercase()
}

// ---- 3. Deref: make your own type behave like what it wraps --------------------

/// A list of usernames that always stays sorted. It wraps a Vec, and Deref
/// lets callers use every read-only slice method on it directly.
struct SortedNames(Vec<String>);

impl SortedNames {
    fn new(mut names: Vec<String>) -> Self {
        names.sort();
        SortedNames(names)
    }
}

impl Deref for SortedNames {
    type Target = [String];

    fn deref(&self) -> &[String] {
        &self.0
    }
}
// We deliberately don't implement DerefMut: that would let callers push or
// reorder items and break the "always sorted" promise.

fn main() {
    println!("1. Cow: only copy when something changes");
    for text in ["no tabs here", "one\ttab"] {
        let result = normalise_tabs(text);
        println!("    {:<14} → {:<16} ({})", format!("{text:?}"), format!("{result:?}"), describe(&result));
    }
    // A Cow derefs to &str, so it's used just like one…
    let cleaned = normalise_tabs("a\tb");
    println!("    length: {}", cleaned.len());
    // …and `into_owned` gives a String either way (copying only if borrowed).
    let owned: String = cleaned.into_owned();
    println!("    owned: {owned:?}");

    println!("\n2. Borrow: look up a HashMap<String, _> with a &str");
    let mut stock: HashMap<String, u32> = HashMap::new();
    stock.insert(String::from("apples"), 12);
    // The keys are Strings, but we can search with a plain &str. No String
    // has to be created just to look something up.
    println!("    apples: {:?}", stock.get("apples"));
    println!("    pears:  {:?}", stock.get("pears"));

    println!("\n3. AsRef: one function, many kinds of input");
    let owned_path = String::from("report.pdf");
    println!("    {:?}", file_extension("photo.jpg")); // &str
    println!("    {:?}", file_extension(&owned_path)); // &String
    println!("    {:?}", file_extension(Path::new("notes.txt"))); // &Path
    println!("    {}", shout("hi") + " " + &shout(String::from("there")));

    println!("\n4. Deref: use a wrapper like the thing inside it");
    let names = SortedNames::new(vec![String::from("zoe"), String::from("ana"), String::from("max")]);
    // `len`, `first`, `contains` and `iter` all come from [String] via Deref.
    println!("    {} names, first: {:?}", names.len(), names.first());
    println!("    contains max: {}", names.contains(&String::from("max")));
    println!("    all: {:?}", names.iter().collect::<Vec<_>>());

    println!("\n5. Deref coercion: &Box<String> → &String → &str, automatically");
    let boxed: Box<String> = Box::new(String::from("deeply wrapped"));
    let len = count_chars(&boxed); // `count_chars` takes &str
    println!("    {len} characters");
}

/// Takes a plain `&str`. Thanks to deref coercion it also accepts `&String`,
/// `&Box<String>`, `&Cow<str>`…
fn count_chars(text: &str) -> usize {
    text.chars().count()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cow_borrows_when_nothing_changes() {
        let text = "plain";
        let result = normalise_tabs(text);
        assert!(matches!(result, Cow::Borrowed(_)));
        assert_eq!(result.as_ptr(), text.as_ptr()); // same memory
    }

    #[test]
    fn cow_owns_when_something_changes() {
        let result = normalise_tabs("a\tb");
        assert!(matches!(result, Cow::Owned(_)));
        assert_eq!(result, "a    b");
    }

    #[test]
    fn hashmap_lookup_with_str() {
        let map = HashMap::from([(String::from("k"), 1)]);
        assert_eq!(map.get("k"), Some(&1));
    }

    #[test]
    fn as_ref_accepts_many_types() {
        assert_eq!(file_extension("a.rs"), Some(String::from("rs")));
        assert_eq!(file_extension(String::from("b.md")), Some(String::from("md")));
        assert_eq!(file_extension(Path::new("noext")), None);
        assert_eq!(shout("a"), shout(String::from("a")));
    }

    #[test]
    fn deref_exposes_slice_methods() {
        let names = SortedNames::new(vec![String::from("b"), String::from("a")]);
        assert_eq!(names.len(), 2);
        assert_eq!(names[0], "a"); // indexing works through Deref too
    }

    #[test]
    fn deref_coercion_through_several_layers() {
        let boxed = Box::new(String::from("abc"));
        assert_eq!(count_chars(&boxed), 3);
    }
}
