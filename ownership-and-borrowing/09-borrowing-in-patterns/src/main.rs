// Lesson 9: borrowing in patterns.
//
// `match`, `if let`, `let` destructuring and closure parameters can all
// MOVE, COPY, BORROW or IGNORE parts. The pattern and matched type together
// decide what happens; a wildcard need not move the matched value.

#[derive(Debug)]
struct Book {
    title: String,
    pages: u32,
}

#[derive(Debug)]
struct User {
    name: String,
    nickname: Option<String>,
}

/// Returns the nickname if there is one, without taking it away from `user`.
/// `as_deref` turns `&Option<String>` into `Option<&str>`.
fn display_name(user: &User) -> &str {
    user.nickname.as_deref().unwrap_or(&user.name)
}

fn main() {
    println!("1. Binding a non-Copy String by value moves it out");
    let nickname: Option<String> = Some(String::from("Ferris"));
    match nickname {
        Some(name) => println!("    moved the String out: {name}"),
        None => println!("    no nickname"),
    }
    // println!("{nickname:?}");
    // error[E0382]: borrow of partially moved value: `nickname`

    println!("\n2. Matching on a reference borrows instead");
    let nickname: Option<String> = Some(String::from("Ferris"));
    match &nickname {
        // Matching `&Option<String>` against `Some(name)` makes `name` a
        // `&String` automatically ("match ergonomics").
        Some(name) => println!("    borrowed: {name} ({} UTF-8 bytes)", name.len()),
        None => println!("    no nickname"),
    }
    println!("    and `nickname` is still usable: {nickname:?}");

    println!("\n3. Changing the inside through `&mut`");
    let mut maybe_list: Option<Vec<i32>> = Some(vec![1, 2]);
    if let Some(list) = &mut maybe_list {
        list.push(3); // `list` is `&mut Vec<i32>`
    }
    println!("    {maybe_list:?}");

    println!("\n4. The same with Option's helper methods");
    let user = User {
        name: String::from("Grace"),
        nickname: None,
    };
    let length = user.nickname.as_ref().map(|n| n.len()); // Option<&String>: no move
    println!(
        "    nickname length: {length:?}, display name: {}",
        display_name(&user)
    );
    let mut counter: Option<u32> = Some(1);
    if let Some(n) = counter.as_mut() {
        *n += 1;
    }
    println!("    counter: {counter:?}");

    println!("\n5. `ref` and `ref mut`: borrow while matching a value");
    let pair = (String::from("key"), 42);
    let (ref key, value) = pair; // borrow the String, copy the number
    println!("    key = {key}, value = {value}, pair still whole: {pair:?}");

    println!("\n6. Moving one field out of a struct: a partial move");
    let book = Book {
        title: String::from("Dune"),
        pages: 412,
    };
    let title = book.title; // moves only `title` out
    println!(
        "    took \"{title}\", the other fields still work: {} pages",
        book.pages
    );
    // println!("{book:?}");
    // error[E0382]: borrow of partially moved value: `book`
    // The struct as a whole is no longer complete.

    println!("\n7. You can't move non-Copy values out by index or through a reference");
    let names = vec![String::from("Ana"), String::from("Bob")];
    // let first = names[0];
    // error[E0507]: cannot move out of index of `Vec<String>`
    let first = &names[0]; // borrow it…
    let first_owned = names[0].clone(); // …or clone it…
    println!("    borrowed {first}, cloned {first_owned}");
    let mut names = names;
    let removed = names.remove(0); // …or take it out of the Vec for real
    println!("    removed {removed}, left {names:?}");
    let shared: &Option<String> = &Some(String::from("x"));
    // let inner = shared.unwrap();
    // error[E0507]: cannot move out of `*shared` which is behind a shared reference
    let inner = shared.as_ref().unwrap(); // borrow the inside instead
    println!("    inner = {inner}");

    println!("\n8. Patterns in closure parameters and for loops");
    let numbers = [3, 8, 2];
    // `iter()` gives `&i32`; the pattern `&n` copies the number out.
    let big: Vec<i32> = numbers
        .iter()
        .filter(|&&n| n > 2)
        .map(|&n| n * 10)
        .collect();
    for (index, &n) in numbers.iter().enumerate() {
        print!("    [{index}]={n}");
    }
    println!("\n    big ones × 10: {big:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_a_reference_leaves_the_value_intact() {
        let value = Some(String::from("abc"));
        let length = match &value {
            Some(s) => s.len(),
            None => 0,
        };
        assert_eq!(length, 3);
        assert_eq!(value.as_deref(), Some("abc"));
    }

    #[test]
    fn if_let_with_mut_changes_the_inside() {
        let mut v = Some(vec![1]);
        if let Some(list) = &mut v {
            list.push(2);
        }
        assert_eq!(v, Some(vec![1, 2]));
    }

    #[test]
    fn display_name_prefers_the_nickname() {
        let with = User {
            name: String::from("Grace"),
            nickname: Some(String::from("G")),
        };
        let without = User {
            name: String::from("Grace"),
            nickname: None,
        };
        assert_eq!(display_name(&with), "G");
        assert_eq!(display_name(&without), "Grace");
    }

    #[test]
    fn partial_move_leaves_other_fields_usable() {
        let book = Book {
            title: String::from("T"),
            pages: 7,
        };
        let title = book.title;
        assert_eq!(title, "T");
        assert_eq!(book.pages, 7);
    }

    #[test]
    fn ref_pattern_borrows_instead_of_moving() {
        let pair = (String::from("k"), 1);
        let (ref k, _) = pair;
        assert_eq!(k, "k");
        assert_eq!(pair.0, "k"); // pair is still whole
    }
}
