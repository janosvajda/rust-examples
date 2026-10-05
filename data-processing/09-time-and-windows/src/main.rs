// Lesson 9: time and windows.
//
// Timestamps as text can be printed, and that's about it. To sort them,
// subtract them, group them into windows or notice that one is missing, turn
// them into a number: seconds since a fixed starting point. This lesson does
// that with the standard library only, then finds gaps and averages windows.

use std::collections::BTreeMap;
use std::fmt;

// ---- 1. A real timestamp type ---------------------------------------------------------------

/// Seconds since 1970-01-01 00:00:00 UTC (the "Unix epoch"). Because it's a
/// number, comparing, sorting and subtracting timestamps just works.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Timestamp(i64);

const MINUTE: i64 = 60;
const DAY: i64 = 24 * 60 * MINUTE;

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap_year(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 for a calendar date (Howard Hinnant's algorithm:
/// count from a year that starts in March, so the leap day comes last).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let year_of_era = y - era * 400;
    let day_of_year = (153 * ((month + 9) % 12) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The reverse: a calendar date from days since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let mp = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

impl Timestamp {
    /// Parses `YYYY-MM-DD HH:MM:SS`, and rejects dates that don't exist.
    fn parse(text: &str) -> Result<Timestamp, String> {
        let bad = || format!("`{text}` is not a valid `YYYY-MM-DD HH:MM:SS` time");
        let text = text.trim();
        let (date, time) = text.split_once(' ').ok_or_else(bad)?;
        let numbers = |s: &str, sep: char| -> Result<Vec<i64>, String> {
            s.split(sep)
                .map(|n| {
                    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
                        return Err(bad());
                    }
                    n.parse::<i64>().map_err(|_| bad())
                })
                .collect()
        };
        let (d, t) = (numbers(date, '-')?, numbers(time, ':')?);
        let (&[year, month, day], &[hour, minute, second]) = (&d[..], &t[..]) else {
            return Err(bad());
        };
        // Years outside 1..=9999 are surely typos, and astronomically large ones
        // would overflow the seconds calculation.
        if !(1..=9999).contains(&year) {
            return Err(format!("year {year} is out of range (1 to 9999)"));
        }
        if !(1..=12).contains(&month) || !(1..=days_in_month(year, month)).contains(&day) {
            return Err(format!("{year}-{month:02}-{day:02} doesn't exist"));
        }
        if !(0..=23).contains(&hour) || !(0..=59).contains(&minute) || !(0..=59).contains(&second) {
            return Err(bad());
        }
        Ok(Timestamp(
            days_from_civil(year, month, day) * DAY + hour * 3600 + minute * MINUTE + second,
        ))
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (days, secs) = (self.0.div_euclid(DAY), self.0.rem_euclid(DAY));
        let (year, month, day) = civil_from_days(days);
        write!(
            f,
            "{year}-{month:02}-{day:02} {:02}:{:02}:{:02}",
            secs / 3600,
            secs / 60 % 60,
            secs % 60
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Reading {
    at: Timestamp,
    celsius: f64,
}

fn parse_readings(text: &str) -> Result<Vec<Reading>, String> {
    text.lines()
        .skip(1)
        .map(|line| {
            let mut fields = line.split(',');
            let (at, _sensor, value) = (fields.next(), fields.next(), fields.next());
            if fields.next().is_some() || _sensor.is_none_or(|s| s.trim().is_empty()) {
                return Err(format!("expected timestamp,sensor,value in `{line}`"));
            }
            let at = Timestamp::parse(at.unwrap_or_default())?;
            let celsius = value
                .and_then(|v| v.trim().parse::<f64>().ok())
                .filter(|v| v.is_finite()) // "NaN" parses as f64
                .ok_or(format!("bad value in `{line}`"))?;
            Ok(Reading { at, celsius })
        })
        .collect()
}

// ---- 2. Tumbling windows: fixed time buckets ----------------------------------------------------

/// The start of the window a timestamp falls in: 00:07 → 00:00 for 15-minute windows.
fn window_start(at: Timestamp, size: i64) -> Timestamp {
    assert!(size > 0, "window size must be positive");
    Timestamp(at.0 - at.0.rem_euclid(size))
}

/// Average temperature per window; empty windows are omitted.
fn average_per_window(readings: &[Reading], size: i64) -> BTreeMap<Timestamp, (usize, f64)> {
    let mut groups: BTreeMap<Timestamp, Vec<f64>> = BTreeMap::new();
    for r in readings {
        groups
            .entry(window_start(r.at, size))
            .or_default()
            .push(r.celsius);
    }
    groups
        .into_iter()
        .map(|(start, values)| {
            let count = values.len();
            (start, (count, values.iter().sum::<f64>() / count as f64))
        })
        .collect()
}

// ---- 3. Gaps: when the data stops ------------------------------------------------------------------

/// A gap is a pause between two readings longer than the expected interval.
/// Returns (last reading before the gap, first reading after it, readings missed).
fn find_gaps(sorted: &[Reading], expected_interval: i64) -> Vec<(Timestamp, Timestamp, i64)> {
    assert!(expected_interval > 0, "expected interval must be positive");
    sorted
        .array_windows()
        .filter(|[a, b]| b.at.0 - a.at.0 > expected_interval)
        .map(|[a, b]| (a.at, b.at, (b.at.0 - a.at.0) / expected_interval - 1))
        .collect()
}

fn main() -> Result<(), String> {
    println!("1. Text → timestamps: real dates only");
    for text in [
        "2024-09-10 23:55:00",
        "2024-02-29 12:00:00",
        "2023-02-29 12:00:00",
        "2024-13-01 00:00:00",
    ] {
        match Timestamp::parse(text) {
            Ok(ts) => println!("    {text} → {} seconds since 1970", ts.0),
            Err(error) => println!("    {text} → error: {error}"),
        }
    }

    let mut readings = parse_readings(include_str!("../data/readings.csv"))?;

    println!("\n2. Out of order: the file isn't sorted by time");
    let out_of_order = readings
        .array_windows()
        .filter(|[a, b]| b.at < a.at)
        .count();
    println!("    {out_of_order} readings arrived earlier than the one before them");
    readings.sort_by_key(|r| r.at); // a number sorts correctly, always
    println!(
        "    sorted: {} → {}",
        readings[0].at,
        readings[readings.len() - 1].at
    );
    let span = readings[readings.len() - 1].at.0 - readings[0].at.0;
    println!("    span: {} minutes, across midnight", span / MINUTE);

    println!("\n3. Gaps: the sensor reports every 5 minutes");
    for (before, after, missed) in find_gaps(&readings, 5 * MINUTE) {
        println!(
            "    silent from {before} to {after}: {} minutes, {missed} readings missing",
            (after.0 - before.0) / MINUTE
        );
    }

    println!("\n4. 15-minute windows");
    for (start, (count, average)) in average_per_window(&readings, 15 * MINUTE) {
        let note = if count < 3 {
            "  ← incomplete window"
        } else {
            ""
        };
        println!("    {start}  readings: {count}, average {average:.2} °C{note}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_timestamps() {
        // reference values from Python's datetime, in UTC
        assert_eq!(Timestamp::parse("1970-01-01 00:00:00"), Ok(Timestamp(0)));
        assert_eq!(
            Timestamp::parse("2024-09-10 12:00:00"),
            Ok(Timestamp(1_725_969_600))
        );
        assert_eq!(
            Timestamp::parse("2024-02-29 23:59:59"),
            Ok(Timestamp(1_709_251_199))
        );
        assert_eq!(
            Timestamp::parse("2000-03-01 00:00:00"),
            Ok(Timestamp(951_868_800))
        );
    }

    #[test]
    fn impossible_dates_are_rejected() {
        assert!(Timestamp::parse("2023-02-29 00:00:00").is_err()); // not a leap year
        assert!(Timestamp::parse("1900-02-29 00:00:00").is_err()); // divisible by 100
        assert!(Timestamp::parse("2000-02-29 00:00:00").is_ok()); // divisible by 400
        assert!(Timestamp::parse("2024-04-31 00:00:00").is_err());
        assert!(Timestamp::parse("2024-09-10 24:00:00").is_err());
        assert!(Timestamp::parse("yesterday").is_err());
    }

    #[test]
    fn formats_people_actually_write() {
        // no zero padding: accepted, and means the same moment
        assert_eq!(
            Timestamp::parse("2024-9-1 9:05:00"),
            Timestamp::parse("2024-09-01 09:05:00")
        );
        // ISO 8601 with a T, and a time zone suffix: rejected, not misread
        assert!(Timestamp::parse("2024-09-10T12:00:00").is_err());
        assert!(Timestamp::parse("2024-09-10 12:00:00Z").is_err());
        assert!(Timestamp::parse("2024-09-10 12:00:00+02:00").is_err());
        // absurd years are rejected instead of overflowing
        assert!(Timestamp::parse("999999999999-01-01 00:00:00").is_err());
        assert!(Timestamp::parse("0-01-01 00:00:00").is_err());
    }

    #[test]
    fn nan_values_are_rejected() {
        assert!(parse_readings("h\n2024-09-10 12:00:00,A12,NaN\n").is_err());
        assert!(parse_readings("h\n2024-09-10 12:00:00,A12,inf\n").is_err());
    }

    #[test]
    fn window_boundaries_and_equal_timestamps() {
        let at = |t: &str| Timestamp::parse(t).unwrap();
        // exactly on a boundary: belongs to the window that STARTS there
        assert_eq!(
            window_start(at("2024-09-11 00:15:00"), 15 * MINUTE),
            at("2024-09-11 00:15:00")
        );
        assert_eq!(
            window_start(at("2024-09-11 00:14:59"), 15 * MINUTE),
            at("2024-09-11 00:00:00")
        );
        // two readings at the same moment are not a gap
        let same: Vec<Reading> = (0..2)
            .map(|_| Reading {
                at: at("2024-09-11 00:00:00"),
                celsius: 1.0,
            })
            .collect();
        assert!(find_gaps(&same, 5 * MINUTE).is_empty());
        // and too few readings for any gap at all
        assert!(find_gaps(&same[..1], 5 * MINUTE).is_empty());
        assert!(find_gaps(&[], 5 * MINUTE).is_empty());
    }

    #[test]
    fn display_round_trips() {
        for text in [
            "1970-01-01 00:00:00",
            "2024-02-29 23:59:59",
            "1969-12-31 23:59:59",
            "2400-12-31 12:34:56",
        ] {
            assert_eq!(Timestamp::parse(text).unwrap().to_string(), text);
        }
    }

    #[test]
    fn windows_and_gaps() {
        let at = |t: &str| Timestamp::parse(t).unwrap();
        assert_eq!(
            window_start(at("2024-09-11 00:07:30"), 15 * MINUTE),
            at("2024-09-11 00:00:00")
        );
        let readings: Vec<Reading> = [
            "2024-09-11 00:00:00",
            "2024-09-11 00:05:00",
            "2024-09-11 00:30:00",
        ]
        .iter()
        .map(|t| Reading {
            at: at(t),
            celsius: 20.0,
        })
        .collect();
        assert_eq!(
            find_gaps(&readings, 5 * MINUTE),
            [(at("2024-09-11 00:05:00"), at("2024-09-11 00:30:00"), 4)]
        );
    }

    #[test]
    fn the_data_has_one_gap_of_six_readings() {
        let mut readings = parse_readings(include_str!("../data/readings.csv")).unwrap();
        readings.sort_by_key(|r| r.at);
        let gaps = find_gaps(&readings, 5 * MINUTE);
        assert_eq!(gaps.len(), 1);
        assert_eq!(gaps[0].2, 6);
    }
    #[test]
    fn negative_clock_fields_are_not_normalized_to_another_date() {
        for time in [
            "2025-01-01 -1:00:00",
            "2025-01-01 00:-1:00",
            "2025-01-01 00:00:-1",
            "2025-01-01 -9223372036854775808:00:00",
            "2025-01-01 24:00:00",
        ] {
            assert!(Timestamp::parse(time).is_err());
        }
    }
}
