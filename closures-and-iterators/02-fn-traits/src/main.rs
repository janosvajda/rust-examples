// Lesson 2: Fn, FnMut and FnOnce.
//
// Every closure implements one or more of three traits, depending on what
// it does with the things it captured:
//   Fn       only reads them        → can be called any number of times, even at once
//   FnMut    changes them           → can be called many times, one call at a time
//   FnOnce   gives them away        → can be called once
// A function that TAKES a closure picks the trait it needs, and the most
// flexible choice for callers is the least demanding one.

/// Calls `f` twice: needs `Fn`, because it may call it more than once.
fn call_twice<F: Fn() -> String>(f: F) -> String {
    format!("{} / {}", f(), f())
}

/// Calls `f` for every item: `FnMut` is enough, since calls happen one at a time.
fn for_each_item<F: FnMut(i32)>(items: &[i32], mut f: F) {
    for &item in items {
        f(item);
    }
}

/// Calls `f` exactly once: `FnOnce` accepts ANY closure, even ones that
/// give away what they captured.
fn run_once<F: FnOnce() -> String>(f: F) -> String {
    f()
}

/// A real-world example: `Option::unwrap_or_else` takes `FnOnce`, because
/// it calls the closure at most once. So the closure is allowed to use up
/// what it captured: this one moves `fallback` into a new variable.
fn name_or_default(name: Option<String>, fallback: String) -> String {
    name.unwrap_or_else(move || {
        let mut guest = fallback; // `fallback` is moved out: only FnOnce
        guest.push_str(" (guest)");
        guest
    })
}

fn main() {
    println!("1. Fn: only reads what it captured");
    let greeting = String::from("hello");
    let read = || greeting.clone();
    // This closure only holds a shared reference to `greeting`, so it's
    // `Copy`: passing it to call_twice copies it, and `read` stays usable.
    println!("    {}", call_twice(read));
    println!("    and it works with FnMut and FnOnce functions too: {}", run_once(read));

    println!("\n2. FnMut: changes what it captured");
    let mut total = 0;
    let mut seen = Vec::new();
    for_each_item(&[3, 4, 5], |n| {
        total += n;
        seen.push(n);
    });
    println!("    total = {total}, seen = {seen:?}");
    // Passing it to call_twice (which needs Fn) doesn't compile:
    //     error[E0594]: cannot assign to `count`, as it is a captured variable in a `Fn` closure
    //
    // And calling an FnMut closure needs the variable to be `mut`:
    //     let increment = || count += 1;  increment();
    //     error[E0596]: cannot borrow `increment` as mutable, as it is not declared as mutable
    let mut count = 0;
    let mut increment = || count += 1;
    increment();
    increment();
    println!("    count = {count}");

    println!("\n3. FnOnce: gives away what it captured");
    let name = String::from("Ferris");
    let give_away = move || name; // returns `name` itself: after one call it's gone
    println!("    {}", run_once(give_away));
    // Calling it twice:
    //     error[E0382]: use of moved value: `give_away`
    // Passing it where Fn is required:
    //     error[E0525]: expected a closure that implements the `Fn` trait,
    //                   but this closure only implements `FnOnce`

    println!("\n4. Standard library functions choose the least demanding trait");
    println!("    {}", name_or_default(None, String::from("guest")));
    println!("    {}", name_or_default(Some(String::from("Ana")), String::from("guest")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fn_closure_can_be_called_many_times() {
        let word = String::from("hi");
        assert_eq!(call_twice(|| word.clone()), "hi / hi");
    }

    #[test]
    fn fnmut_closure_accumulates() {
        let mut product = 1;
        for_each_item(&[2, 3, 4], |n| product *= n);
        assert_eq!(product, 24);
    }

    #[test]
    fn fnonce_accepts_every_kind_of_closure() {
        let owned = String::from("x");
        assert_eq!(run_once(move || owned), "x"); // FnOnce only
        assert_eq!(run_once(|| String::from("y")), "y"); // also Fn
    }

    #[test]
    fn unwrap_or_else_only_calls_when_needed() {
        assert_eq!(name_or_default(Some("A".into()), "B".into()), "A");
        assert_eq!(name_or_default(None, "B".into()), "B (guest)");
    }
}
