// Adapter pattern: our smart-home app works with temperature sensors that
// report in Celsius. We want to use an old sensor library that reports in
// tenths of a degree Fahrenheit and has a completely different interface.
// We can't change that library, so we write an adapter that makes it look
// like one of our own sensors.

// ---- Our application's interface (the "target") -------------------------

/// What our app expects from every temperature sensor.
trait TemperatureSensor {
    fn name(&self) -> String;
    fn celsius(&self) -> f64;
}

/// A sensor that already fits: it was written for our app.
struct ModernSensor {
    room: String,
    reading_c: f64,
}

impl TemperatureSensor for ModernSensor {
    fn name(&self) -> String {
        format!("{} (modern)", self.room)
    }

    fn celsius(&self) -> f64 {
        self.reading_c
    }
}

// ---- A third-party library we can't change (the "adaptee") ---------------

/// Pretend this module is an external crate: we can use it, but not edit it.
mod legacy_thermo {
    /// An old US-made sensor. Different names, different units, and it
    /// reports whole tenths of a degree Fahrenheit as an integer.
    pub struct ThermoDevice {
        serial: u32,
        tenths_fahrenheit: i32,
    }

    impl ThermoDevice {
        pub fn connect(serial: u32, tenths_fahrenheit: i32) -> Self {
            ThermoDevice {
                serial,
                tenths_fahrenheit,
            }
        }

        pub fn serial_number(&self) -> u32 {
            self.serial
        }

        /// For example, 725 means 72.5 °F.
        pub fn read_tenths_fahrenheit(&self) -> i32 {
            self.tenths_fahrenheit
        }
    }
}

// ---- The adapter ---------------------------------------------------------

/// Wraps a legacy device and translates its interface into ours.
///
/// The adapter *owns* the legacy device and implements our trait by calling
/// the device's methods and converting the results.
struct LegacySensorAdapter {
    device: legacy_thermo::ThermoDevice,
    room: String,
}

impl LegacySensorAdapter {
    fn new(device: legacy_thermo::ThermoDevice, room: &str) -> Self {
        LegacySensorAdapter {
            device,
            room: room.to_string(),
        }
    }
}

impl TemperatureSensor for LegacySensorAdapter {
    fn name(&self) -> String {
        // Translate: the legacy device only knows its serial number.
        format!("{} (legacy #{})", self.room, self.device.serial_number())
    }

    fn celsius(&self) -> f64 {
        // Translate units: tenths of °F → °F → °C.
        let fahrenheit = self.device.read_tenths_fahrenheit() as f64 / 10.0;
        (fahrenheit - 32.0) * 5.0 / 9.0
    }
}

// ---- Code that only knows about our interface ----------------------------

/// Works with any sensor that implements `TemperatureSensor`, so it works
/// with the adapter too, without knowing the legacy library exists.
fn average_celsius(sensors: &[Box<dyn TemperatureSensor>]) -> Option<f64> {
    if sensors.is_empty() {
        return None;
    }
    let total: f64 = sensors.iter().map(|sensor| sensor.celsius()).sum();
    Some(total / sensors.len() as f64)
}

fn main() {
    // A mix of modern sensors and adapted legacy ones, all in one list.
    let sensors: Vec<Box<dyn TemperatureSensor>> = vec![
        Box::new(ModernSensor {
            room: "Living room".to_string(),
            reading_c: 21.5,
        }),
        Box::new(LegacySensorAdapter::new(
            legacy_thermo::ThermoDevice::connect(4471, 725), // 72.5 °F
            "Bedroom",
        )),
        Box::new(LegacySensorAdapter::new(
            legacy_thermo::ThermoDevice::connect(4472, 662), // 66.2 °F
            "Garage",
        )),
    ];

    for sensor in &sensors {
        println!("{:<28} {:5.1} °C", sensor.name(), sensor.celsius());
    }

    if let Some(average) = average_celsius(&sensors) {
        println!("\nAverage temperature in the house: {average:.1} °C");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn adapted(tenths_fahrenheit: i32) -> LegacySensorAdapter {
        LegacySensorAdapter::new(
            legacy_thermo::ThermoDevice::connect(1, tenths_fahrenheit),
            "Test",
        )
    }

    fn assert_close(actual: f64, expected: f64) {
        assert!((actual - expected).abs() < 1e-9, "{actual} != {expected}");
    }

    #[test]
    fn converts_known_temperatures() {
        assert_close(adapted(320).celsius(), 0.0); // freezing
        assert_close(adapted(2120).celsius(), 100.0); // boiling
        assert_close(adapted(-400).celsius(), -40.0); // where both scales meet
    }

    #[test]
    fn translates_the_name() {
        assert_eq!(adapted(700).name(), "Test (legacy #1)");
    }

    #[test]
    fn adapter_works_wherever_a_sensor_is_expected() {
        let sensors: Vec<Box<dyn TemperatureSensor>> = vec![
            Box::new(ModernSensor {
                room: "A".to_string(),
                reading_c: 10.0,
            }),
            Box::new(adapted(860)), // 86 °F = 30 °C
        ];
        assert_close(average_celsius(&sensors).unwrap(), 20.0);
    }

    #[test]
    fn no_sensors_no_average() {
        assert_eq!(average_celsius(&[]), None);
    }
}
