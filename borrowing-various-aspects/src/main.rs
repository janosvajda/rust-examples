// Demonstrating various aspects of borrowing in Rust

fn main() {
    // Mutable and Immutable References in the Same Scope
    let data = vec![1, 2, 3];
    let _immutable_ref = &data;
    // Uncommenting the following line will result in a compilation error
    // data.push(4);

    // Dangling References
    let _dangling_ref = create_dangling_reference();

    // Borrowing Across Function Calls
    let mut value = 42;
    borrow_across_functions(&mut value);
    println!("Modified value: {}", value);

    // Borrowing Struct Fields
    let mut my_book = Book::new("Title", "Author", 100);
    let title_ref = &mut my_book.title;
    title_ref.push_str(" - New Edition");
    let _author_ref = &my_book.author;
    let _pages_ref = &my_book.pages;

    // Slices
    let data = vec![1, 2, 3, 4, 5];
    let slice = &data[1..4];
    println!("Slice: {:?}", slice);

    // Borrowing and Iterators
    let data = vec![10, 20, 30];
    for item in &data {
        println!("Iterator item: {}", item);
    }
}

// Dangling References
fn create_dangling_reference() -> i32 {
    let value = 42;
    value
}

// Borrowing Across Function Calls
fn borrow_across_functions(value: &mut i32) {
    *value += 10;
}

// Book struct for borrowing struct fields example
struct Book {
    title: String,
    author: String,
    pages: u32,
}

impl Book {
    // Constructor for Book
    fn new(title: &str, author: &str, pages: u32) -> Book {
        Book {
            title: String::from(title),
            author: String::from(author),
            pages,
        }
    }
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_borrow_across_functions() {
        let mut value = 42;
        borrow_across_functions(&mut value);
        assert_eq!(value, 52);
    }

    #[test]
    fn test_borrow_struct_fields() {
        let mut my_book = Book::new("Title", "Author", 100);
        let title_ref = &mut my_book.title;
        title_ref.push_str(" - New Edition");
        assert_eq!(title_ref, "Title - New Edition");
    }

    #[test]
    fn test_slices() {
        let data = vec![1, 2, 3, 4, 5];
        let slice = &data[1..4];
        assert_eq!(slice, &[2, 3, 4]);
    }
}
