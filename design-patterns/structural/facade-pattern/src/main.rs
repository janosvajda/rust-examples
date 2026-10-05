// Facade pattern: a smart home has many separate devices, each with its own
// controls. Instead of making the user operate every device one by one, a
// single `SmartHome` facade offers simple actions like "leave home" and
// "arrive home" that coordinate all of them.

// ---- The subsystems: each device has its own, detailed interface ----------

#[derive(Debug, Default)]
struct Lights {
    on_rooms: Vec<String>,
}

impl Lights {
    fn turn_on(&mut self, room: &str) {
        if !self.on_rooms.iter().any(|r| r == room) {
            self.on_rooms.push(room.to_string());
        }
    }

    fn turn_all_off(&mut self) {
        self.on_rooms.clear();
    }
}

#[derive(Debug)]
struct Thermostat {
    target_c: f64,
}

impl Thermostat {
    fn set_target(&mut self, celsius: f64) {
        self.target_c = celsius;
    }
}

#[derive(Debug, Default)]
struct DoorLock {
    locked: bool,
}

impl DoorLock {
    fn lock(&mut self) {
        self.locked = true;
    }

    fn unlock(&mut self, pin: u32) -> Result<(), String> {
        if pin == 1234 {
            self.locked = false;
            Ok(())
        } else {
            Err("wrong PIN".to_string())
        }
    }
}

#[derive(Debug, Default)]
struct Alarm {
    armed: bool,
}

impl Alarm {
    fn arm(&mut self) {
        self.armed = true;
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

// ---- The facade: one simple interface in front of all of them ------------

/// Knows which devices exist and the right order to operate them in.
/// Callers only see a few high-level actions.
struct SmartHome {
    lights: Lights,
    thermostat: Thermostat,
    door: DoorLock,
    alarm: Alarm,
}

impl SmartHome {
    fn new() -> Self {
        SmartHome {
            lights: Lights::default(),
            thermostat: Thermostat { target_c: 21.0 },
            door: DoorLock::default(),
            alarm: Alarm::default(),
        }
    }

    /// One call instead of four separate steps.
    fn leave_home(&mut self) {
        self.lights.turn_all_off();
        self.thermostat.set_target(16.0); // save energy while nobody is home
        self.door.lock();
        self.alarm.arm(); // arm last, after the door is locked
    }

    /// The order matters here too: unlock first, and only disarm the alarm
    /// if unlocking succeeded.
    fn arrive_home(&mut self, pin: u32) -> Result<(), String> {
        self.door.unlock(pin)?;
        self.alarm.disarm();
        self.thermostat.set_target(21.0);
        self.lights.turn_on("hallway");
        Ok(())
    }

    fn movie_night(&mut self) {
        self.lights.turn_all_off();
        self.lights.turn_on("living room");
        self.thermostat.set_target(22.0);
    }

    fn status(&self) -> String {
        format!(
            "lights on: {:?}, heating: {} °C, door {}, alarm {}",
            self.lights.on_rooms,
            self.thermostat.target_c,
            if self.door.locked {
                "locked"
            } else {
                "unlocked"
            },
            if self.alarm.armed { "armed" } else { "off" },
        )
    }
}

fn main() {
    let mut home = SmartHome::new();
    home.lights.turn_on("kitchen"); // someone was cooking
    println!("start:        {}", home.status());

    home.leave_home();
    println!("leave home:   {}", home.status());

    match home.arrive_home(1111) {
        Ok(()) => println!("arrive home:  {}", home.status()),
        Err(error) => println!("arrive home:  refused ({error}), {}", home.status()),
    }

    home.arrive_home(1234).unwrap();
    println!("arrive home:  {}", home.status());

    home.movie_night();
    println!("movie night:  {}", home.status());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn leaving_secures_the_house() {
        let mut home = SmartHome::new();
        home.lights.turn_on("kitchen");
        home.leave_home();
        assert!(home.lights.on_rooms.is_empty());
        assert_eq!(home.thermostat.target_c, 16.0);
        assert!(home.door.locked);
        assert!(home.alarm.armed);
    }

    #[test]
    fn arriving_with_the_right_pin_opens_up() {
        let mut home = SmartHome::new();
        home.leave_home();
        home.arrive_home(1234).unwrap();
        assert!(!home.door.locked);
        assert!(!home.alarm.armed);
        assert_eq!(home.lights.on_rooms, vec!["hallway"]);
    }

    #[test]
    fn wrong_pin_changes_nothing() {
        let mut home = SmartHome::new();
        home.leave_home();
        assert!(home.arrive_home(1111).is_err());
        // The alarm must stay armed if the door didn't open.
        assert!(home.door.locked);
        assert!(home.alarm.armed);
    }

    #[test]
    fn movie_night_leaves_only_the_living_room_lit() {
        let mut home = SmartHome::new();
        home.lights.turn_on("kitchen");
        home.movie_night();
        assert_eq!(home.lights.on_rooms, vec!["living room"]);
    }
}
