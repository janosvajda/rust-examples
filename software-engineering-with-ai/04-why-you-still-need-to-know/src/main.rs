// Lesson 4: why you still need to know what is what.
//
// Each example has two versions of the same function:
//   - `plausible`: the kind of code an AI assistant easily produces. It compiles,
//     it works on the obvious input, and it looks fine at a glance.
//   - `better`: what someone who knows the language and the problem would write.
//
// Only knowledge tells them apart. The tests show where the plausible versions break.

use std::time::Instant;

/// The first draft: compiles, passes a quick try, looks fine.
mod plausible {
    /// Average of some numbers.
    pub fn average(numbers: &[i32]) -> i32 {
        numbers.iter().sum::<i32>() / numbers.len() as i32
    }

    /// Does the list contain the same value twice? Compares every pair.
    pub fn has_duplicates(values: &[u64]) -> bool {
        for i in 0..values.len() {
            for j in (i + 1)..values.len() {
                if values[i] == values[j] {
                    return true;
                }
            }
        }
        false
    }

    /// The longest name. Clones every new leader, to keep the borrow checker quiet.
    pub fn longest_name(names: &[String]) -> String {
        let mut longest = String::new();
        for name in names {
            if name.len() > longest.len() {
                longest = name.clone();
            }
        }
        longest
    }

    /// Read the port from a config line like `port=8080`.
    pub fn parse_port(line: &str) -> u16 {
        line.split('=').nth(1).unwrap().parse().unwrap()
    }
}

/// The same four functions, written with knowledge of the problem.
mod better {
    use std::collections::HashSet;

    /// An empty list has no average, so say so with `None`. Sum in `i64` so the total
    /// can't overflow, and divide as floating point so 1 and 2 average to 1.5, not 1.
    pub fn average(numbers: &[i32]) -> Option<f64> {
        if numbers.is_empty() {
            return None;
        }
        let sum: i64 = numbers.iter().map(|&n| i64::from(n)).sum();
        Some(sum as f64 / numbers.len() as f64)
    }

    /// Remember what we've seen in a hash set: one pass instead of comparing every pair.
    pub fn has_duplicates(values: &[u64]) -> bool {
        let mut seen = HashSet::with_capacity(values.len());
        // `insert` returns false when the value was already there
        !values.iter().all(|v| seen.insert(v))
    }

    /// Return a borrowed `&str`: no copying at all. `None` for an empty list,
    /// which is different from a list whose longest name is "".
    pub fn longest_name(names: &[String]) -> Option<&str> {
        names
            .iter()
            .max_by_key(|name| name.len())
            .map(String::as_str)
    }

    /// Bad input is normal for a config file, so return an error that says what's wrong.
    pub fn parse_port(line: &str) -> Result<u16, String> {
        let (key, value) = line
            .split_once('=')
            .ok_or(format!("expected `port=<number>`, got `{line}`"))?;
        if key.trim() != "port" {
            return Err(format!("expected the key `port`, got `{}`", key.trim()));
        }
        value
            .trim()
            .parse()
            .map_err(|_| format!("`{}` is not a port number (0-65535)", value.trim()))
    }
}

