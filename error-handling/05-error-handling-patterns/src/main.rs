// Lesson 5: everyday error-handling patterns.
//
// The situations that come up all the time in real code, and the short,
// idiomatic way to write each one.

use std::collections::HashMap;
use std::num::ParseIntError;

// ---- 1. Transforming results without `match`: combinators -----------------------

/// `map` changes the Ok value, `map_err` changes the error, `and_then` runs
/// another step that can fail. Each only runs if there's something to work on.
fn parse_celsius_to_fahrenheit(text: &str) -> Result<f64, String> {
    text.trim()
        .parse::<f64>()
        .map_err(|e| format!("\"{text}\" is not a temperature: {e}"))
        .and_then(|c| {
            if c < -273.15 {
                Err(format!("{c} °C is below absolute zero"))
            } else {
                Ok(c)
            }
        })
        .map(|c| c * 9.0 / 5.0 + 32.0)
}

// ---- 2. Many results: stop at the first error, or keep going -------------------

/// `collect` can gather `Result`s into a `Result<Vec<_>, _>`: all values if
/// every item worked, or the FIRST error, stopping right there.
fn parse_all(inputs: &[&str]) -> Result<Vec<i32>, ParseIntError> {
    inputs.iter().map(|s| s.parse::<i32>()).collect()
}

/// When one bad item shouldn't spoil the rest: keep the good values and
/// collect the problems separately.
fn parse_what_you_can(inputs: &[&str]) -> (Vec<i32>, Vec<String>) {
    let mut good = Vec::new();
    let mut problems = Vec::new();
    for input in inputs {
        match input.parse::<i32>() {
            Ok(n) => good.push(n),
            Err(e) => problems.push(format!("{input:?}: {e}")),
        }
    }
    (good, problems)
}

/// Just skip the failures. A Result can be iterated like a list of zero
/// (Err) or one (Ok) items, so `flat_map` keeps only the Ok values.
fn sum_valid(inputs: &[&str]) -> i32 {
    inputs.iter().flat_map(|s| s.parse::<i32>()).sum()
}

// ---- 3. `let … else`: unwrap or leave, in one line -------------------------------

/// `let Some(x) = … else { … }` binds the value, or runs the else block, which
/// must leave the function (return, break, continue or panic).
fn greeting(users: &HashMap<u32, String>, id: u32) -> String {
    let Some(name) = users.get(&id) else {
        return String::from("Hello, guest!");
    };
    format!("Hello, {name}!")
}

/// Let chains: several `let` patterns and conditions in one `if`, joined with `&&`.
/// Each step runs only if the one before it succeeded, and every binding
/// (`text`, `port`) can be used in the steps after it and in the body.
fn admin_port(settings: &HashMap<&str, &str>) -> Option<u16> {
    if let Some(text) = settings.get("admin_port")
        && let Ok(port) = text.trim().parse::<u16>()
        && port >= 1024
    {
        Some(port)
    } else {
        None
    }
}

/// `if let` guards: a match arm that applies only when one more pattern matches.
/// If the guard's pattern fails, matching simply continues with the next arm.
fn run_command(input: &str) -> String {
    match input.split_once(' ') {
        Some(("wait", seconds)) if let Ok(n) = seconds.parse::<u32>() => format!("waiting {n} s"),
        Some(("wait", seconds)) => format!("`{seconds}` is not a number of seconds"),
        Some((command, _)) => format!("unknown command `{command}`"),
        None => format!("`{input}` needs an argument"),
    }
}

// ---- 4. Fallbacks: default values and alternatives ------------------------------

fn port_from(env_value: Option<&str>) -> u16 {
    env_value
        .and_then(|v| v.parse().ok()) // Result → Option, dropping the error on purpose
        .unwrap_or(8080) // fall back to a default
}

/// Try one source, then another. `or` uses `backup` only if `primary` is None.
fn find_config(primary: Option<&str>, backup: Option<&str>) -> Result<String, String> {
    primary
        .or(backup)
        .map(str::to_string)
        .ok_or_else(|| String::from("no configuration found anywhere"))
}

// ---- 5. Retry an operation that can fail temporarily ------------------------------

/// Tries `operation` up to `attempts` times. Returns the first success, or
/// the last error if every attempt failed.
fn retry<T, E>(attempts: u32, mut operation: impl FnMut(u32) -> Result<T, E>) -> Result<T, E> {
    let mut attempt = 1;
    loop {
        match operation(attempt) {
            Ok(value) => return Ok(value),
            Err(error) if attempt >= attempts => return Err(error),
            Err(_) => attempt += 1,
        }
    }
}

