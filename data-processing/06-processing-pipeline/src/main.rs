// Lesson 6: a processing pipeline.
//
// Everything from the course, as one program built from small stages:
//
//   read  →  parse & validate  →  convert units  →  aggregate  →  write a report
//
// Each stage is a plain function that's easy to test on its own. Because the
// middle stages work on one line at a time and don't share anything, the
// middle stages can use Rayon's parallel fold/reduce; reading and writing
// remain sequential.

use rayon::prelude::*;
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::path::Path;
use std::time::Instant;

// ---- The data types that flow between stages ----------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct Reading<'a> {
    sensor: &'a str,
    celsius: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Stats {
    count: u64,
    min: f64,
    max: f64,
    mean: f64,
}

/// The result of the aggregate stage: statistics per sensor, and how many
/// lines were rejected (lesson 3: never drop data without counting it).
#[derive(Debug, Clone, Default, PartialEq)]
struct Totals {
    per_sensor: BTreeMap<String, Stats>,
    rejected: u64,
}

// ---- Stage 1: read ---------------------------------------------------------------------------------

/// Loads the lines into memory so the next stages can share them out across
/// threads. (For files bigger than memory, process them in chunks: see the README.)
///
/// Reads bytes, not text: a line that isn't valid UTF-8 becomes a line with
/// an empty rejection sentinel, which the next stage rejects and counts.
/// `BufRead::lines()` would return an error instead, and end the whole run.
fn read_lines(path: &Path) -> io::Result<Vec<String>> {
    let bytes = std::fs::read(path)?;
    Ok(bytes
        .split(|&b| b == b'\n')
        .skip(1) // the header
        .filter(|line| !line.is_empty())
        .map(|line| {
            std::str::from_utf8(line)
                .map(|text| text.trim_end_matches('\r').to_owned())
                .unwrap_or_default()
        })
        .collect())
}

// ---- Stage 2: parse & validate ---------------------------------------------------------------

/// One line → a reading in its original unit, or None if the line is unusable.
fn parse(line: &str) -> Option<(Reading<'_>, &str)> {
    let mut fields = line.split(',');
    let (time, sensor, value, unit) = (
        fields.next()?,
        fields.next()?,
        fields.next()?,
        fields.next()?,
    );
    if time.is_empty() || sensor.trim().is_empty() || fields.next().is_some() {
        return None;
    }
    let value: f64 = value.parse().ok()?;
    Some((
        Reading {
            sensor,
            celsius: value,
        },
        unit,
    ))
}

// ---- Stage 3: convert units --------------------------------------------------------------------

/// Everything in Celsius from here on; unknown units and impossible values are rejected.
fn to_celsius<'a>((reading, unit): (Reading<'a>, &str)) -> Option<Reading<'a>> {
    let celsius = match unit {
        "C" => reading.celsius,
        "F" => (reading.celsius - 32.0) / 1.8,
        _ => return None,
    };
    // `is_finite`: "inf" parses as f64 and is even ≥ -273.15, but it isn't a temperature.
    (celsius.is_finite() && celsius >= -273.15).then_some(Reading { celsius, ..reading })
}

/// Stages 2 and 3 together: what happens to every single line.
fn process(line: &str) -> Option<Reading<'_>> {
    parse(line).and_then(to_celsius)
}

// ---- Stage 4: aggregate ----------------------------------------------------------------------------

impl Totals {
    /// Add one processed line (a reading, or a rejected line).
    fn add(mut self, processed: Option<Reading<'_>>) -> Self {
        match processed {
            Some(r) => match self.per_sensor.get_mut(r.sensor) {
                Some(s) => {
                    s.count += 1;
                    s.min = s.min.min(r.celsius);
                    s.max = s.max.max(r.celsius);
                    s.mean += (r.celsius - s.mean) / s.count as f64; // a running mean
                }
                None => {
                    let stats = Stats {
                        count: 1,
                        min: r.celsius,
                        max: r.celsius,
                        mean: r.celsius,
                    };
                    self.per_sensor.insert(r.sensor.to_string(), stats);
                }
            },
            None => self.rejected += 1,
        }
        self
    }

