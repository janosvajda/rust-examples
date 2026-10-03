// Strategy pattern: a route planner that can estimate travel time in
// different ways (by car, by bike, on foot). The planner doesn't know or care
// how the estimate is calculated. It just asks whatever strategy it was given.

/// A single leg of a trip.
#[derive(Debug, Clone, Copy)]
struct Route {
    distance_km: f64,
    // Number of hills along the way. Some ways of travelling care, others don't.
    hills: u32,
}

/// The strategy interface: every way of travelling must be able to estimate
/// how many minutes a route takes.
trait TravelStrategy {
    fn name(&self) -> &str;
    fn minutes(&self, route: &Route) -> f64;
}

// ---- Concrete strategies ------------------------------------------------

/// Cars are fast, and hills don't slow them down much.
struct ByCar;

impl TravelStrategy for ByCar {
    fn name(&self) -> &str {
        "car"
    }

    fn minutes(&self, route: &Route) -> f64 {
        // Average 50 km/h in town, plus 5 minutes to park.
        route.distance_km / 50.0 * 60.0 + 5.0
    }
}

/// Bikes are slower, and every hill costs a few extra minutes.
struct ByBike;

impl TravelStrategy for ByBike {
    fn name(&self) -> &str {
        "bike"
    }

    fn minutes(&self, route: &Route) -> f64 {
        // Average 15 km/h, plus 3 minutes per hill.
        route.distance_km / 15.0 * 60.0 + route.hills as f64 * 3.0
    }
}

/// Walking is slowest, and hills hurt the most.
struct OnFoot;

impl TravelStrategy for OnFoot {
    fn name(&self) -> &str {
        "walking"
    }

    fn minutes(&self, route: &Route) -> f64 {
        // Average 5 km/h, plus 5 minutes per hill.
        route.distance_km / 5.0 * 60.0 + route.hills as f64 * 5.0
    }
}

// ---- The context: the code that *uses* a strategy -----------------------

/// The route planner holds one strategy and can swap it at any time.
///
/// `Box<dyn TravelStrategy>` means "some type that implements
/// TravelStrategy; which one is decided at runtime".
struct RoutePlanner {
    strategy: Box<dyn TravelStrategy>,
}

impl RoutePlanner {
    fn new(strategy: Box<dyn TravelStrategy>) -> Self {
        RoutePlanner { strategy }
    }

    /// Changes how travel time is calculated, without touching anything else.
    fn set_strategy(&mut self, strategy: Box<dyn TravelStrategy>) {
        self.strategy = strategy;
    }

    /// Adds up the time for every leg of the trip using the current strategy.
    fn trip_minutes(&self, legs: &[Route]) -> f64 {
        legs.iter().map(|leg| self.strategy.minutes(leg)).sum()
    }

    fn describe(&self, legs: &[Route]) -> String {
        format!(
            "by {:<8} {:>6.1} minutes",
            self.strategy.name(),
            self.trip_minutes(legs)
        )
    }
}

// ---- A lighter Rust alternative: closures as strategies -----------------

/// When a strategy is just one function, a closure is often enough. No trait
/// and no struct needed. `impl Fn(&Route) -> f64` accepts any closure or
/// function with that signature.
fn trip_minutes_with(legs: &[Route], estimate: impl Fn(&Route) -> f64) -> f64 {
    legs.iter().map(estimate).sum()
}

fn main() {
    // A trip to the office: a flat stretch, then a hilly one.
    let trip = [
        Route { distance_km: 4.0, hills: 0 },
        Route { distance_km: 6.0, hills: 3 },
    ];

    // The same planner, three different strategies.
    let mut planner = RoutePlanner::new(Box::new(ByCar));
    println!("{}", planner.describe(&trip));

    planner.set_strategy(Box::new(ByBike));
    println!("{}", planner.describe(&trip));

    planner.set_strategy(Box::new(OnFoot));
    println!("{}", planner.describe(&trip));

    // Choosing a strategy at runtime, e.g. from user input or settings.
    let raining = true;
    let strategy: Box<dyn TravelStrategy> = if raining {
        Box::new(ByCar)
    } else {
        Box::new(ByBike)
    };
    planner.set_strategy(strategy);
    println!("\nIt's raining, so the planner picked: {}", planner.describe(&trip));

    // The closure version: an e-scooter strategy written inline.
    let scooter = |route: &Route| route.distance_km / 20.0 * 60.0 + route.hills as f64;
    println!(
        "by scooter  {:>6.1} minutes (strategy written as a closure)",
        trip_minutes_with(&trip, scooter)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    const FLAT_10KM: Route = Route { distance_km: 10.0, hills: 0 };
    const HILLY_10KM: Route = Route { distance_km: 10.0, hills: 2 };

    #[test]
    fn each_strategy_calculates_differently() {
        assert_eq!(ByCar.minutes(&FLAT_10KM), 17.0); // 12 min driving + 5 parking
        assert_eq!(ByBike.minutes(&FLAT_10KM), 40.0);
        assert_eq!(OnFoot.minutes(&FLAT_10KM), 120.0);
    }

    #[test]
    fn hills_only_matter_to_some_strategies() {
        assert_eq!(ByCar.minutes(&HILLY_10KM), ByCar.minutes(&FLAT_10KM));
        assert_eq!(ByBike.minutes(&HILLY_10KM), 46.0);
        assert_eq!(OnFoot.minutes(&HILLY_10KM), 130.0);
    }

    #[test]
    fn planner_uses_whichever_strategy_it_holds() {
        let trip = [FLAT_10KM, HILLY_10KM];
        let mut planner = RoutePlanner::new(Box::new(ByCar));
        assert_eq!(planner.trip_minutes(&trip), 34.0);

        planner.set_strategy(Box::new(ByBike));
        assert_eq!(planner.trip_minutes(&trip), 86.0);
    }

    #[test]
    fn closures_work_as_strategies_too() {
        let trip = [FLAT_10KM, HILLY_10KM];
        // A strategy that ignores everything and says 1 minute per leg.
        assert_eq!(trip_minutes_with(&trip, |_| 1.0), 2.0);
        // A named function works as well.
        fn per_km(route: &Route) -> f64 {
            route.distance_km
        }
        assert_eq!(trip_minutes_with(&trip, per_km), 20.0);
    }
}
