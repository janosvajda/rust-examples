// Lesson 2: the `?` operator.
//
// `value?` means: if this is Ok(v), give me v and carry on; if it's Err(e),
// return Err(e) from the current function right now. It's how errors travel
// up through your code without a `match` at every step.

use std::collections::HashMap;
use std::error::Error;
use std::num::ParseIntError;

// ---- 1. Without `?` and with `?` --------------------------------------------------

/// The long way: a `match` for every step that can fail.
// Clippy suggests `?` here, which is exactly the point of this lesson:
// this version is written out on purpose, to show what `?` replaces.
#[allow(clippy::question_mark)]
fn add_strings_long(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x = match a.trim().parse::<i32>() {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    let y = match b.trim().parse::<i32>() {
        Ok(value) => value,
        Err(error) => return Err(error),
    };
    Ok(x + y)
}

/// Exactly the same behaviour, written with `?`.
fn add_strings(a: &str, b: &str) -> Result<i32, ParseIntError> {
    let x: i32 = a.trim().parse()?;
    let y: i32 = b.trim().parse()?;
    Ok(x + y)
}

// ---- 2. `?` chains through several functions ---------------------------------------

/// Parses lines like "width=80". Each step can fail; `?` passes any error
/// straight up to whoever called `parse_settings`.
fn parse_line(line: &str) -> Result<(String, u32), String> {
    let (key, value) = line
        .split_once('=')
        .ok_or(format!("missing '=' in \"{line}\""))?; // Option → Result, then `?`
    let number = value
        .trim()
        .parse::<u32>()
        .map_err(|e| format!("bad number for {key}: {e}"))?; // change the error type, then `?`
    Ok((key.trim().to_string(), number))
}

fn parse_settings(text: &str) -> Result<HashMap<String, u32>, String> {
    let mut settings = HashMap::new();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let (key, value) = parse_line(line)?; // the first bad line stops everything
        settings.insert(key, value);
    }
    Ok(settings)
}

// ---- 3. `?` works on Option too ------------------------------------------------------

/// In a function returning `Option`, `?` on an Option returns None early.
fn initials(full_name: &str) -> Option<String> {
    let mut words = full_name.split_whitespace();
    let first = words.next()?.chars().next()?;
    let last = words.last()?.chars().next()?;
    Some(format!("{first}.{last}."))
}

// ---- 4. Different error types: Box<dyn Error> ---------------------------------------

/// Reads a number from a file. Two different things can fail, with two
/// different error types (io::Error and ParseIntError). `Box<dyn Error>`
/// accepts any error type, and `?` converts automatically.
///
/// With `-> Result<i32, ParseIntError>` instead, the file read would fail:
///     error[E0277]: `?` couldn't convert the error to `ParseIntError`
fn read_number(path: &str) -> Result<i32, Box<dyn Error>> {
    let text = std::fs::read_to_string(path)?; // io::Error → Box<dyn Error>
    let number = text.trim().parse::<i32>()?; // ParseIntError → Box<dyn Error>
    Ok(number)
}

fn main() -> Result<(), Box<dyn Error>> {
    // `main` itself can return a Result, so `?` works here too. If main
    // returns Err, Rust prints "Error: …" and exits with code 1.

    println!("1. Same function, with and without `?`");
    println!("    long:  {:?}", add_strings_long("2", "40"));
    println!("    short: {:?}", add_strings("2", "40"));
    println!("    short: {:?}", add_strings("2", "forty"));

    println!("\n2. Errors travel up through several functions");
    println!("    {:?}", parse_settings("width=80\nheight=24"));
    println!("    {:?}", parse_settings("width=80\nheight"));
    println!("    {:?}", parse_settings("width=80\nheight=tall"));

    println!("\n3. `?` on Option");
    println!("    {:?}", initials("Grace Brewster Hopper"));
    println!("    {:?}", initials("Plato"));

    println!("\n4. Different error types in one function");
    let path = std::env::temp_dir().join("rust-examples-number.txt");
    std::fs::write(&path, "42\n")?; // `?` in main
    println!("    from a good file: {:?}", read_number(path.to_str().unwrap())?);
    match read_number("/no/such/file.txt") {
        Ok(n) => println!("    {n}"),
        Err(error) => println!("    from a missing file: error: {error}"),
    }
    std::fs::remove_file(&path)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_versions_behave_the_same() {
        for (a, b) in [("1", "2"), ("x", "2"), ("1", "y")] {
            assert_eq!(add_strings(a, b), add_strings_long(a, b));
        }
    }

    #[test]
    fn first_bad_line_stops_parsing() {
        let error = parse_settings("a=1\nb\nc=oops").unwrap_err();
        assert_eq!(error, "missing '=' in \"b\"");
    }

    #[test]
    fn good_settings_are_collected() {
        let settings = parse_settings("a=1\n\nb = 2").unwrap();
        assert_eq!(settings["a"], 1);
        assert_eq!(settings["b"], 2);
    }

    #[test]
    fn question_mark_on_option() {
        assert_eq!(initials("Ada Lovelace"), Some(String::from("A.L.")));
        assert_eq!(initials("Ada"), None); // no last name
        assert_eq!(initials(""), None);
    }

    #[test]
    fn missing_file_is_an_error() {
        assert!(read_number("/no/such/file.txt").is_err());
    }

    #[test]
    fn tests_can_use_question_mark_too() -> Result<(), String> {
        // A test can return Result; an Err fails the test.
        let settings = parse_settings("x=5")?;
        assert_eq!(settings["x"], 5);
        Ok(())
    }
}
