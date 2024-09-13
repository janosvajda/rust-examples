# Generic Storage System in Rust

This project demonstrates how to use **Rust Generics** to build a flexible storage system that can store various types of real-world items such as books, food, and clothes.

## Project Overview

The `Storage<T>` struct is a generic container capable of storing any type of object (`Book`, `Food`, `Clothes`). By using generics, the storage system is reusable and type-safe for different data types, making it flexible and efficient.

### Features

- **Generic Storage**: Store any type of item using the `Storage<T>` container.
- **Replaceable Items**: Easily replace stored items with new ones.
- **Real-World Objects**: Includes specific types for books, food, and clothes.

### Code Overview

The application defines the following main components:
- `Storage<T>`: Generic storage container that can hold any type `T`.
- `Book`: Represents a book with a title and author.
- `Food`: Represents a food item with name and calories.
- `Clothes`: Represents clothing with size and material.

### Example Usage

The following example shows how you can store and replace items in the `Storage<T>`:

```rust
let mut book_storage = Storage::new(Book {
    title: String::from("The Rust Programming Language"),
    author: String::from("Steve Klabnik"),
});
println!(
    "Stored book: {} by {}",
    book_storage.get_item().title,
    book_storage.get_item().author
);

// Replace the stored book
book_storage.replace_item(Book {
    title: String::from("Programming Rust"),
    author: String::from("Jim Blandy"),
});
println!(
    "Updated book: {} by {}",
    book_storage.get_item().title,
    book_storage.get_item().author
);
