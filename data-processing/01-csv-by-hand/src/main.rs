// Lesson 1: CSV by hand.
//
// CSV ("comma-separated values") looks trivial: split each line at the commas.
// It isn't quite: a field may itself contain a comma, so it's wrapped in
// quotes, and a quote inside quotes is written twice. This lesson reads a CSV
// file into structs with only the standard library, handles those rules, and
// reports errors instead of silently skipping data.

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// One row of the file, with each field in its proper type.
#[derive(Debug, Clone, PartialEq)]
struct Reading {
    timestamp: String,
    sensor_id: String,
    value: f64,
    unit: String,
    location: String,
    experiment: String,
}

// ---- 1. Splitting a line into fields -------------------------------------------------

/// Splits one CSV line into its fields, following the quoting rules:
///   a,b,c                 → [a] [b] [c]
///   a,"b, still b",c      → [a] [b, still b] [c]
///   a,"say ""hi""",c      → [a] [say "hi"] [c]
fn split_fields(line: &str) -> Result<Vec<String>, String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        match (c, in_quotes) {
            ('"', false) if field.is_empty() => in_quotes = true, // a quoted field starts
            ('"', true) if chars.peek() == Some(&'"') => {
                field.push('"'); // "" inside quotes is one literal quote
                chars.next();
            }
            ('"', true) => in_quotes = false, // the quoted part ends
            (',', false) => fields.push(std::mem::take(&mut field)), // end of a field
            _ => field.push(c),
        }
    }
    if in_quotes {
        return Err(String::from("a quoted field is never closed"));
    }
    fields.push(field);
    Ok(fields)
}

// ---- 2. Turning fields into a struct ---------------------------------------------------

fn parse_reading(fields: &[String]) -> Result<Reading, String> {
    let [timestamp, sensor_id, value, unit, location, experiment] = fields else {
        return Err(format!("expected 6 fields, found {}", fields.len()));
    };
    let value = value
        .trim()
        .parse::<f64>()
        .map_err(|_| format!("`{value}` is not a number"))?;
    // "NaN", "inf" and "1e400" all parse successfully as f64, so check explicitly.
    if !value.is_finite() {
        return Err(format!("`{value}` is not a finite number"));
    }
    Ok(Reading {
        timestamp: timestamp.trim().to_string(),
        sensor_id: sensor_id.trim().to_string(),
        value,
        unit: unit.trim().to_string(),
        location: location.trim().to_string(),
        experiment: experiment.trim().to_string(),
    })
}

// ---- 3. Reading the whole file -------------------------------------------------------------

/// Reads every row. A missing file or a bad row is an ERROR that names the line,
/// never a silently empty result.
fn read_readings(path: &Path) -> Result<Vec<Reading>, Box<dyn Error>> {
    let file = File::open(path).map_err(|e| format!("can't open {}: {e}", path.display()))?;
    let mut readings = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line_number = index + 1;
        // A line that isn't valid UTF-8 text is an error too, and says which line.
        let line = line.map_err(|e| format!("line {line_number}: {e}"))?;
        if line_number == 1 || line.trim().is_empty() {
            continue; // the header, and blank lines
        }
        let fields = split_fields(&line).map_err(|e| format!("line {line_number}: {e}"))?;
        let reading = parse_reading(&fields).map_err(|e| format!("line {line_number}: {e}"))?;
        readings.push(reading);
    }
    Ok(readings)
}

// ---- 4. A small pipeline: filter, then transform ---------------------------------------

/// Keep temperature readings only, converted from Celsius to Fahrenheit.
fn temperatures_in_fahrenheit(readings: &[Reading]) -> Vec<Reading> {
    readings
        .iter()
        .filter(|r| r.unit == "Celsius")
        .map(|r| Reading { value: r.value * 1.8 + 32.0, unit: String::from("Fahrenheit"), ..r.clone() })
        .collect()
}

