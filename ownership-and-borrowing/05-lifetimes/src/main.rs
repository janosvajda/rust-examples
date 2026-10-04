// Lesson 5: lifetimes.
//
// A reference must never outlive the value it points to. The compiler checks
// this by tracking the region for which each reference must remain valid.
// Uses through copied references or destructors can also require validity.
//
// Many lifetimes are inferred. Signature elision rules fill in common
// relationships; explicit annotations (`'a`) describe other relationships.

// ---- 1. Returning a reference: which input does it come from? ---------------

// fn longest(a: &str, b: &str) -> &str {
//     if a.len() >= b.len() { a } else { b }
// }
// error[E0106]: missing lifetime specifier
//
// The compiler can't tell whether the result borrows from `a` or from `b`,
// so it can't check the caller's code. `'a` says: the result lives no
// longer than BOTH inputs.
/// Chooses the input with more UTF-8 bytes; ties choose `a`.
fn longest<'a>(a: &'a str, b: &'a str) -> &'a str {
    if a.len() >= b.len() { a } else { b }
}

/// Here the result only ever comes from `text`, so only `text` needs `'a`.
/// `prefix` can live as long or as short as it likes.
fn after_prefix<'a>(text: &'a str, prefix: &str) -> &'a str {
    text.strip_prefix(prefix).unwrap_or(text)
}

// ---- 2. No annotation needed: the elision rules ---------------------------------
//
// With exactly one input lifetime, the compiler assigns it to the elided
// output lifetime. Here the result borrows
// from it. Writing `fn first_line<'a>(text: &'a str) -> &'a str` would mean
// exactly the same thing.
fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

// ---- 3. You can't return a reference to something you're about to drop -------
//
// fn make_greeting(name: &str) -> &str {
//     let greeting = format!("Hello, {name}!");
//     &greeting
// }
// error[E0515]: cannot return reference to local variable `greeting`
//
// `greeting` is dropped when the function returns. Return the owned value instead:
fn make_greeting(name: &str) -> String {
    format!("Hello, {name}!")
}

// ---- 4. A struct that holds a reference ------------------------------------------

/// Borrows part of some text instead of copying it. The `'a` means a
/// `Highlight` can't outlive the text it points into.
struct Highlight<'a> {
    text: &'a str,
}

impl<'a> Highlight<'a> {
    fn new(text: &'a str) -> Self {
        Highlight { text }
    }

    fn word_count(&self) -> usize {
        self.text.split_whitespace().count()
    }
}

// ---- 5. 'static: references that are valid for the whole program ------------

/// String literals are stored in the program itself, so they never go away.
/// A reference to one is valid everywhere, for as long as the program runs.
fn app_name() -> &'static str {
    "borrow-checker-demo"
}

fn main() {
    println!("1. A lifetime annotation links the output to the inputs");
    let first = String::from("a long sentence");
    let result;
    {
        let second = String::from("short");
        result = longest(&first, &second);
        println!("    longest = {result}");
    }
    // println!("{result}");
    // error[E0597]: `second` does not live long enough
    //   `result` might point into `second`, which was dropped at the `}`.
    println!("    (using `result` after the block wouldn't compile)");

    println!("\n2. Only the inputs that matter need the lifetime");
    let url = String::from("https://example.com");
    let host = {
        let scheme = String::from("https://");
        after_prefix(&url, &scheme) // `scheme` dies here, but `host` doesn't borrow it
    };
    println!("    host = {host}");

    println!("\n3. Elision: one input reference, no annotation needed");
    println!("    first line = {}", first_line("line one\nline two"));

    println!("\n4. Return owned data when it's created inside the function");
    println!("    {}", make_greeting("Ferris"));

    println!("\n5. A struct can't outlive the data it borrows");
    let article = String::from("Rust is fast. Rust is safe.");
    let highlight = Highlight::new(&article[..13]);
    println!(
        "    highlight = \"{}\" ({} words)",
        highlight.text,
        highlight.word_count()
    );
    // let h;
    // {
    //     let temporary = String::from("gone soon");
    //     h = Highlight::new(&temporary);
    // }
    // println!("{}", h.text);
    // error[E0597]: `temporary` does not live long enough

    println!("\n6. 'static: lives for the whole program");
    println!("    app name = {}", app_name());

    println!("\n7. The classic dangling reference is caught too");
    // let r;
    // {
    //     let x = 5;
    //     r = &x;
    // }
    // println!("{r}");
    // error[E0597]: `x` does not live long enough
    println!("    (see the comment in the code)");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longest_returns_the_longer_input() {
        assert_eq!(longest("abc", "de"), "abc");
        assert_eq!(longest("a", "bc"), "bc");
        assert_eq!(longest("ab", "cd"), "ab"); // ties go to the first
        assert_eq!(longest("éé", "abc"), "éé"); // four UTF-8 bytes beat three
    }

    #[test]
    fn result_can_outlive_the_unrelated_input() {
        let text = String::from("v1.2.3");
        let version = {
            let prefix = String::from("v");
            after_prefix(&text, &prefix)
        }; // `prefix` is gone; `version` only borrows `text`
        assert_eq!(version, "1.2.3");
    }

    #[test]
    fn struct_borrows_without_copying() {
        let source = String::from("one two three");
        let h = Highlight::new(&source[4..]);
        assert_eq!(h.word_count(), 2);
        assert_eq!(h.text.as_ptr(), source[4..].as_ptr());
    }

    #[test]
    fn owned_results_have_no_lifetime_to_worry_about() {
        let greeting = make_greeting("you");
        assert_eq!(greeting, "Hello, you!");
    }
}