fn main() {
    println!("1. Combinators: map, map_err, and_then");
    for input in ["100", "-300", "hot"] {
        match parse_celsius_to_fahrenheit(input) {
            Ok(f) => println!("    {input:>5} °C → {f} °F"),
            Err(e) => println!("    {input:>5} → error: {e}"),
        }
    }

    println!("\n2. Many results");
    let inputs = ["10", "x", "30", "y"];
    println!("    stop at the first error: {:?}", parse_all(&inputs).map_err(|e| e.to_string()));
    println!("    all good:                {:?}", parse_all(&["1", "2"]));
    let (good, problems) = parse_what_you_can(&inputs);
    println!("    keep going: good = {good:?}, problems = {problems:?}");
    println!("    skip failures, sum the rest: {}", sum_valid(&inputs));

    println!("\n3. let … else, let chains, if let guards");
    let users = HashMap::from([(1, String::from("Ana"))]);
    println!("    {}", greeting(&users, 1));
    println!("    {}", greeting(&users, 2));
    let settings = HashMap::from([("admin_port", "8443"), ("name", "demo")]);
    println!("    let chain, admin_port: {:?}", admin_port(&settings));
    println!("    let chain, not set:    {:?}", admin_port(&HashMap::new()));
    for input in ["wait 5", "wait soon", "jump 3", "wait"] {
        println!("    if let guard: {input:>9} → {}", run_command(input));
    }

    println!("\n4. Fallbacks");
    println!("    port from \"9000\": {}", port_from(Some("9000")));
    println!("    port from \"oops\": {}", port_from(Some("oops")));
    println!("    port from nothing: {}", port_from(None));
    println!("    {:?}", find_config(None, Some("backup.toml")));
    println!("    {:?}", find_config(None, None));

    println!("\n5. Retry");
    let result = retry(5, |attempt| {
        println!("    attempt {attempt}…");
        if attempt < 3 { Err("server busy") } else { Ok("connected") }
    });
    println!("    result: {result:?}");
    let gave_up: Result<(), &str> = retry(2, |_| Err("still down"));
    println!("    giving up after 2: {gave_up:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combinators_chain_steps() {
        assert_eq!(parse_celsius_to_fahrenheit("100"), Ok(212.0));
        assert!(parse_celsius_to_fahrenheit("-300").unwrap_err().contains("absolute zero"));
        assert!(parse_celsius_to_fahrenheit("hot").unwrap_err().contains("not a temperature"));
    }

    #[test]
    fn collect_stops_at_first_error() {
        assert_eq!(parse_all(&["1", "2", "3"]), Ok(vec![1, 2, 3]));
        assert!(parse_all(&["1", "x", "3"]).is_err());
    }

    #[test]
    fn partition_keeps_good_and_bad() {
        let (good, bad) = parse_what_you_can(&["1", "x", "2"]);
        assert_eq!(good, vec![1, 2]);
        assert_eq!(bad.len(), 1);
    }

    #[test]
    fn flatten_skips_failures() {
        assert_eq!(sum_valid(&["5", "nope", "10"]), 15);
    }

    #[test]
    fn let_else_falls_back() {
        let users = HashMap::new();
        assert_eq!(greeting(&users, 9), "Hello, guest!");
    }

    #[test]
    fn let_chain_needs_every_step_to_succeed() {
        assert_eq!(admin_port(&HashMap::from([("admin_port", "8443")])), Some(8443));
        assert_eq!(admin_port(&HashMap::from([("admin_port", "80")])), None); // below 1024
        assert_eq!(admin_port(&HashMap::from([("admin_port", "high")])), None); // not a number
        assert_eq!(admin_port(&HashMap::new()), None); // missing
    }

    #[test]
    fn if_let_guard_falls_through_to_the_next_arm() {
        assert_eq!(run_command("wait 5"), "waiting 5 s");
        assert_eq!(run_command("wait soon"), "`soon` is not a number of seconds");
        assert_eq!(run_command("jump 3"), "unknown command `jump`");
        assert_eq!(run_command("wait"), "`wait` needs an argument");
    }

    #[test]
    fn fallbacks() {
        assert_eq!(port_from(Some("123")), 123);
        assert_eq!(port_from(Some("bad")), 8080);
        assert_eq!(find_config(Some("a"), Some("b")), Ok(String::from("a")));
        assert!(find_config(None, None).is_err());
    }

    #[test]
    fn retry_returns_first_success_or_last_error() {
        let mut calls = 0;
        let ok: Result<u32, &str> = retry(5, |n| {
            calls += 1;
            if n == 2 { Ok(n) } else { Err("no") }
        });
        assert_eq!(ok, Ok(2));
        assert_eq!(calls, 2);

        let failed: Result<(), String> = retry(3, |n| Err(format!("fail {n}")));
        assert_eq!(failed, Err(String::from("fail 3")));
    }
}
