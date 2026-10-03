// Lesson 2: repetition.
//
// `$( … ),*` in a pattern matches something repeated, separated by commas.
// The same `$( … )*` in the template repeats the output once per match.
// That's how `vec![1, 2, 3]` and `println!` accept any number of arguments.

use std::collections::HashMap;

// ---- 1. Our own vec! -----------------------------------------------------------------

/// `$( $item:expr ),*` = zero or more expressions separated by commas.
/// In the template, `$( v.push($item); )*` repeats the push for each one.
macro_rules! my_vec {
    ( $( $item:expr ),* ) => {{
        // For `my_vec![]` there are no pushes, so `mut` would be unused.
        #[allow(unused_mut)]
        let mut v = Vec::new();
        $( v.push($item); )*
        v
    }};
}

// ---- 2. A hashmap! literal, with an optional trailing comma ----------------------------

/// Each repetition here is a PAIR: `key => value`.
/// `$(,)?` at the end accepts an optional trailing comma, so you can write
/// one entry per line and end every line with a comma.
macro_rules! hashmap {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map = HashMap::new();
        $( map.insert($key, $value); )*
        map
    }};
}

// ---- 3. Repetition operators: *, + and ? ------------------------------------------------

/// `+` means ONE or more (an empty call doesn't match, so it's a compile error).
/// The template repeats ` + $rest` for each extra number.
macro_rules! sum {
    ( $first:expr $( , $rest:expr )* ) => {
        $first $( + $rest )*
    };
}

/// `*` zero or more, `+` one or more, `?` zero or one. Here `?` makes the
/// `with` part optional.
macro_rules! greeting {
    ( $name:expr $( , with $punctuation:literal )? ) => {{
        #[allow(unused_mut)]
        let mut text = format!("Hello, {}", $name);
        $( text.push_str($punctuation); )?
        text
    }};
}

// ---- 4. Recursion: a macro that calls itself ------------------------------------------------

/// The maximum of any number of values. The first rule handles one value;
/// the second compares the first value with the max of the REST, by
/// calling itself with one fewer argument each time.
macro_rules! max {
    ( $x:expr ) => { $x };
    ( $x:expr, $( $rest:expr ),+ ) => {{
        let rest = max!( $( $rest ),+ );
        if $x > rest { $x } else { rest }
    }};
}

/// Counting arguments with recursion: 0 for nothing, 1 + the count of the rest.
macro_rules! count {
    () => { 0usize };
    ( $head:tt $( $tail:tt )* ) => { 1usize + count!( $( $tail )* ) };
}

fn main() {
    println!("1. my_vec!");
    let numbers = my_vec![1, 2, 3, 4];
    let empty: Vec<i32> = my_vec![];
    println!("    {numbers:?} and {empty:?}");

    println!("\n2. hashmap!");
    let capitals = hashmap! {
        "Hungary" => "Budapest",
        "Austria" => "Vienna",
        "Czechia" => "Prague", // trailing comma accepted
    };
    let mut sorted: Vec<_> = capitals.iter().collect();
    sorted.sort();
    println!("    {sorted:?}");

    println!("\n3. Repetition operators");
    println!("    sum!(1, 2, 3, 4) = {}", sum!(1, 2, 3, 4));
    println!("    sum!(10) = {}", sum!(10));
    // sum!() doesn't compile: the pattern needs at least one number.
    println!("    {}", greeting!("Ferris"));
    println!("    {}", greeting!("Ferris", with "!!!"));

    println!("\n4. Recursion");
    println!("    max!(3, 9, 4, 1) = {}", max!(3, 9, 4, 1));
    println!("    max!(2.5) = {}", max!(2.5));
    println!("    count!(a b c d e) = {}", count!(a b c d e));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn my_vec_matches_vec() {
        assert_eq!(my_vec![1, 2, 3], vec![1, 2, 3]);
        assert_eq!(my_vec!["a"], vec!["a"]);
    }

    #[test]
    fn hashmap_literal() {
        let map = hashmap! { 1 => "one", 2 => "two" };
        assert_eq!(map.len(), 2);
        assert_eq!(map[&2], "two");
    }

    #[test]
    fn sum_and_max() {
        assert_eq!(sum!(5, 5, 5), 15);
        assert_eq!(max!(1, 7, 3), 7);
        assert_eq!(max!(-4), -4);
    }

    #[test]
    fn optional_part() {
        assert_eq!(greeting!("a"), "Hello, a");
        assert_eq!(greeting!("a", with "?"), "Hello, a?");
    }

    #[test]
    fn counting_is_done_at_compile_time() {
        const N: usize = count!(x y z);
        assert_eq!(N, 3);
        assert_eq!(count!(), 0);
    }
}
