// Observer pattern: a weather station that notifies every display subscribed
// to it whenever a new reading comes in. The station doesn't know what the
// displays do with the data; it just tells them all.

use std::cell::RefCell;
use std::rc::Rc;

/// The data that gets sent to every observer.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Reading {
    temperature_c: f64,
    humidity_percent: f64,
}

/// The observer interface: anything that wants to hear about new readings.
trait Observer {
    fn update(&self, reading: &Reading);
}

/// Returned when subscribing, so the observer can unsubscribe later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SubscriptionId(u32);

// ---- The subject: the thing being watched -------------------------------

/// The weather station keeps a list of observers and notifies them all.
struct WeatherStation {
    observers: Vec<(SubscriptionId, Rc<dyn Observer>)>,
    next_id: u32,
}

impl WeatherStation {
    fn new() -> Self {
        WeatherStation {
            observers: Vec::new(),
            next_id: 0,
        }
    }

    /// Adds an observer and returns an id that can be used to remove it.
    ///
    /// The station stores an `Rc` (a shared pointer), so whoever created the
    /// observer can keep using it too.
    fn subscribe(&mut self, observer: Rc<dyn Observer>) -> SubscriptionId {
        let id = SubscriptionId(self.next_id);
        self.next_id += 1;
        self.observers.push((id, observer));
        id
    }

    /// Removes an observer. Returns `false` if it wasn't subscribed.
    fn unsubscribe(&mut self, id: SubscriptionId) -> bool {
        let before = self.observers.len();
        self.observers.retain(|(existing, _)| *existing != id);
        self.observers.len() != before
    }

    /// Called whenever the sensors produce a new reading.
    fn publish(&self, reading: Reading) {
        for (_, observer) in &self.observers {
            observer.update(&reading);
        }
    }
}

// ---- Concrete observers --------------------------------------------------

/// Shows the latest reading on screen.
struct CurrentConditionsDisplay;

impl Observer for CurrentConditionsDisplay {
    fn update(&self, reading: &Reading) {
        println!(
            "  [current]  {:.1} °C, {:.0}% humidity",
            reading.temperature_c, reading.humidity_percent
        );
    }
}

/// Remembers every temperature to show the minimum and maximum.
///
/// `update` only gets `&self` (a shared reference), but this observer needs
/// to change its own data. `RefCell` allows that: it checks the borrowing
/// rules at runtime instead of compile time.
struct StatisticsDisplay {
    temperatures: RefCell<Vec<f64>>,
}

impl StatisticsDisplay {
    fn new() -> Self {
        StatisticsDisplay {
            temperatures: RefCell::new(Vec::new()),
        }
    }

    fn min_max(&self) -> Option<(f64, f64)> {
        let temperatures = self.temperatures.borrow();
        let min = temperatures.iter().copied().reduce(f64::min)?;
        let max = temperatures.iter().copied().reduce(f64::max)?;
        Some((min, max))
    }
}

impl Observer for StatisticsDisplay {
    fn update(&self, reading: &Reading) {
        self.temperatures.borrow_mut().push(reading.temperature_c);
        if let Some((min, max)) = self.min_max() {
            println!("  [stats]    min {min:.1} °C, max {max:.1} °C");
        }
    }
}

/// Only speaks up when it's dangerously hot.
struct HeatAlert {
    threshold_c: f64,
}

impl Observer for HeatAlert {
    fn update(&self, reading: &Reading) {
        if reading.temperature_c >= self.threshold_c {
            println!(
                "  [ALERT]    heat warning: {:.1} °C!",
                reading.temperature_c
            );
        }
    }
}

fn main() {
    let mut station = WeatherStation::new();

    // Keep our own handle to the statistics display so we can read it later.
    let stats = Rc::new(StatisticsDisplay::new());

    station.subscribe(Rc::new(CurrentConditionsDisplay));
    station.subscribe(stats.clone());
    let alert_id = station.subscribe(Rc::new(HeatAlert { threshold_c: 30.0 }));

    let readings = [
        Reading {
            temperature_c: 22.5,
            humidity_percent: 60.0,
        },
        Reading {
            temperature_c: 31.0,
            humidity_percent: 40.0,
        },
        Reading {
            temperature_c: 27.0,
            humidity_percent: 55.0,
        },
    ];

    for reading in readings {
        println!("New reading:");
        station.publish(reading);
    }

    // Observers can come and go while the program runs.
    station.unsubscribe(alert_id);
    println!("\nHeat alert unsubscribed. New reading:");
    station.publish(Reading {
        temperature_c: 35.0,
        humidity_percent: 30.0,
    });

    if let Some((min, max)) = stats.min_max() {
        println!("\nStatistics kept by the display: min {min:.1} °C, max {max:.1} °C");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A test observer that records everything it receives.
    struct Recorder {
        received: RefCell<Vec<Reading>>,
    }

    impl Recorder {
        fn new() -> Rc<Self> {
            Rc::new(Recorder {
                received: RefCell::new(Vec::new()),
            })
        }
    }

    impl Observer for Recorder {
        fn update(&self, reading: &Reading) {
            self.received.borrow_mut().push(*reading);
        }
    }

    const MILD: Reading = Reading {
        temperature_c: 20.0,
        humidity_percent: 50.0,
    };
    const HOT: Reading = Reading {
        temperature_c: 33.0,
        humidity_percent: 20.0,
    };

    #[test]
    fn every_observer_is_notified() {
        let mut station = WeatherStation::new();
        let first = Recorder::new();
        let second = Recorder::new();
        station.subscribe(first.clone());
        station.subscribe(second.clone());

        station.publish(MILD);

        assert_eq!(*first.received.borrow(), vec![MILD]);
        assert_eq!(*second.received.borrow(), vec![MILD]);
    }

    #[test]
    fn unsubscribed_observers_stop_receiving() {
        let mut station = WeatherStation::new();
        let recorder = Recorder::new();
        let id = station.subscribe(recorder.clone());

        station.publish(MILD);
        assert!(station.unsubscribe(id));
        station.publish(HOT);

        assert_eq!(*recorder.received.borrow(), vec![MILD]);
        assert!(!station.unsubscribe(id)); // already gone
    }

    #[test]
    fn publishing_with_no_observers_is_fine() {
        WeatherStation::new().publish(MILD);
    }

    #[test]
    fn statistics_display_tracks_min_and_max() {
        let stats = StatisticsDisplay::new();
        assert_eq!(stats.min_max(), None);
        stats.update(&MILD);
        stats.update(&HOT);
        assert_eq!(stats.min_max(), Some((20.0, 33.0)));
    }
}
