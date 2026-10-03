// Decorator pattern: a coffee shop where you start with a base drink and add
// extras on top. Each extra wraps the drink before it, adding its own price
// and description, so any combination is possible without a separate type
// for every combination.

/// Anything that can be sold as a drink: a base drink, or a drink with
/// extras added. Base drinks and extras share this one interface.
trait Beverage {
    fn description(&self) -> String;
    /// Price in cents, so we never get rounding errors with money.
    fn cost_cents(&self) -> u32;
}

// ---- Base drinks ----------------------------------------------------------

struct Espresso;

impl Beverage for Espresso {
    fn description(&self) -> String {
        "Espresso".to_string()
    }
    fn cost_cents(&self) -> u32 {
        250
    }
}

struct Tea;

impl Beverage for Tea {
    fn description(&self) -> String {
        "Tea".to_string()
    }
    fn cost_cents(&self) -> u32 {
        200
    }
}

// ---- Decorators: extras that wrap another beverage -----------------------
//
// Each one holds the drink it decorates (`inner`) and implements `Beverage`
// itself. It answers by asking the inner drink first, then adding its part.

struct Milk {
    inner: Box<dyn Beverage>,
}

impl Beverage for Milk {
    fn description(&self) -> String {
        format!("{} + milk", self.inner.description())
    }
    fn cost_cents(&self) -> u32 {
        self.inner.cost_cents() + 50
    }
}

struct Sugar {
    inner: Box<dyn Beverage>,
}

impl Beverage for Sugar {
    fn description(&self) -> String {
        format!("{} + sugar", self.inner.description())
    }
    fn cost_cents(&self) -> u32 {
        self.inner.cost_cents() + 10
    }
}

struct WhippedCream {
    inner: Box<dyn Beverage>,
}

impl Beverage for WhippedCream {
    fn description(&self) -> String {
        format!("{} + whipped cream", self.inner.description())
    }
    fn cost_cents(&self) -> u32 {
        self.inner.cost_cents() + 80
    }
}

/// A decorator can also change behaviour instead of just adding to it:
/// this one makes the whole drink bigger, so it multiplies the price.
struct LargeSize {
    inner: Box<dyn Beverage>,
}

impl Beverage for LargeSize {
    fn description(&self) -> String {
        format!("Large {}", self.inner.description())
    }
    fn cost_cents(&self) -> u32 {
        // 50% more, rounded up to a whole cent.
        (self.inner.cost_cents() * 3).div_ceil(2)
    }
}

// ---- Small helpers so orders read naturally -------------------------------

fn with_milk(drink: Box<dyn Beverage>) -> Box<dyn Beverage> {
    Box::new(Milk { inner: drink })
}

fn with_sugar(drink: Box<dyn Beverage>) -> Box<dyn Beverage> {
    Box::new(Sugar { inner: drink })
}

fn with_cream(drink: Box<dyn Beverage>) -> Box<dyn Beverage> {
    Box::new(WhippedCream { inner: drink })
}

fn large(drink: Box<dyn Beverage>) -> Box<dyn Beverage> {
    Box::new(LargeSize { inner: drink })
}

fn price(cents: u32) -> String {
    format!("{}.{:02} €", cents / 100, cents % 100)
}

fn main() {
    let orders: Vec<Box<dyn Beverage>> = vec![
        // Just the base drink.
        Box::new(Espresso),
        // One extra.
        with_milk(Box::new(Espresso)),
        // Extras stack, and the same extra can be added twice.
        with_sugar(with_sugar(with_milk(Box::new(Tea)))),
        // Order matters: "large" wraps everything inside it.
        large(with_cream(with_milk(Box::new(Espresso)))),
    ];

    for drink in &orders {
        println!("{:<45} {:>8}", drink.description(), price(drink.cost_cents()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base_drinks_on_their_own() {
        assert_eq!(Espresso.cost_cents(), 250);
        assert_eq!(Tea.description(), "Tea");
    }

    #[test]
    fn each_extra_adds_its_price_and_name() {
        let drink = with_cream(with_milk(Box::new(Espresso)));
        assert_eq!(drink.cost_cents(), 250 + 50 + 80);
        assert_eq!(drink.description(), "Espresso + milk + whipped cream");
    }

    #[test]
    fn the_same_extra_can_be_added_twice() {
        let drink = with_sugar(with_sugar(Box::new(Tea)));
        assert_eq!(drink.cost_cents(), 220);
        assert_eq!(drink.description(), "Tea + sugar + sugar");
    }

    #[test]
    fn wrapping_order_matters() {
        // Large applies to whatever is inside it.
        let large_then_milk = with_milk(large(Box::new(Espresso)));
        let milk_then_large = large(with_milk(Box::new(Espresso)));
        assert_eq!(large_then_milk.cost_cents(), 375 + 50);
        assert_eq!(milk_then_large.cost_cents(), 450);
    }

    #[test]
    fn price_formatting() {
        assert_eq!(price(250), "2.50 €");
        assert_eq!(price(1005), "10.05 €");
    }
}