    /// Combine two partial results. Parallel code needs this: every thread
    /// builds its own Totals, and they're merged at the end.
    fn merge(mut self, other: Totals) -> Self {
        self.rejected += other.rejected;
        for (sensor, o) in other.per_sensor {
            self.per_sensor
                .entry(sensor)
                .and_modify(|s| {
                    // Each mean counts as much as the number of readings behind it.
                    let total = s.count + o.count;
                    s.mean = (s.mean * s.count as f64 + o.mean * o.count as f64) / total as f64;
                    s.count = total;
                    s.min = s.min.min(o.min);
                    s.max = s.max.max(o.max);
                })
                .or_insert(o);
        }
        self
    }
}

/// The whole middle of the pipeline, one line after another.
fn run_sequential(lines: &[String]) -> Totals {
    lines
        .iter()
        .map(|line| process(line))
        .fold(Totals::default(), Totals::add)
}

/// The same pipeline on every core: `par_iter` instead of `iter`. Each thread
/// folds its share of the lines into its own Totals; `reduce` merges them.
fn run_parallel(lines: &[String]) -> Totals {
    lines
        .par_iter()
        .map(|line| process(line))
        .fold(Totals::default, Totals::add)
        .reduce(Totals::default, Totals::merge)
}

// ---- Stage 5: write a report -------------------------------------------------------------------

fn write_report(path: &Path, totals: &Totals) -> io::Result<()> {
    let mut out = BufWriter::new(File::create(path)?);
    writeln!(out, "sensor,count,min,max,mean")?;
    for (sensor, s) in &totals.per_sensor {
        writeln!(
            out,
            "{sensor},{},{:.2},{:.2},{:.2}",
            s.count, s.min, s.max, s.mean
        )?;
    }
    out.flush()
}

// ---- Test data ---------------------------------------------------------------------------------------

/// A log with mixed units (every 7th line in Fahrenheit) and some broken lines.
fn generate_input(path: &Path, lines: u64) -> io::Result<()> {
    let sensors = ["A12", "B22", "C33", "D44", "E55", "F66", "G77", "H88"];
    let mut out = BufWriter::new(File::create(path)?);
    let mut seed: u64 = 7;
    writeln!(out, "time,sensor,value,unit")?;
    for i in 0..lines {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let sensor = sensors[(seed >> 33) as usize % sensors.len()];
        let celsius = 15.0 + (seed >> 40) as f64 / (1u64 << 24) as f64 * 15.0;
        match i {
            _ if i % 1000 == 999 => writeln!(out, "{i},{sensor},ERR,C")?, // a broken sensor
            _ if i % 7 == 0 => writeln!(out, "{i},{sensor},{:.2},F", celsius * 1.8 + 32.0)?,
            _ => writeln!(out, "{i},{sensor},{celsius:.2},C")?,
        }
    }
    out.flush()
}

