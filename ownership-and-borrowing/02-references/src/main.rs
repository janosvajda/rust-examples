// Lesson 2: references.
//
// A reference lets you use a value without taking ownership of it.
//   &T      a shared reference: you can read through it.
//   &mut T  a mutable reference: you can read and change through it.
// Creating a reference is called *borrowing*. The owner keeps ownership and
// gets full use of the value back once the borrow ends.

#[derive(Debug)]
struct Book {
    title: String,
    author: String,
    pages: u32,
}

impl Book {
    fn new(title: &str, author: &str, pages: u32) -> Book {
        Book {
            title: title.to_string(),
            author: author.to_string(),
            pages,
        }
    }

    /// `&self`: borrows the book to read it. The most common kind of method.
    fn summary(&self) -> String {
        format!("\"{}\" by {}, {} pages", self.title, self.author, self.pages)
    }

    /// `&mut self`: borrows the book to change it.
    fn rename(&mut self, new_title: &str) {
        self.title = new_title.to_string();
    }

    /// `self`: takes ownership of the whole book. After calling this, the
    /// caller no longer has the book; they get its title instead.
    fn into_title(self) -> String {
        self.title
    }
}

/// Borrows a String to read it. The caller keeps ownership.
// Clippy suggests `&str` here, which is better. Lesson 4 explains why; this
// lesson keeps `&String` so the type matches the value being borrowed.
#[allow(clippy::ptr_arg)]
fn count_vowels(text: &String) -> usize {
    text.chars().filter(|c| "aeiouAEIOU".contains(*c)).count()
}

/// Borrows a number mutably to change it. `*` follows the reference to the
/// value it points to (called dereferencing).
fn double(number: &mut i32) {
    *number *= 2;
}

fn main() {
    println!("1. Borrow to read: &");
    let name = String::from("Ferris the crab");
    let vowels = count_vowels(&name); // lend `name`; we still own it
    println!("    \"{name}\" has {vowels} vowels, and `name` is still ours");

    println!("\n2. Borrow to change: &mut");
    let mut score = 21;
    double(&mut score); // the variable must be `mut` to lend it mutably
    println!("    score = {score}");
    // let fixed = 1;
    // double(&mut fixed);
    // error[E0596]: cannot borrow `fixed` as mutable, as it is not declared as mutable

    println!("\n3. A shared reference can't be used to change anything");
    // fn rename(name: &String) { name.push_str("!"); }
    // error[E0596]: cannot borrow `*name` as mutable, as it is behind a `&` reference
    println!("    (see the comment in the code: `&` means read-only)");

    println!("\n4. Methods borrow too: &self, &mut self, self");
    let mut book = Book::new("1984", "George Orwell", 328);
    println!("    {}", book.summary()); // borrows `book` to read
    book.rename("Animal Farm"); // borrows `book` to change
    println!("    {}", book.summary());
    let title = book.into_title(); // moves `book` into the method
    println!("    took the title out: {title}");
    // println!("{}", book.summary());
    // error[E0382]: borrow of moved value: `book`

    println!("\n5. Many shared references at the same time are fine");
    let text = String::from("shared");
    let r1 = &text;
    let r2 = &text;
    let r3 = r1; // copying a shared reference just makes another one
    println!("    {r1}, {r2}, {r3}: all point at the same String");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrowing_to_read_leaves_the_owner_intact() {
        let text = String::from("banana");
        assert_eq!(count_vowels(&text), 3);
        assert_eq!(text, "banana"); // still ours
    }

    #[test]
    fn borrowing_mutably_changes_the_original() {
        let mut n = 5;
        double(&mut n);
        double(&mut n);
        assert_eq!(n, 20);
    }

    #[test]
    fn methods_with_each_kind_of_self() {
        let mut book = Book::new("A", "B", 10);
        assert_eq!(book.summary(), "\"A\" by B, 10 pages");
        book.rename("C");
        assert_eq!(book.into_title(), "C");
    }

    #[test]
    fn a_reference_points_at_the_original_not_a_copy() {
        let value = 42;
        let reference = &value;
        assert!(std::ptr::eq(reference, &value));
    }
}
