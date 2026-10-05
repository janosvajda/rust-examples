// Factory pattern: a robot factory. You order a robot by saying what job it
// should do; the factory decides which concrete model to build. The caller
// only ever sees "a Robot", never HelperBot, GuardBot or ScoutBot directly.

use std::fmt;
use std::str::FromStr;

/// What every robot can do. The caller works only with this trait.
trait Robot {
    fn model(&self) -> &'static str;
    fn work(&self) -> String;
}

// ---- Concrete products: the caller never names these -------------------------

struct HelperBot;

impl Robot for HelperBot {
    fn model(&self) -> &'static str {
        "HelperBot"
    }
    fn work(&self) -> String {
        "carries boxes and waters the plants".to_string()
    }
}

struct GuardBot {
    shift_hours: u32,
}

impl Robot for GuardBot {
    fn model(&self) -> &'static str {
        "GuardBot"
    }
    fn work(&self) -> String {
        format!("patrols the warehouse for {} hours", self.shift_hours)
    }
}

struct ScoutBot {
    range_km: u32,
}

impl Robot for ScoutBot {
    fn model(&self) -> &'static str {
        "ScoutBot"
    }
    fn work(&self) -> String {
        format!("explores up to {} km away", self.range_km)
    }
}

// ---- The order: what the caller asks for --------------------------------------

/// The jobs you can order a robot for. An enum makes the list of valid
/// orders explicit, and the compiler checks the factory handles every one.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Job {
    Help,
    Guard,
    Explore,
}

/// Orders often arrive as text (a config file, a web request), so a job can
/// be parsed from a string. Unknown orders are an error, not a crash.
impl FromStr for Job {
    type Err = UnknownJob;

    fn from_str(text: &str) -> Result<Job, UnknownJob> {
        match text.trim().to_lowercase().as_str() {
            "help" => Ok(Job::Help),
            "guard" => Ok(Job::Guard),
            "explore" => Ok(Job::Explore),
            _ => Err(UnknownJob(text.to_string())),
        }
    }
}

#[derive(Debug, PartialEq)]
struct UnknownJob(String);

impl fmt::Display for UnknownJob {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "no robot can do \"{}\"", self.0)
    }
}

// ---- The factory: the one place that decides what to build ------------------

struct RobotFactory {
    // Settings the factory applies to every robot it builds. Callers don't
    // need to know about them.
    guard_shift_hours: u32,
    scout_range_km: u32,
}

impl RobotFactory {
    /// The factory method. Returns `Box<dyn Robot>`: "some robot", decided
    /// here. Adding a new model only means changing this function.
    fn build(&self, job: Job) -> Box<dyn Robot> {
        match job {
            Job::Help => Box::new(HelperBot),
            Job::Guard => Box::new(GuardBot {
                shift_hours: self.guard_shift_hours,
            }),
            Job::Explore => Box::new(ScoutBot {
                range_km: self.scout_range_km,
            }),
        }
    }
}

fn main() {
    let factory = RobotFactory {
        guard_shift_hours: 8,
        scout_range_km: 25,
    };

    // The orders arrive as text, e.g. typed by a user.
    let orders = ["help", "guard", "explore", "dance"];

    for order in orders {
        match order.parse::<Job>() {
            Ok(job) => {
                let robot = factory.build(job);
                println!("order \"{order}\" → {}: {}", robot.model(), robot.work());
            }
            Err(error) => println!("order \"{order}\" → refused: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn factory() -> RobotFactory {
        RobotFactory {
            guard_shift_hours: 12,
            scout_range_km: 5,
        }
    }

    #[test]
    fn factory_picks_the_model_for_each_job() {
        let f = factory();
        assert_eq!(f.build(Job::Help).model(), "HelperBot");
        assert_eq!(f.build(Job::Guard).model(), "GuardBot");
        assert_eq!(f.build(Job::Explore).model(), "ScoutBot");
    }

    #[test]
    fn factory_applies_its_settings() {
        let f = factory();
        assert_eq!(
            f.build(Job::Guard).work(),
            "patrols the warehouse for 12 hours"
        );
        assert_eq!(f.build(Job::Explore).work(), "explores up to 5 km away");
    }

    #[test]
    fn orders_are_parsed_from_text() {
        assert_eq!("Guard".parse::<Job>(), Ok(Job::Guard));
        assert_eq!(" explore ".parse::<Job>(), Ok(Job::Explore));
        assert_eq!("dance".parse::<Job>(), Err(UnknownJob("dance".to_string())));
    }
}