fn main() -> io::Result<()> {
    let dir = std::env::temp_dir();
    let input = dir.join("rust-examples-pipeline-input.csv");
    let report = dir.join("rust-examples-pipeline-report.csv");

    generate_input(&input, 4_000_000)?;
    let lines = read_lines(&input)?;
    println!("Input: {} lines\n", lines.len());

    let start = Instant::now();
    let sequential = run_sequential(&lines);
    let sequential_time = start.elapsed();

    let start = Instant::now();
    let parallel = run_parallel(&lines);
    let parallel_time = start.elapsed();

    let cores = std::thread::available_parallelism().map_or(1, |n| n.get());
    println!("sequential: {sequential_time:>10.2?}");
    println!(
        "parallel:   {parallel_time:>10.2?}  with {cores} estimated available workers, {:.1}× faster",
        sequential_time.as_secs_f64() / parallel_time.as_secs_f64()
    );

    // Same counts, minimums and maximums. The means can differ in the last digits:
    // floating-point addition gives slightly different results in a different order.
    let counts = |t: &Totals| {
        t.per_sensor
            .values()
            .map(|s| (s.count, s.min, s.max))
            .collect::<Vec<_>>()
    };
    assert_eq!(counts(&sequential), counts(&parallel));
    assert_eq!(sequential.rejected, parallel.rejected);
    let a12 = (
        sequential.per_sensor["A12"].mean,
        parallel.per_sensor["A12"].mean,
    );
    println!(
        "\nA12 mean, sequential: {:.10}\nA12 mean, parallel:   {:.10}",
        a12.0, a12.1
    );

    write_report(&report, &parallel)?;
    println!(
        "\nReport ({}), {} lines rejected:",
        report.display(),
        parallel.rejected
    );
    print!("{}", std::fs::read_to_string(&report)?);

    std::fs::remove_file(&input)?;
    std::fs::remove_file(&report)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_stage_on_its_own() {
        assert_eq!(
            parse("1,A12,21.5,C"),
            Some((
                Reading {
                    sensor: "A12",
                    celsius: 21.5
                },
                "C"
            ))
        );
        assert_eq!(parse("1,A12,ERR,C"), None);
        assert_eq!(
            to_celsius((
                Reading {
                    sensor: "X",
                    celsius: 212.0
                },
                "F"
            )),
            Some(Reading {
                sensor: "X",
                celsius: 100.0
            })
        );
        assert_eq!(
            to_celsius((
                Reading {
                    sensor: "X",
                    celsius: 1.0
                },
                "K"
            )),
            None
        );
        assert_eq!(
            to_celsius((
                Reading {
                    sensor: "X",
                    celsius: -500.0
                },
                "C"
            )),
            None
        );
    }

    #[test]
    fn aggregate_counts_rejections() {
        let lines: Vec<String> = ["1,A,10,C", "2,A,20,C", "3,B,x,C", "4,B,50,F"]
            .map(String::from)
            .to_vec();
        let totals = run_sequential(&lines);
        assert_eq!(
            totals.per_sensor["A"],
            Stats {
                count: 2,
                min: 10.0,
                max: 20.0,
                mean: 15.0
            }
        );
        assert_eq!(totals.per_sensor["B"].count, 1);
        assert_eq!(totals.rejected, 1);
    }

    #[test]
    fn parallel_gives_the_same_answer() {
        let lines: Vec<String> = (0..10_000)
            .map(|i| format!("{i},S{},{}.5,C", i % 5, i % 40))
            .collect();
        let (seq, par) = (run_sequential(&lines), run_parallel(&lines));
        assert_eq!(seq.rejected, par.rejected);
        for (sensor, s) in &seq.per_sensor {
            let p = par.per_sensor[sensor];
            assert_eq!((s.count, s.min, s.max), (p.count, p.min, p.max));
            assert!(
                (s.mean - p.mean).abs() < 1e-6,
                "means differ by more than rounding"
            );
        }
    }

    #[test]
    fn nan_infinity_and_broken_text_are_rejected() {
        for line in [
            "1,A,NaN,C",
            "1,A,inf,C",
            "1,A,1e400,C",
            "1,A,\u{FFFD}\u{FFFD},C",
            "",
            "1,A",
        ] {
            assert_eq!(process(line), None, "{line:?} should be rejected");
        }
    }

    #[test]
    fn invalid_utf8_lines_are_counted_not_fatal() {
        let path = std::env::temp_dir().join("rust-examples-pipeline-utf8-test.csv");
        std::fs::write(
            &path,
            b"header\r\n1,A,20.0,C\r\n2,A,\xff\xfe,C\r\n3,A,22.0,C\r\n",
        )
        .unwrap();
        let totals = run_parallel(&read_lines(&path).unwrap());
        std::fs::remove_file(&path).unwrap();
        assert_eq!(totals.per_sensor["A"].count, 2);
        assert_eq!(totals.rejected, 1);
    }

    #[test]
    fn empty_input() {
        assert_eq!(run_sequential(&[]), Totals::default());
        assert_eq!(run_parallel(&[]), Totals::default());
    }

    #[test]
    fn merge_combines_partial_results() {
        let a = Totals::default().add(Some(Reading {
            sensor: "A",
            celsius: 1.0,
        }));
        let b = Totals::default()
            .add(Some(Reading {
                sensor: "A",
                celsius: 5.0,
            }))
            .add(None);
        let merged = a.merge(b);
        assert_eq!(
            merged.per_sensor["A"],
            Stats {
                count: 2,
                min: 1.0,
                max: 5.0,
                mean: 3.0
            }
        );
        assert_eq!(merged.rejected, 1);
    }
    #[test]
    fn invalid_utf8_in_an_identifier_does_not_create_a_different_sensor() {
        let path = std::env::temp_dir().join("rust-examples-pipeline-invalid-id.csv");
        std::fs::write(&path, b"header\n1,A\xff,20,C\n2,A,21,C\n").unwrap();
        let totals = run_parallel(&read_lines(&path).unwrap());
        std::fs::remove_file(&path).unwrap();
        assert_eq!(totals.rejected, 1);
        assert_eq!(totals.per_sensor.len(), 1);
        assert!(process("t,A,20,C,extra").is_none());
    }
}
