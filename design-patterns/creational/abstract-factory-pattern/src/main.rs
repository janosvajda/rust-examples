// Abstract Factory pattern: farms that grow matching sets of produce.
// A tropical farm grows a tropical fruit and a tropical vegetable; a
// temperate farm grows a temperate pair. The shop only talks to "a farm",
// so it always gets a set that belongs together, without naming any of the
// concrete types.

// ---- Product interfaces ---------------------------------------------------------

trait Fruit {
    fn name(&self) -> &'static str;
    fn ripe_in_days(&self) -> u32;
}

trait Vegetable {
    fn name(&self) -> &'static str;
    fn needs_cooking(&self) -> bool;
}

// ---- Tropical family ------------------------------------------------------------

struct Mango;

impl Fruit for Mango {
    fn name(&self) -> &'static str {
        "mango"
    }
    fn ripe_in_days(&self) -> u32 {
        5
    }
}

struct Okra;

impl Vegetable for Okra {
    fn name(&self) -> &'static str {
        "okra"
    }
    fn needs_cooking(&self) -> bool {
        true
    }
}

// ---- Temperate family -----------------------------------------------------------

struct Apple;

impl Fruit for Apple {
    fn name(&self) -> &'static str {
        "apple"
    }
    fn ripe_in_days(&self) -> u32 {
        0
    }
}

struct Carrot;

impl Vegetable for Carrot {
    fn name(&self) -> &'static str {
        "carrot"
    }
    fn needs_cooking(&self) -> bool {
        false
    }
}

// ---- The abstract factory -------------------------------------------------------

/// One interface that creates a whole FAMILY of related products. Each farm
/// implements both methods, so the fruit and vegetable you get always match.
trait Farm {
    fn climate(&self) -> &'static str;
    fn create_fruit(&self) -> Box<dyn Fruit>;
    fn create_vegetable(&self) -> Box<dyn Vegetable>;
}

struct TropicalFarm;

impl Farm for TropicalFarm {
    fn climate(&self) -> &'static str {
        "tropical"
    }
    fn create_fruit(&self) -> Box<dyn Fruit> {
        Box::new(Mango)
    }
    fn create_vegetable(&self) -> Box<dyn Vegetable> {
        Box::new(Okra)
    }
}

struct TemperateFarm;

impl Farm for TemperateFarm {
    fn climate(&self) -> &'static str {
        "temperate"
    }
    fn create_fruit(&self) -> Box<dyn Fruit> {
        Box::new(Apple)
    }
    fn create_vegetable(&self) -> Box<dyn Vegetable> {
        Box::new(Carrot)
    }
}

// ---- The client: knows only the Farm trait ------------------------------------

/// A basket made from whatever farm it's given. It can't mix a mango with
/// a carrot by accident, because both come from the same farm.
struct Basket {
    fruit: Box<dyn Fruit>,
    vegetable: Box<dyn Vegetable>,
}

fn pack_basket(farm: &dyn Farm) -> Basket {
    Basket {
        fruit: farm.create_fruit(),
        vegetable: farm.create_vegetable(),
    }
}

fn describe(basket: &Basket) -> String {
    let ripe = match basket.fruit.ripe_in_days() {
        0 => "ripe now".to_string(),
        days => format!("ripe in {days} days"),
    };
    let cooking = if basket.vegetable.needs_cooking() {
        "cook it"
    } else {
        "eat it raw"
    };
    format!(
        "one {} ({ripe}) and one {} ({cooking})",
        basket.fruit.name(),
        basket.vegetable.name()
    )
}

/// Picks a farm at runtime, for example from where the customer lives.
fn farm_for(region: &str) -> Box<dyn Farm> {
    match region {
        "Brazil" | "India" | "Kenya" => Box::new(TropicalFarm),
        _ => Box::new(TemperateFarm),
    }
}

fn main() {
    for region in ["India", "Hungary"] {
        let farm = farm_for(region);
        let basket = pack_basket(farm.as_ref());
        println!(
            "{region:<8} gets a {} set: {}",
            farm.climate(),
            describe(&basket)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tropical_farm_makes_a_tropical_set() {
        let basket = pack_basket(&TropicalFarm);
        assert_eq!(basket.fruit.name(), "mango");
        assert_eq!(basket.vegetable.name(), "okra");
    }

    #[test]
    fn temperate_farm_makes_a_temperate_set() {
        let basket = pack_basket(&TemperateFarm);
        assert_eq!(basket.fruit.name(), "apple");
        assert_eq!(basket.vegetable.name(), "carrot");
    }

    #[test]
    fn the_region_decides_the_farm() {
        assert_eq!(farm_for("Kenya").climate(), "tropical");
        assert_eq!(farm_for("Hungary").climate(), "temperate");
    }

    #[test]
    fn description_uses_both_products() {
        let text = describe(&pack_basket(&TemperateFarm));
        assert_eq!(text, "one apple (ripe now) and one carrot (eat it raw)");
    }
}
