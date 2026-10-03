// Lesson 1: macro_rules! basics.
//
// A macro is code that writes code. A `macro_rules!` macro matches the
// tokens you pass it against patterns, and replaces the call with the
// matching template, before the program is type-checked. That lets macros
// do what functions can't: take any number of arguments, accept code
// (not just values), or create new items.
//
// Macros must be defined BEFORE they're used in a file. Using one above its
// definition gives:
//     error: cannot find macro `shout` in this scope

// ---- 1. The simplest macro: one rule ----------------------------------------------

/// `($x:expr)` is the pattern: one expression, called `$x`.
/// `{ $x * $x }` is the template that replaces the call.
macro_rules! square {
    ($x:expr) => {
        $x * $x
    };
}

// ---- 2. Several rules: like `match` for code -------------------------------------------

/// The rules are tried top to bottom; the first that matches wins.
macro_rules! greet {
    () => {
        String::from("Hello, world!")
    };
    ($name:expr) => {
        format!("Hello, {}!", $name)
    };
    ($greeting:literal to $name:expr) => {
        format!("{}, {}!", $greeting, $name)
    };
}

// ---- 3. Fragment types: what kind of code each piece is -----------------------------------

/// `ident` = a name, `ty` = a type, `expr` = an expression. This macro
/// DEFINES A NEW FUNCTION, something no ordinary function can do.
macro_rules! make_getter {
    ($name:ident, $type:ty, $value:expr) => {
        fn $name() -> $type {
            $value
        }
    };
}

make_getter!(answer, u32, 42);
make_getter!(language, &'static str, "Rust");

/// `block` = a `{ … }` block of code. This macro runs the block and
/// reports how long it took.
macro_rules! timed {
    ($label:literal, $body:block) => {{
        let start = std::time::Instant::now();
        let result = $body;
        println!("    [{} took {:?}]", $label, start.elapsed());
        result
    }};
}

// ---- 4. An `expr` stays one expression -----------------------------------------------------

/// Unlike text substitution (C's #define), `$x:expr` is inserted as ONE
/// grouped expression. So double!(1 + 1) is (1 + 1) * 2 = 4, not 1 + 1 * 2 = 3.
macro_rules! double {
    ($x:expr) => {
        $x * 2
    };
}

// ---- 5. Hygiene: a macro's variables don't leak ------------------------------------------

/// The `x` created inside the macro is invisible to the caller, even if the
/// caller also has an `x`. Macros can't accidentally clash with your names.
macro_rules! set_x_to_ten {
    () => {
        let x = 10;
        let _ = x;
    };
}

fn main() {
    println!("1. A simple macro");
    println!("    square!(7) = {}", square!(7));
    println!("    square!(2 + 1) = {}", square!(2 + 1));

    println!("\n2. Several rules");
    println!("    {}", greet!());
    println!("    {}", greet!("Ferris"));
    println!("    {}", greet!("Good morning" to "Ana"));
    // greet!("a", "b");   →   error: no rules expected `,`

    println!("\n3. Macros can create functions");
    println!("    answer() = {}, language() = {}", answer(), language());
    let total: u64 = timed!("summing a million numbers", { (1..=1_000_000u64).sum() });
    println!("    sum = {total}");

    println!("\n4. An expression stays grouped");
    println!("    double!(1 + 1) = {}", double!(1 + 1));

    println!("\n5. Hygiene");
    let x = 1;
    set_x_to_ten!();
    println!("    x is still {x}: the macro's own x didn't touch ours");
    // Using the macro's variable from outside doesn't work either:
    //     error[E0425]: cannot find value `x` in this scope
}

#[cfg(test)]
mod tests {
    #[test]
    fn square_works_on_expressions() {
        assert_eq!(square!(4), 16);
        assert_eq!(square!(1 + 2), 9);
    }

    #[test]
    fn rules_are_matched_in_order() {
        assert_eq!(greet!(), "Hello, world!");
        assert_eq!(greet!("you"), "Hello, you!");
        assert_eq!(greet!("Hi" to "Bob"), "Hi, Bob!");
    }

    #[test]
    fn generated_functions_exist() {
        assert_eq!(super::answer(), 42);
        assert_eq!(super::language(), "Rust");
    }

    #[test]
    fn expr_fragments_keep_precedence() {
        assert_eq!(double!(1 + 1), 4);
    }

    #[test]
    fn timed_returns_the_block_value() {
        assert_eq!(timed!("test", { 2 + 2 }), 4);
    }
}
