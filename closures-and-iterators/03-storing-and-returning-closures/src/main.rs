// Lesson 3: storing and returning closures.
//
// A closure's type has no name you can write, so to return one or keep one
// in a struct you use either:
//   impl Fn(…) -> …        one specific closure type, known to the compiler (fast)
//   Box<dyn Fn(…) -> …>    any closure with that signature, decided at run time (flexible)

use std::collections::HashMap;

// ---- 1. Returning a closure: impl Fn --------------------------------------------------

/// Returns a closure that adds `n`. `move` is required: the closure keeps
/// `n` after this function returns. Without it:
///     error[E0373]: closure may outlive the current function, but it borrows `n`,
///                   which is owned by the current function
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

/// A closure factory with more state: a counter that remembers its count.
fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

// ---- 2. Returning one of SEVERAL closures: Box<dyn Fn> ---------------------------------

/// Two branches, two different closure types: `impl Fn` can't express that.
///     if … { move |x| x * n } else { move |x| x + n }
///     error[E0308]: `if` and `else` have incompatible types
///     = note: no two closures, even if identical, have the same type
/// A Box<dyn Fn> can hold either.
fn make_operation(name: &str, n: i32) -> Option<Box<dyn Fn(i32) -> i32>> {
    match name {
        "add" => Some(Box::new(move |x| x + n)),
        "multiply" => Some(Box::new(move |x| x * n)),
        "power" if n >= 0 => Some(Box::new(move |x| x.pow(n as u32))),
        _ => None,
    }
}

// ---- 3. A struct holding a closure: generic field ------------------------------------

/// Remembers the result of an expensive calculation per input.
/// `F` is generic: each Memo is specialised for one closure (no Box, no
/// run-time lookup).
struct Memo<F: Fn(u64) -> u64> {
    calculation: F,
    cache: HashMap<u64, u64>,
    calls: u32,
}

impl<F: Fn(u64) -> u64> Memo<F> {
    fn new(calculation: F) -> Self {
        Memo {
            calculation,
            cache: HashMap::new(),
            calls: 0,
        }
    }

    fn get(&mut self, input: u64) -> u64 {
        if let Some(&cached) = self.cache.get(&input) {
            return cached;
        }
        self.calls += 1;
        // Calling a closure stored in a field needs the parentheses:
        // `(self.calculation)(input)`, not `self.calculation(input)`, which
        // would look for a METHOD called `calculation`.
        let result = (self.calculation)(input);
        self.cache.insert(input, result);
        result
    }
}

// ---- 4. A struct holding MANY different closures: Vec<Box<dyn Fn>> ------------------

/// One click handler: any closure taking the label and returning a message.
/// A `type` alias gives the long trait-object type a readable name.
type Handler = Box<dyn Fn(&str) -> String>;

/// A button with any number of click handlers, each a different closure.
struct Button {
    label: String,
    handlers: Vec<Handler>,
}

impl Button {
    fn on_click(&mut self, handler: impl Fn(&str) -> String + 'static) {
        self.handlers.push(Box::new(handler));
    }

    fn click(&self) -> Vec<String> {
        self.handlers
            .iter()
            .map(|handler| handler(&self.label))
            .collect()
    }
}

fn main() {
    println!("1. Returning closures with impl Fn");
    let add_five = make_adder(5);
    let add_ten = make_adder(10);
    println!(
        "    add_five(1) = {}, add_ten(1) = {}",
        add_five(1),
        add_ten(1)
    );
    let mut next_ticket = make_counter();
    println!(
        "    tickets: {}, {}, {}",
        next_ticket(),
        next_ticket(),
        next_ticket()
    );

    println!("\n2. Choosing a closure at run time with Box<dyn Fn>");
    for name in ["add", "multiply", "power", "divide"] {
        match make_operation(name, 3) {
            Some(op) => println!("    {name:<8} 3 on 4 = {}", op(4)),
            None => println!("    {name:<8} unknown operation"),
        }
    }

    println!("\n3. A struct with a generic closure field: memoisation");
    let mut slow_square = Memo::new(|n| {
        std::thread::sleep(std::time::Duration::from_millis(100)); // pretend it's expensive
        n * n
    });
    for n in [4, 4, 9, 4, 9] {
        print!("    {n}² = {} ", slow_square.get(n));
    }
    println!(
        "\n    only {} real calculations for 5 requests",
        slow_square.calls
    );

    println!("\n4. A struct with many different closures");
    let mut button = Button {
        label: String::from("Save"),
        handlers: Vec::new(),
    };
    let user = String::from("Ana");
    button.on_click(|label| format!("clicked {label}"));
    button.on_click(move |label| format!("{user} pressed {label}")); // captures `user`
    button.on_click(|label| format!("{} letters", label.len()));
    for message in button.click() {
        println!("    {message}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adders_remember_their_n() {
        let add_two = make_adder(2);
        assert_eq!(add_two(40), 42);
    }

    #[test]
    fn each_counter_has_its_own_state() {
        let mut a = make_counter();
        let mut b = make_counter();
        assert_eq!((a(), a(), b()), (1, 2, 1));
    }

    #[test]
    fn boxed_operations() {
        assert_eq!(make_operation("power", 2).unwrap()(5), 25);
        assert!(make_operation("nope", 1).is_none());
    }

    #[test]
    fn memo_calculates_each_input_once() {
        let mut memo = Memo::new(|n| n + 1);
        memo.get(1);
        memo.get(1);
        memo.get(2);
        assert_eq!(memo.calls, 2);
    }

    #[test]
    fn button_runs_every_handler_in_order() {
        let mut button = Button {
            label: String::from("Go"),
            handlers: Vec::new(),
        };
        button.on_click(|l| l.to_uppercase());
        button.on_click(|l| l.repeat(2));
        assert_eq!(button.click(), ["GO", "GoGo"]);
    }
}
