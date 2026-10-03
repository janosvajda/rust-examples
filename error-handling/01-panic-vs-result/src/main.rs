// Lesson 1: panic vs Result.
//
// Rust has two ways to deal with things going wrong:
//   panic!     for bugs: something that should never happen. Stops the thread.
//   Result     for expected failures: bad input, missing file, network down.
//              The caller decides what to do.

use std::num::ParseIntError;

// ---- Result: the failure is part of the function's signature ---------------

/// Parsing user input can fail for perfectly normal reasons, so the
/// signature says so: you get either a `u16` or a `ParseIntError`.
fn parse_port(text: &str) -> Result<u16, ParseIntError> {
    text.trim().parse::<u16>()
}

/// Division by zero is an expected case here, so it's a Result too.
/// `String` is the simplest possible error type: just a message.
fn divide(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err(format!("cannot divide {a} by zero"))
    } else {
        Ok(a / b)
    }
}

// ---- Option: "might be missing" is not an error -----------------------------

/// Looking for something that isn't there is normal, not a failure.
/// `Option` says "maybe there's a value" without saying why not.
fn find_user(id: u32) -> Option<&'static str> {
    match id {
        1 => Some("Ana"),
        2 => Some("Bob"),
        _ => None,
    }
}

// ---- panic!: for broken assumptions, not bad input ---------------------------

/// The caller promised `percent` is between 0 and 100. If it isn't, the
/// caller has a bug, and continuing would only spread wrong numbers, so we
/// stop. `assert!` panics with the message if the condition is false.
fn apply_discount(price_cents: u32, percent: u32) -> u32 {
    assert!(percent <= 100, "discount must be 0..=100, got {percent}");
    price_cents - price_cents * percent / 100
}

fn main() {
    println!("1. Handling a Result with match");
    for input in ["8080", "eighty", "70000"] {
        match parse_port(input) {
            Ok(port) => println!("    {input:>7} → port {port}"),
            Err(error) => println!("    {input:>7} → not a port: {error}"),
        }
    }

    println!("\n2. A Result with a String error");
    println!("    {:?}", divide(10, 2));
    println!("    {:?}", divide(10, 0));

    println!("\n3. Option: missing is not an error");
    for id in [1, 7] {
        match find_user(id) {
            Some(name) => println!("    user {id} is {name}"),
            None => println!("    no user {id}"),
        }
    }

    println!("\n4. Shortcuts that panic: unwrap and expect");
    // `unwrap` returns the value, or panics on Err/None. Fine in tests and
    // quick experiments; in real code it turns bad input into a crash.
    let port = parse_port("443").unwrap();
    println!("    unwrap on Ok: {port}");
    // parse_port("4x3").unwrap();
    //   thread 'main' panicked at …:
    //   called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
    //
    // `expect` is the same, but you write the panic message. Use it to say
    // WHY you were sure it couldn't fail.
    let fixed = parse_port("80").expect("80 is a valid port");
    println!("    expect on Ok: {fixed}");

    println!("\n5. Safe fallbacks that never panic");
    println!("    unwrap_or:         {}", parse_port("oops").unwrap_or(8080));
    println!("    unwrap_or_default: {}", parse_port("oops").unwrap_or_default());
    println!("    is_ok / is_err:    {} / {}", parse_port("1").is_ok(), parse_port("x").is_err());

    println!("\n6. A panic for a broken promise");
    println!("    20% off 1000 cents = {}", apply_discount(1000, 20));
    // apply_discount(1000, 150) would panic:
    //   discount must be 0..=100, got 150
    println!("    (apply_discount(1000, 150) would panic: see the tests)");

    // Ignoring a Result gives a compiler warning:
    // divide(1, 0);
    //   warning: unused `Result` that must be used
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_port_accepts_valid_numbers() {
        assert_eq!(parse_port(" 8080 "), Ok(8080));
    }

    #[test]
    fn parse_port_rejects_bad_input() {
        assert!(parse_port("eighty").is_err());
        assert!(parse_port("70000").is_err()); // too big for u16
    }

    #[test]
    fn divide_by_zero_is_an_error_not_a_crash() {
        assert_eq!(divide(10, 0), Err(String::from("cannot divide 10 by zero")));
    }

    #[test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value")]
    fn unwrap_on_err_panics() {
        parse_port("4x3").unwrap();
    }

    #[test]
    #[should_panic(expected = "discount must be 0..=100, got 150")]
    fn broken_promise_panics() {
        apply_discount(1000, 150);
    }
}
