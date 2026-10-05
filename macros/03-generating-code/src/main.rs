// Lesson 3: macros that generate code.
//
// The most useful macros don't compute values; they write repetitive
// DEFINITIONS for you: the same impl for many types, a family of similar
// structs, a batch of test functions. Write the pattern once, use it many times.

use std::fmt;

// ---- 1. The same trait impl for many types ----------------------------------------------

trait Describe {
    fn describe(&self) -> String;
}

/// One invocation implements `Describe` for every type listed.
/// `$( $t:ty ),*` repeats a whole `impl` block per type.
macro_rules! impl_describe_for_numbers {
    ( $( $t:ty ),* ) => {
        $(
            impl Describe for $t {
                fn describe(&self) -> String {
                    format!("{} (a {}, {} bytes)", self, stringify!($t), std::mem::size_of::<$t>())
                }
            }
        )*
    };
}

impl_describe_for_numbers!(u8, i32, u64, f32, f64);

// ---- 2. A family of similar types: units of measurement --------------------------------

/// Generates a newtype, its Display impl, and conversions, for each unit.
/// `stringify!` turns tokens into a string at compile time.
macro_rules! units {
    ( $( $name:ident => $symbol:literal ),* $(,)? ) => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
            struct $name(f64);

            impl fmt::Display for $name {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                    write!(f, "{} {}", self.0, $symbol)
                }
            }

            impl From<f64> for $name {
                fn from(value: f64) -> Self {
                    $name(value)
                }
            }
        )*
    };
}

units! {
    Metres => "m",
    Seconds => "s",
    Kilograms => "kg",
}

// ---- 3. An enum with a list of all its variants and their names --------------------------

/// Generates an enum, plus `ALL` (every variant) and `name()`, so adding a
/// variant can never leave the list or the names out of date.
macro_rules! listed_enum {
    ( $enum_name:ident { $( $variant:ident ),* $(,)? } ) => {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum $enum_name {
            $( $variant ),*
        }

        impl $enum_name {
            const ALL: &'static [$enum_name] = &[ $( $enum_name::$variant ),* ];

            fn name(&self) -> &'static str {
                match self {
                    $( $enum_name::$variant => stringify!($variant) ),*
                }
            }
        }
    };
}

listed_enum!(Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday
});

fn speed(distance: Metres, time: Seconds) -> f64 {
    distance.0 / time.0
}

fn main() {
    println!("1. One impl, many types");
    println!("    {}", 7u8.describe());
    println!("    {}", (-40i32).describe());
    println!("    {}", 2.5f64.describe());

    println!("\n2. A family of unit types");
    let run = Metres(400.0);
    let time = Seconds::from(52.5);
    let bag = Kilograms(23.0);
    println!("    {run} in {time}: {:.2} m/s", speed(run, time));
    println!("    luggage: {bag}");
    // speed(time, run) doesn't compile: Seconds and Metres are different types,
    // so units can't be mixed up by accident.

    println!("\n3. An enum that knows all its variants");
    for day in Weekday::ALL {
        print!("    {}", day.name());
    }
    println!("\n    {} working days", Weekday::ALL.len());
}

// ---- 4. Generating test functions -----------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// Each line becomes its own #[test] function with its own name, so a
    /// failure tells you exactly which case broke.
    macro_rules! speed_tests {
        ( $( $name:ident: $metres:expr, $seconds:expr => $expected:expr ),* $(,)? ) => {
            $(
                #[test]
                fn $name() {
                    assert_eq!(speed(Metres($metres), Seconds($seconds)), $expected);
                }
            )*
        };
    }

    speed_tests! {
        walking: 100.0, 80.0 => 1.25,
        running: 400.0, 50.0 => 8.0,
        standing_still: 0.0, 10.0 => 0.0,
    }

    #[test]
    fn generated_impls_and_types() {
        assert_eq!(5u8.describe(), "5 (a u8, 1 bytes)");
        assert_eq!(Metres(3.0).to_string(), "3 m");
        assert_eq!(Kilograms::from(1.5), Kilograms(1.5));
    }

    #[test]
    fn listed_enum_is_complete() {
        assert_eq!(Weekday::ALL.len(), 5);
        assert_eq!(Weekday::Friday.name(), "Friday");
        assert_eq!(Weekday::ALL[0], Weekday::Monday);
    }
}