fn main() {
    println!("1. Average");
    println!(
        "    plausible: average of [1, 2] = {}",
        plausible::average(&[1, 2])
    );
    println!(
        "    better:    average of [1, 2] = {:?}",
        better::average(&[1, 2])
    );
    println!(
        "    better:    average of []     = {:?}",
        better::average(&[])
    );
    println!("    (plausible::average(&[]) would panic: attempt to divide by zero)");

    println!("\n2. Duplicates: the same answer, a very different amount of work");
    for n in [5_000u64, 10_000, 20_000] {
        let values: Vec<u64> = (0..n).collect(); // no duplicates: the worst case
        let start = Instant::now();
        let slow = plausible::has_duplicates(&values);
        let slow_time = start.elapsed();
        let start = Instant::now();
        let fast = better::has_duplicates(&values);
        let fast_time = start.elapsed();
        assert_eq!(slow, fast);
        println!("    {n:>6} values: plausible {slow_time:>10.2?}   better {fast_time:>10.2?}");
    }
    println!(
        "    Doubling the input makes the plausible version about 4x slower, the better one about 2x."
    );

    println!("\n3. Longest name");
    let names = vec![
        String::from("Ada"),
        String::from("Grace"),
        String::from("Linus"),
    ];
    println!(
        "    plausible: {:?} (a new copy of the text)",
        plausible::longest_name(&names)
    );
    println!(
        "    better:    {:?} (borrowed, no copy)",
        better::longest_name(&names)
    );
    println!(
        "    \"Grace\" and \"Linus\" are equally long: one version keeps the first, the other the last."
    );
    println!("    Which is right? Only the specification can say, and nobody wrote one.");
    println!(
        "    empty list: plausible {:?}, better {:?}",
        plausible::longest_name(&[]),
        better::longest_name(&[])
    );

    println!("\n4. Parsing a config line");
    println!(
        "    plausible::parse_port(\"port=8080\") = {}",
        plausible::parse_port("port=8080")
    );
    for line in ["port=8080", "port = 8080", "port=abc", "port"] {
        println!(
            "    better::parse_port({line:?}) = {:?}",
            better::parse_port(line)
        );
    }
    println!(
        "    plausible::parse_port works only for the first one; it panics on the other three."
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- 1. average ----

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn plausible_average_panics_on_an_empty_list() {
        plausible::average(&[]);
    }

    #[test]
    fn plausible_average_throws_away_the_fraction() {
        assert_eq!(plausible::average(&[1, 2]), 1); // the real average is 1.5
    }

    #[test]
    fn better_average() {
        assert_eq!(better::average(&[1, 2]), Some(1.5));
        assert_eq!(better::average(&[]), None);
        assert_eq!(
            better::average(&[i32::MAX, i32::MAX]),
            Some(i32::MAX as f64)
        ); // no overflow
    }

    // ---- 2. duplicates ----

    #[test]
    fn both_duplicate_checks_agree() {
        for values in [vec![], vec![1], vec![1, 2, 3], vec![1, 2, 1], vec![7, 7]] {
            assert_eq!(
                plausible::has_duplicates(&values),
                better::has_duplicates(&values)
            );
        }
    }

    // ---- 3. longest name ----

    #[test]
    fn longest_name() {
        let names = vec![String::from("Ada"), String::from("Grace")];
        assert_eq!(plausible::longest_name(&names), "Grace");
        assert_eq!(better::longest_name(&names), Some("Grace"));
        // a tie: the plausible version keeps the first, max_by_key keeps the last
        let tie = vec![String::from("Grace"), String::from("Linus")];
        assert_eq!(plausible::longest_name(&tie), "Grace");
        assert_eq!(better::longest_name(&tie), Some("Linus"));
        // the plausible version can't tell "no names" from "the longest name is empty"
        assert_eq!(plausible::longest_name(&[]), "");
        assert_eq!(better::longest_name(&[]), None);
    }

    // ---- 4. parsing ----

    #[test]
    #[should_panic]
    fn plausible_parse_panics_on_spaces() {
        plausible::parse_port("port = 8080");
    }

    #[test]
    fn better_parse_handles_real_input() {
        assert_eq!(better::parse_port("port=8080"), Ok(8080));
        assert_eq!(better::parse_port("port = 8080"), Ok(8080));
        assert!(
            better::parse_port("port=abc")
                .unwrap_err()
                .contains("not a port number")
        );
        assert!(better::parse_port("port=70000").is_err()); // too big for a port
        assert!(
            better::parse_port("host=example.com")
                .unwrap_err()
                .contains("expected the key `port`")
        );
        assert!(better::parse_port("port").is_err());
    }
}