fn main() -> Result<(), Box<dyn Error>> {
    println!("1. Why splitting at commas isn't enough");
    let line = r#"2024-09-10 12:30,D44,21.8,Celsius,"Lab 4, cold room",Temperature Monitoring"#;
    let naive: Vec<&str> = line.split(',').collect();
    println!("    split(','):    {} fields: {naive:?}", naive.len());
    let proper = split_fields(line)?;
    println!("    split_fields:  {} fields: {proper:?}", proper.len());

    println!("\n2. Reading the file");
    // CARGO_MANIFEST_DIR is this crate's folder, so the path works wherever you run from.
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/sensor_readings.csv");
    let readings = read_readings(&path)?;
    println!("    {} readings", readings.len());
    for r in &readings {
        println!("    {} {:<4} {:>6} {:<8} {:<17} {}", r.timestamp, r.sensor_id, r.value, r.unit, r.location, r.experiment);
    }

    println!("\n3. Pipeline: temperatures only, in Fahrenheit");
    for r in temperatures_in_fahrenheit(&readings) {
        println!("    {} {:<4} {:.1} °F ({})", r.timestamp, r.sensor_id, r.value, r.location);
    }

    println!("\n4. Errors name the problem and the line");
    match read_readings(Path::new("no/such/file.csv")) {
        Ok(_) => println!("    unexpectedly found the file"),
        Err(error) => println!("    {error}"),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_fields() {
        assert_eq!(split_fields("a,b,c").unwrap(), ["a", "b", "c"]);
        assert_eq!(split_fields("a,,c").unwrap(), ["a", "", "c"]); // an empty field
    }

    #[test]
    fn quoted_fields_may_contain_commas_and_quotes() {
        assert_eq!(split_fields(r#"a,"b, still b",c"#).unwrap(), ["a", "b, still b", "c"]);
        assert_eq!(split_fields(r#""say ""hi""",x"#).unwrap(), [r#"say "hi""#, "x"]);
    }

    #[test]
    fn unclosed_quote_is_an_error() {
        assert!(split_fields(r#"a,"never closed"#).is_err());
    }

    #[test]
    fn rows_become_readings() {
        let fields = split_fields("t,A1, 23.5 ,Celsius,Lab 1,Test").unwrap();
        let reading = parse_reading(&fields).unwrap();
        assert_eq!(reading.value, 23.5);
        assert_eq!(reading.location, "Lab 1");
        assert!(parse_reading(&split_fields("t,A1,warm,Celsius,Lab 1,Test").unwrap()).is_err());
        assert!(parse_reading(&split_fields("too,few").unwrap()).is_err());
    }

    #[test]
    fn the_data_file_reads_completely() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/sensor_readings.csv");
        let readings = read_readings(&path).unwrap();
        assert_eq!(readings.len(), 8);
        assert_eq!(readings[7].location, "Lab 4, cold room");
        assert_eq!(readings[7].experiment, r#"Temperature Monitoring, "backup" sensor"#);
    }

    #[test]
    fn nan_and_infinity_are_rejected() {
        for value in ["NaN", "inf", "-infinity", "1e400"] {
            let fields = split_fields(&format!("t,A1,{value},Celsius,Lab 1,Test")).unwrap();
            assert!(parse_reading(&fields).is_err(), "{value} should be rejected");
        }
    }

    fn read_text(name: &str, bytes: &[u8]) -> Result<Vec<Reading>, Box<dyn Error>> {
        let path = std::env::temp_dir().join(name);
        std::fs::write(&path, bytes).unwrap();
        let result = read_readings(&path);
        std::fs::remove_file(&path).unwrap();
        result
    }

    #[test]
    fn edge_cases_of_whole_files() {
        // Windows line endings
        assert_eq!(read_text("csv-crlf.csv", b"h\r\nt,A1,1,C,L,E\r\n").unwrap()[0].experiment, "E");
        // an empty file, and a file with only a header
        assert!(read_text("csv-empty.csv", b"").unwrap().is_empty());
        assert!(read_text("csv-header.csv", b"Timestamp,SensorID\n").unwrap().is_empty());
        // no newline at the very end
        assert_eq!(read_text("csv-no-newline.csv", b"h\nt,A1,1,C,L,E").unwrap().len(), 1);
        // bytes that aren't UTF-8: an error that names the line
        let error = read_text("csv-binary.csv", b"h\nt,A1,1,C,L,E\n\xff\xfe\n").unwrap_err();
        assert!(error.to_string().starts_with("line 3:"), "{error}");
        // a line break inside a quoted field isn't supported: a clear error, not wrong data
        let error = read_text("csv-multiline.csv", b"h\nt,A1,1,C,\"two\nlines\",E\n").unwrap_err();
        assert!(error.to_string().contains("never closed"), "{error}");
    }

    #[test]
    fn missing_file_is_an_error_not_an_empty_list() {
        assert!(read_readings(Path::new("no/such/file.csv")).is_err());
    }

    #[test]
    fn pipeline_keeps_and_converts_temperatures() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/sensor_readings.csv");
        let fahrenheit = temperatures_in_fahrenheit(&read_readings(&path).unwrap());
        assert_eq!(fahrenheit.len(), 4);
        assert!((fahrenheit[0].value - 74.3).abs() < 1e-9); // 23.5 °C
        assert!(fahrenheit.iter().all(|r| r.unit == "Fahrenheit"));
    }
}
