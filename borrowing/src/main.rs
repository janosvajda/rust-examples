// Define a struct to represent a book
#[derive(Debug)]
struct Book {
    title: String,
    author: String,
    pages: u32,
}

// Implementation block for the Book struct
impl Book {
    // Constructor for creating a new Book instance
    fn new(title: &str, author: &str, pages: u32) -> Book {
        Book {
            title: String::from(title),
            author: String::from(author),
            pages,
        }
    }

    // Method to print information about the book
    fn print_info(&self) {
        println!("Title: {}\nAuthor: {}\nPages: {}", self.title, self.author, self.pages);
    }

    // Method to change the title of the book
    fn change_title(&mut self, new_title: &str) {
        self.title = String::from(new_title);
    }
}

fn main() {
    // Create a book instance
    let mut my_book = Book::new("1984", "George Orwell", 328);

    // Borrow the book immutably to print its info
    my_book.print_info();

    // Borrow the book mutably to change its title
    my_book.change_title("Animal Farm");

    // Borrow the book immutably again to print updated info
    my_book.print_info();
}

// Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_creation_and_info() {
        let book = Book::new("Test Book", "Test Author", 100);
        assert_eq!(book.title, "Test Book");
        assert_eq!(book.author, "Test Author");
        assert_eq!(book.pages, 100);
    }

    #[test]
    fn test_change_title() {
        let mut book = Book::new("Original Title", "Author", 200);
        book.change_title("New Title");
        assert_eq!(book.title, "New Title");
    }
}
