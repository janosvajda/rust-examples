// Define the trait for products
trait Product {
    fn harvest(&self);
    fn display(&self);
}

// Define the trait for fruits
trait Fruit: Product {}

// Define the trait for vegetables
trait Vegetable: Product {}

// Define concrete implementations for fruits
struct Strawberry;

impl Product for Strawberry {
    fn harvest(&self) {
        println!("Harvesting strawberries...");
    }

    fn display(&self) {
        println!("Strawberry");
    }
}

impl Fruit for Strawberry {}

struct Apple;

impl Product for Apple {
    fn harvest(&self) {
        println!("Harvesting apples...");
    }

    fn display(&self) {
        println!("Apple");
    }
}

impl Fruit for Apple {}

struct Raspberry;

impl Product for Raspberry {
    fn harvest(&self) {
        println!("Harvesting raspberries...");
    }

    fn display(&self) {
        println!("Raspberry");
    }
}

impl Fruit for Raspberry {}

// Define concrete implementations for vegetables
struct Onion;

impl Product for Onion {
    fn harvest(&self) {
        println!("Harvesting onions...");
    }

    fn display(&self) {
        println!("Onion");
    }
}

impl Vegetable for Onion {}

struct Carrot;

impl Product for Carrot {
    fn harvest(&self) {
        println!("Harvesting carrots...");
    }

    fn display(&self) {
        println!("Carrot");
    }
}

impl Vegetable for Carrot {}

// Define the error types for fruit and vegetable creation
#[derive(Debug)]
enum CreationError {
    UnknownFruit(String),
    UnknownVegetable(String),
}

impl std::fmt::Display for CreationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CreationError::UnknownFruit(name) => write!(f, "Unknown fruit: {}", name),
            CreationError::UnknownVegetable(name) => write!(f, "Unknown vegetable: {}", name),
        }
    }
}

impl std::error::Error for CreationError {}

// Define the abstract factory trait for fruits
trait FruitFactory {
    fn create_fruit(&self, fruit_name: &str) -> Result<Box<dyn Fruit>, CreationError>;
}

// Define the abstract factory trait for vegetables
trait VegetableFactory {
    fn create_vegetable(&self, vegetable_name: &str) -> Result<Box<dyn Vegetable>, CreationError>;
}

// Implement the concrete factory for a fruit farm with error handling
struct FruitFarm;

impl FruitFactory for FruitFarm {
    fn create_fruit(&self, fruit_name: &str) -> Result<Box<dyn Fruit>, CreationError> {
        match fruit_name {
            "strawberry" => Ok(Box::new(Strawberry)),
            "apple" => Ok(Box::new(Apple)),
            "raspberry" => Ok(Box::new(Raspberry)),
            _ => Err(CreationError::UnknownFruit(fruit_name.to_string())),
        }
    }
}

// Implement the concrete factory for a vegetable farm with error handling
struct VegetableFarm;

impl VegetableFactory for VegetableFarm {
    fn create_vegetable(&self, vegetable_name: &str) -> Result<Box<dyn Vegetable>, CreationError> {
        match vegetable_name {
            "onion" => Ok(Box::new(Onion)),
            "carrot" => Ok(Box::new(Carrot)),
            _ => Err(CreationError::UnknownVegetable(vegetable_name.to_string())),
        }
    }
}

// Usage example
fn main() {
    let fruit_factory: Box<dyn FruitFactory> = Box::new(FruitFarm);
    let vegetable_factory: Box<dyn VegetableFactory> = Box::new(VegetableFarm);

    let fruit_names = vec!["strawberry", "apple", "raspberry", "banana"];
    for name in fruit_names {
        match fruit_factory.create_fruit(name) {
            Ok(fruit) => {
                fruit.harvest();
                fruit.display();
            }
            Err(error) => {
                eprintln!("Error creating fruit: {}", error);
                // Handle the error as needed
            }
        }
    }

    let vegetable_names = vec!["onion", "carrot", "potato"];
    for name in vegetable_names {
        match vegetable_factory.create_vegetable(name) {
            Ok(vegetable) => {
                vegetable.harvest();
                vegetable.display();
            }
            Err(error) => {
                eprintln!("Error creating vegetable: {}", error);
                // Handle the error as needed
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_fruits_and_vegetables() {
        let fruit_factory: Box<dyn FruitFactory> = Box::new(FruitFarm);
        let vegetable_factory: Box<dyn VegetableFactory> = Box::new(VegetableFarm);

        let fruit_names = vec!["strawberry", "apple", "raspberry", "banana"];
        for name in fruit_names {
            match fruit_factory.create_fruit(name) {
                Ok(fruit) => {
                    fruit.harvest();
                    fruit.display();
                }
                Err(error) => {
                    eprintln!("Error creating fruit: {}", error);
                    // Handle the error as needed
                }
            }
        }

        let vegetable_names = vec!["onion", "carrot", "potato"];
        for name in vegetable_names {
            match vegetable_factory.create_vegetable(name) {
                Ok(vegetable) => {
                    vegetable.harvest();
                    vegetable.display();
                }
                Err(error) => {
                    eprintln!("Error creating vegetable: {}", error);
                    // Handle the error as needed
                }
            }
        }
    }
}
