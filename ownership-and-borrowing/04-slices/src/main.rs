// Lesson 4: slices.
//
// A slice is a reference to a *part* of a collection: a run of items that
// sit next to each other in memory.
//   &str  a slice of text (part of a String, or a string literal)
//   &[T]  a slice of items (part of a Vec, an array, ...)
// A slice is a borrow like any other, so all the borrowing rules apply to it.

/// Returns the first word of `text` as a slice of it: no copying.
///
/// Taking `&str` instead of `&String` means this works for `String`s,
/// string literals, and slices of either.
fn first_word(text: &str) -> &str {
    text.split(' ').next().unwrap_or("")
}

/// Works with a Vec, an array, or part of either, because it takes `&[i32]`.
fn average(numbers: &[i32]) -> Option<f64> {
    if numbers.is_empty() {
        return None;
    }
    let total: i32 = numbers.iter().sum();
    Some(total as f64 / numbers.len() as f64)
}

/// A mutable slice lets a function change part of a collection in place.
fn double_all(numbers: &mut [i32]) {
    for n in numbers {
        *n *= 2;
    }
}

fn main() {
    println!("1. A string slice is a view into part of a String");
    let sentence = String::from("hello wonderful world");
    let hello = &sentence[0..5]; // bytes 0 up to (not including) 5
    let world = &sentence[16..]; // byte 16 to the end
    println!("    \"{hello}\" and \"{world}\", no new Strings were made");

    println!("\n2. &str accepts Strings, literals and slices alike");
    let owned = String::from("rust is fun");
    println!("    from a String:  {}", first_word(&owned)); // &String becomes &str automatically
    println!("    from a literal: {}", first_word("borrow checker"));
    println!("    from a slice:   {}", first_word(&owned[5..]));

    println!("\n3. A slice keeps its source borrowed");
    let mut text = String::from("hello world");
    let word = first_word(&text);
    // text.clear();
    // println!("{word}");
    // error[E0502]: cannot borrow `text` as mutable because it is also borrowed as immutable
    println!("    first word = {word}");
    text.clear(); // fine once `word` is no longer used
    println!("    text is now empty: {:?}", text);

    println!("\n4. Slices of Vecs and arrays: &[T]");
    let scores = vec![70, 85, 90, 100, 65];
    let array = [1, 2, 3];
    println!("    whole Vec:    {:?}", average(&scores));
    println!("    first three:  {:?}", average(&scores[..3]));
    println!("    an array:     {:?}", average(&array));
    println!("    empty slice:  {:?}", average(&[]));

    println!("\n5. A mutable slice changes the original");
    let mut values = vec![1, 2, 3, 4, 5];
    double_all(&mut values[1..4]); // only the middle three
    println!("    values = {values:?}");

    println!("\n6. String slices are measured in bytes, not characters");
    let city = String::from("Győr");
    println!("    \"{city}\" has {} characters but {} bytes", city.chars().count(), city.len());
    // &city[0..3] would panic at runtime: byte 3 is in the middle of "ő",
    // which takes 2 bytes in UTF-8.
    println!("    safe: {:?}", city.get(0..3)); // returns None instead of panicking
    println!("    safe: {:?}", city.get(0..4));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_word_of_different_kinds_of_string() {
        assert_eq!(first_word("one two"), "one");
        assert_eq!(first_word(&String::from("single")), "single");
        assert_eq!(first_word(""), "");
    }

    #[test]
    fn a_slice_points_into_the_original_memory() {
        let text = String::from("hello world");
        let word = first_word(&text);
        // Same address as the start of `text`: nothing was copied.
        assert_eq!(word.as_ptr(), text.as_ptr());
    }

    #[test]
    fn average_of_any_slice() {
        assert_eq!(average(&[2, 4, 6]), Some(4.0));
        assert_eq!(average(&[1, 2, 3, 4][2..]), Some(3.5));
        assert_eq!(average(&[]), None);
    }

    #[test]
    fn mutable_slice_changes_only_its_part() {
        let mut v = vec![1, 1, 1, 1];
        double_all(&mut v[..2]);
        assert_eq!(v, vec![2, 2, 1, 1]);
    }

    #[test]
    fn slicing_in_the_middle_of_a_character_is_refused() {
        let city = "Győr";
        assert_eq!(city.get(0..3), None);
        assert_eq!(city.get(0..4), Some("Győ"));
    }
}
