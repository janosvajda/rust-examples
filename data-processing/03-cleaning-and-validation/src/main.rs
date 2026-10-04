// Lesson 3: cleaning and validation.
//
// Real data is messy: missing values, "n/a", different spellings of the same
// unit, impossible numbers, duplicate rows. Cleaning turns it into data you
// can trust, and the golden rule is: never change or drop anything silently.
// Every fix and every rejected row is reported, with its line number.

use std::collections::HashMap;
use std::fmt;

/// One canonical unit per kind of measurement. Everything is converted to these.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Unit {
    Celsius,
    MmHg,
    Ph,
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Unit::Celsius => "Celsius",
            Unit::MmHg => "mmHg",
            Unit::Ph => "pH",
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Reading {
    timestamp: String,
    sensor: String,
    value: f64,
    unit: Unit,
}

/// Why a row was rejected. One variant per kind of problem, so a report can
/// count them, and tests can check exactly which problem was found.
#[derive(Debug, Clone, PartialEq)]
enum Problem {
    WrongFieldCount(usize),
    MissingValue,
    NotANumber(String),
    UnknownUnit(String),
    OutOfRange { value: f64, unit: Unit, allowed: &'static str },
    /// Same sensor, same time, but a DIFFERENT value: one of them is wrong.
    Conflict { first_line: usize, kept: f64, other: f64 },
}

impl fmt::Display for Problem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Problem::WrongFieldCount(n) => write!(f, "expected 4 fields, found {n}"),
            Problem::MissingValue => write!(f, "the value is missing"),
            Problem::NotANumber(text) => write!(f, "`{text}` is not a valid number"),
            Problem::UnknownUnit(text) => write!(f, "unknown unit `{text}`"),
            Problem::OutOfRange { value, unit, allowed } => {
                write!(f, "{value} {unit} is impossible (allowed: {allowed})")
            }
            Problem::Conflict { first_line, kept, other } => {
                write!(f, "conflicts with line {first_line}: {other} here, {kept} there (kept the first)")
            }
        }
    }
}

// ---- 1. Normalise: many spellings, one meaning --------------------------------------------

/// A function that converts a value into the canonical unit.
type Conversion = fn(f64) -> f64;

/// Recognises the spellings people actually use, and says how to convert the
/// value into the canonical unit.
fn normalize_unit(text: &str) -> Option<(Unit, Conversion)> {
    fn same(v: f64) -> f64 {
        v
    }
    fn fahrenheit_to_celsius(f: f64) -> f64 {
        (f - 32.0) / 1.8
    }
    fn kpa_to_mmhg(kpa: f64) -> f64 {
        kpa * 7.500_62
    }
    match text.trim().to_lowercase().as_str() {
        "celsius" | "c" | "°c" => Some((Unit::Celsius, same)),
        "fahrenheit" | "f" | "°f" => Some((Unit::Celsius, fahrenheit_to_celsius)),
        "mmhg" => Some((Unit::MmHg, same)),
        "kpa" => Some((Unit::MmHg, kpa_to_mmhg)),
        "ph" => Some((Unit::Ph, same)),
        _ => None,
    }
}

// ---- 2. Validate: is the value possible at all? -------------------------------------------

fn check_range(value: f64, unit: Unit) -> Result<(), Problem> {
    let (ok, allowed) = match unit {
        Unit::Celsius => (value >= -273.15, "≥ -273.15, absolute zero"),
        Unit::MmHg => (value > 0.0, "> 0"),
        Unit::Ph => ((0.0..=14.0).contains(&value), "0 to 14"),
    };
    if ok { Ok(()) } else { Err(Problem::OutOfRange { value, unit, allowed }) }
}

// ---- 3. Clean one row: parse, normalise, validate, and note every change --------------------

fn clean_row(line: &str) -> Result<(Reading, Vec<String>), Problem> {
    let fields: Vec<&str> = line.split(',').collect();
    let [timestamp, sensor, value, unit] = fields[..] else {
        return Err(Problem::WrongFieldCount(fields.len()));
    };
    let mut fixes = Vec::new();

    let clean_sensor = sensor.trim().to_uppercase();
    if clean_sensor != sensor {
        fixes.push(format!("sensor `{sensor}` → `{clean_sensor}`"));
    }

    let value = match value.trim() {
        "" | "n/a" | "NA" | "-" => return Err(Problem::MissingValue), // the usual "no value" markers
        text => text
            .parse::<f64>()
            .ok()
            .filter(|v| v.is_finite()) // "NaN" and "inf" parse as f64, but aren't measurements
            .ok_or_else(|| Problem::NotANumber(text.to_string()))?,
    };

    let (unit_kind, convert) = normalize_unit(unit).ok_or_else(|| Problem::UnknownUnit(unit.trim().to_string()))?;
    let converted = convert(value);
    if converted != value {
        fixes.push(format!("{value} {} → {converted:.1} {unit_kind}", unit.trim()));
    } else if unit != unit_kind.to_string() {
        fixes.push(format!("unit `{unit}` → `{unit_kind}`"));
    }
    check_range(converted, unit_kind)?;

    let reading = Reading { timestamp: timestamp.trim().to_string(), sensor: clean_sensor, value: converted, unit: unit_kind };
    Ok((reading, fixes))
}

