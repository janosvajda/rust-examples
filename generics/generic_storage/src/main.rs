struct Storage<T> {
    item: T,
}

impl<T> Storage<T> {
    // Create a new storage container
    fn new(item: T) -> Self {
        Storage { item }
    }

    // Get the item stored in the container
    fn get_item(&self) -> &T {
        &self.item
    }

    // Replace the item with a new one
    fn replace_item(&mut self, new_item: T) {
        self.item = new_item;
    }
}

// Struct representing a Book
#[derive(Debug, PartialEq)]
struct Book {
    title: String,
    author: String,
}

// Struct representing Food
#[derive(Debug, PartialEq)]
struct Food {
    name: String,
    calories: u32,
}

// Struct representing Clothes
#[derive(Debug, PartialEq)]
struct Clothes {
    size: String,
    material: String,
}

fn main() {
    // Example: Storing a Book
    let mut book_storage = Storage::new(Book {
        title: String::from("The Rust Programming Language"),
        author: String::from("Steve Klabnik"),
    });
    println!(
        "Stored book: {} by {}",
        book_storage.get_item().title,
        book_storage.get_item().author
    );

    // Replace the book with another book using replace_item
    book_storage.replace_item(Book {
        title: String::from("Programming Rust"),
        author: String::from("Jim Blandy"),
    });
    println!(
        "Updated book: {} by {}",
        book_storage.get_item().title,
        book_storage.get_item().author
    );

    // Example: Storing Food
    let mut food_storage = Storage::new(Food {
        name: String::from("Apple"),
        calories: 95,
    });
    println!(
        "Stored food: {}, Calories: {}",
        food_storage.get_item().name,
        food_storage.get_item().calories
    );

    // Replace the food with another food using replace_item
    food_storage.replace_item(Food {
        name: String::from("Banana"),
        calories: 105,
    });
    println!(
        "Updated food: {}, Calories: {}",
        food_storage.get_item().name,
        food_storage.get_item().calories
    );

    // Example: Storing Clothes
    let mut clothes_storage = Storage::new(Clothes {
        size: String::from("L"),
        material: String::from("Cotton"),
    });
    println!(
        "Stored clothes: Size {}, Material {}",
        clothes_storage.get_item().size,
        clothes_storage.get_item().material
    );

    // Replace the clothes with another clothes using replace_item
    clothes_storage.replace_item(Clothes {
        size: String::from("M"),
        material: String::from("Silk"),
    });
    println!(
        "Updated clothes: Size {}, Material {}",
        clothes_storage.get_item().size,
        clothes_storage.get_item().material
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_storage() {
        let book = Book {
            title: String::from("Rust in Action"),
            author: String::from("Tim McNamara"),
        };
        let mut book_storage = Storage::new(book);

        assert_eq!(
            *book_storage.get_item(),
            Book {
                title: String::from("Rust in Action"),
                author: String::from("Tim McNamara"),
            }
        );

        // Replace the stored book
        book_storage.replace_item(Book {
            title: String::from("Programming Rust"),
            author: String::from("Jim Blandy"),
        });

        assert_eq!(
            *book_storage.get_item(),
            Book {
                title: String::from("Programming Rust"),
                author: String::from("Jim Blandy"),
            }
        );
    }

    #[test]
    fn test_food_storage() {
        let food = Food {
            name: String::from("Banana"),
            calories: 105,
        };
        let mut food_storage = Storage::new(food);

        assert_eq!(
            *food_storage.get_item(),
            Food {
                name: String::from("Banana"),
                calories: 105,
            }
        );

        // Replace the stored food
        food_storage.replace_item(Food {
            name: String::from("Orange"),
            calories: 62,
        });

        assert_eq!(
            *food_storage.get_item(),
            Food {
                name: String::from("Orange"),
                calories: 62,
            }
        );
    }

    #[test]
    fn test_clothes_storage() {
        let clothes = Clothes {
            size: String::from("M"),
            material: String::from("Wool"),
        };
        let mut clothes_storage = Storage::new(clothes);

        assert_eq!(
            *clothes_storage.get_item(),
            Clothes {
                size: String::from("M"),
                material: String::from("Wool"),
            }
        );

        // Replace the stored clothes
        clothes_storage.replace_item(Clothes {
            size: String::from("L"),
            material: String::from("Silk"),
        });

        assert_eq!(
            *clothes_storage.get_item(),
            Clothes {
                size: String::from("L"),
                material: String::from("Silk"),
            }
        );
    }
}
