// Lesson 5: JSON with serde.
//
// serde ("SERialise / DEserialise") turns Rust structs into JSON (and many
// other formats) and back. You describe the shape once with a derive, and
// serde writes the parsing code: typed, fast, and with precise error messages.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The JSON uses camelCase names ("sensorId"); Rust uses snake_case.
/// `rename_all` maps between them, so both sides keep their own style.
/// `deny_unknown_fields`: a field we don't know is an error, so a typo like
/// "calibrate" can't be silently ignored (and `calibrated` silently default to false).
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Reading {
    sensor_id: String,
    taken_at: String,
    value: f64,
    unit: Unit,
    location: String,
    /// A field that may be missing: absent in the JSON → None.
    note: Option<String>,
    /// A field with a default: absent in the JSON → false.
    #[serde(default)]
    calibrated: bool,
}

/// Only these units are accepted. Anything else is an error, not a string to
/// check later. In the JSON they're written in lower case.
#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
enum Unit {
    Celsius,
    MmHg,
    Ph,
}

/// What we write back out: a summary per sensor.
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct SensorSummary {
    sensor_id: String,
    unit: Unit,
    readings: usize,
    average: f64,
    /// Left out of the JSON entirely when there are no notes.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,
}

fn summarize(readings: &[Reading]) -> Vec<SensorSummary> {
    let mut groups: BTreeMap<&str, Vec<&Reading>> = BTreeMap::new();
    for reading in readings {
        groups.entry(&reading.sensor_id).or_default().push(reading);
    }
    groups
        .into_iter()
        .map(|(sensor_id, rs)| SensorSummary {
            sensor_id: sensor_id.to_string(),
            unit: rs[0].unit,
            readings: rs.len(),
            average: rs.iter().map(|r| r.value).sum::<f64>() / rs.len() as f64,
            notes: rs.iter().filter_map(|r| r.note.clone()).collect(),
        })
        .collect()
}

fn main() -> Result<(), serde_json::Error> {
    println!("1. JSON → structs");
    let readings: Vec<Reading> = serde_json::from_str(include_str!("../data/readings.json"))?;
    for r in &readings {
        println!(
            "    {} {} {:>5} {:?} calibrated={} note={:?}",
            r.taken_at, r.sensor_id, r.value, r.unit, r.calibrated, r.note
        );
    }

    println!("\n2. Structs → JSON");
    println!("{}", serde_json::to_string_pretty(&summarize(&readings))?);

    println!("\n3. Errors say what and where");
    let broken = [
        r#"{ "sensorId": "A12" "takenAt": "t" }"#,
        r#"{ "sensorId": "A12", "takenAt": "t", "value": "warm", "unit": "celsius", "location": "L" }"#,
        r#"{ "sensorId": "A12", "takenAt": "t", "value": 1, "unit": "kelvin", "location": "L" }"#,
        r#"{ "sensorId": "A12", "takenAt": "t", "value": 1, "unit": "celsius" }"#,
        r#"{ "sensorId": "A12", "takenAt": "t", "value": 1, "unit": "celsius", "location": "L", "calibrate": true }"#,
    ];
    for json in broken {
        if let Err(error) = serde_json::from_str::<Reading>(json) {
            println!("    {error}");
        }
    }

    println!("\n4. JSON Lines: one object per line, read one line at a time");
    let jsonl = "{\"sensor\":\"A12\",\"value\":23.5}\n{\"sensor\":\"B22\",\"value\":760}\n";
    for line in jsonl.lines() {
        let value: serde_json::Value = serde_json::from_str(line)?;
        println!("    sensor {} → {}", value["sensor"], value["value"]);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_case_json_becomes_a_struct() {
        let json = r#"{ "sensorId": "A1", "takenAt": "t", "value": 2.5, "unit": "ph", "location": "L" }"#;
        let r: Reading = serde_json::from_str(json).unwrap();
        assert_eq!(r.sensor_id, "A1");
        assert_eq!(r.unit, Unit::Ph);
        assert_eq!(r.note, None); // missing → None
        assert!(!r.calibrated); // missing → default
    }

    #[test]
    fn unknown_units_and_wrong_types_are_rejected() {
        let bad_unit = r#"{ "sensorId": "A1", "takenAt": "t", "value": 1, "unit": "kelvin", "location": "L" }"#;
        assert!(serde_json::from_str::<Reading>(bad_unit).unwrap_err().to_string().contains("unknown variant `kelvin`"));
        let bad_value = r#"{ "sensorId": "A1", "takenAt": "t", "value": "x", "unit": "ph", "location": "L" }"#;
        assert!(serde_json::from_str::<Reading>(bad_value).is_err());
    }

    #[test]
    fn tricky_json_is_rejected_with_a_reason() {
        let base = r#""sensorId": "A1", "takenAt": "t", "unit": "ph", "location": "L""#;
        let error = |json: String| serde_json::from_str::<Reading>(&json).unwrap_err().to_string();
        assert!(error(format!("{{ {base}, \"value\": NaN }}")).contains("expected value")); // JSON has no NaN
        assert!(error(format!("{{ {base}, \"value\": 1e400 }}")).contains("number out of range"));
        assert!(error(format!("{{ {base}, \"value\": 1, \"value\": 2 }}")).contains("duplicate field"));
        assert!(error(format!("{{ {base}, \"value\": 1, \"calibrate\": true }}")).contains("unknown field `calibrate`"));
        assert!(error(format!("{{ {base}, \"value\": 1 }} extra")).contains("trailing characters"));
    }

    #[test]
    fn deep_nesting_is_an_error_not_a_crash() {
        let deep = "[".repeat(100_000) + &"]".repeat(100_000);
        let error = serde_json::from_str::<serde_json::Value>(&deep).unwrap_err();
        assert!(error.to_string().contains("recursion limit exceeded"));
    }

    #[test]
    fn an_empty_array_is_fine() {
        let readings: Vec<Reading> = serde_json::from_str("[]").unwrap();
        assert!(readings.is_empty());
        assert!(summarize(&readings).is_empty());
    }

    #[test]
    fn round_trip() {
        let readings: Vec<Reading> = serde_json::from_str(include_str!("../data/readings.json")).unwrap();
        let again: Vec<Reading> = serde_json::from_str(&serde_json::to_string(&readings).unwrap()).unwrap();
        assert_eq!(readings, again);
    }

    #[test]
    fn empty_notes_are_left_out_of_the_output() {
        let readings: Vec<Reading> = serde_json::from_str(include_str!("../data/readings.json")).unwrap();
        let json = serde_json::to_string(&summarize(&readings)).unwrap();
        assert_eq!(json.matches("notes").count(), 1); // only C33 has a note
    }
}