// ---- 4. Clean the whole file: keep going, collect everything --------------------------------

#[derive(Debug, Default)]
struct Report {
    readings: Vec<Reading>,
    fixes: Vec<(usize, String)>,
    problems: Vec<(usize, Problem)>,
    duplicates: Vec<usize>,
}

fn clean(text: &str) -> Report {
    let mut report = Report::default();
    // (timestamp, sensor) → (line number, value) of the reading already kept
    let mut seen: HashMap<(String, String), (usize, f64)> = HashMap::new();
    for (index, line) in text.lines().enumerate().skip(1) {
        let line_number = index + 1;
        if line.trim().is_empty() {
            continue;
        }
        match clean_row(line) {
            Ok((reading, fixes)) => {
                // A duplicate is the same sensor at the same time, after cleaning:
                // " a12 " and "A12" are the same sensor.
                let key = (reading.timestamp.clone(), reading.sensor.clone());
                if let Some(&(first_line, kept)) = seen.get(&key) {
                    if kept == reading.value {
                        report.duplicates.push(line_number); // a harmless repeat
                    } else {
                        let conflict = Problem::Conflict { first_line, kept, other: reading.value };
                        report.problems.push((line_number, conflict)); // a person must decide
                    }
                    continue;
                }
                seen.insert(key, (line_number, reading.value));
                report.fixes.extend(fixes.into_iter().map(|fix| (line_number, fix)));
                report.readings.push(reading);
            }
            Err(problem) => report.problems.push((line_number, problem)),
        }
    }
    report
}

fn main() {
    let text = include_str!("../data/messy_readings.csv");
    let report = clean(text);
    let rows = text.lines().skip(1).filter(|l| !l.trim().is_empty()).count();

    println!("{rows} rows in, {} clean readings out\n", report.readings.len());

    println!("Kept ({}):", report.readings.len());
    for r in &report.readings {
        println!("    {} {:<4} {:>6.1} {}", r.timestamp, r.sensor, r.value, r.unit);
    }

    println!("\nFixed along the way ({}):", report.fixes.len());
    for (line, fix) in &report.fixes {
        println!("    line {line:>2}: {fix}");
    }

    println!("\nRejected ({}):", report.problems.len());
    for (line, problem) in &report.problems {
        println!("    line {line:>2}: {problem}");
    }

    println!("\nDuplicates dropped ({}): lines {:?}", report.duplicates.len(), report.duplicates);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spellings_are_normalised() {
        let (reading, fixes) = clean_row("t, a12 ,24.1,celsius").unwrap();
        assert_eq!(reading.sensor, "A12");
        assert_eq!(reading.unit, Unit::Celsius);
        assert_eq!(fixes.len(), 2);
    }

    #[test]
    fn units_are_converted() {
        let (reading, _) = clean_row("t,A12,212,°F").unwrap();
        assert!((reading.value - 100.0).abs() < 1e-9);
        let (reading, _) = clean_row("t,B22,100,kPa").unwrap();
        assert!((reading.value - 750.062).abs() < 1e-9);
    }

    #[test]
    fn each_problem_is_named() {
        assert_eq!(clean_row("t,A12,,Celsius").unwrap_err(), Problem::MissingValue);
        assert_eq!(clean_row("t,A12,n/a,Celsius").unwrap_err(), Problem::MissingValue);
        assert_eq!(clean_row("t,A12,warm,Celsius").unwrap_err(), Problem::NotANumber(String::from("warm")));
        assert_eq!(clean_row("t,B22,755").unwrap_err(), Problem::WrongFieldCount(3));
        assert_eq!(clean_row("t,B22,750,mHg").unwrap_err(), Problem::UnknownUnit(String::from("mHg")));
        assert!(matches!(clean_row("t,A12,-300,Celsius"), Err(Problem::OutOfRange { .. })));
        assert!(matches!(clean_row("t,C33,15.2,pH"), Err(Problem::OutOfRange { .. })));
        assert_eq!(clean_row("t,A12,inf,Celsius").unwrap_err(), Problem::NotANumber(String::from("inf")));
        assert_eq!(clean_row("t,A12,NaN,Celsius").unwrap_err(), Problem::NotANumber(String::from("NaN")));
    }

    #[test]
    fn duplicates_and_conflicts_are_told_apart() {
        let report = clean("h\nt,A12,20.0,C\nt,A12,20.0,C\nt,A12,21.0,C\n");
        assert_eq!(report.readings.len(), 1);
        assert_eq!(report.duplicates, [3]); // same value: a repeat
        assert!(matches!(report.problems[..], [(4, Problem::Conflict { first_line: 2, .. })]));
    }

    #[test]
    fn empty_and_header_only_input() {
        assert!(clean("").readings.is_empty());
        assert!(clean("timestamp,sensor,value,unit\n").problems.is_empty());
    }

    #[test]
    fn the_messy_file_is_fully_accounted_for() {
        let text = include_str!("../data/messy_readings.csv");
        let report = clean(text);
        let rows = text.lines().skip(1).filter(|l| !l.trim().is_empty()).count();
        // every row ends up somewhere: nothing disappears silently
        assert_eq!(report.readings.len() + report.problems.len() + report.duplicates.len(), rows);
        assert_eq!(report.duplicates, [13]);
    }
}
