// Lesson 4: aggregation and statistics.
//
// Aggregating means turning many rows into a few numbers that answer a
// question: "what's the typical temperature in each lab?". The tools are
// grouping (a map from key to values) and statistics (count, mean, median,
// percentiles). And one warning: the average isn't always the typical value.

use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
struct Reading<'a> {
    timestamp: &'a str,
    lab: &'a str,
    celsius: f64,
}

/// Fixed-width Gregorian calendar date and a 24-hour clock, without a time zone.
fn valid_timestamp(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 16
        || !bytes.iter().enumerate().all(|(i, &b)| match i {
            4 | 7 => b == b'-',
            10 => b == b' ',
            13 => b == b':',
            _ => b.is_ascii_digit(),
        })
    {
        return false;
    }
    // The format check guarantees that every slice below contains ASCII digits.
    let number = |range: std::ops::Range<usize>| text[range].parse::<u32>().unwrap();
    let (year, month, day, hour, minute) = (
        number(0..4),
        number(5..7),
        number(8..10),
        number(11..13),
        number(14..16),
    );
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => return false,
    };
    year > 0 && (1..=days).contains(&day) && hour < 24 && minute < 60
}

/// The rows borrow from the text: no String is allocated per field.
/// Returns the readings and how many lines couldn't be used (never silently dropped).
fn parse(text: &str) -> (Vec<Reading<'_>>, usize) {
    let mut readings = Vec::new();
    let mut bad = 0;
    for line in text.lines().skip(1).filter(|l| !l.trim().is_empty()) {
        let mut fields = line.split(',');
        let reading = (|| {
            let (timestamp, lab, value) = (fields.next()?, fields.next()?, fields.next()?);
            // "NaN" parses as f64, and one NaN would turn every mean and sum into NaN.
            if !valid_timestamp(timestamp) || lab.trim().is_empty() || fields.next().is_some() {
                return None;
            }
            let celsius = value.trim().parse::<f64>().ok().filter(|v| v.is_finite())?;
            Some(Reading {
                timestamp,
                lab,
                celsius,
            })
        })();
        match reading {
            Some(r) => readings.push(r),
            None => bad += 1,
        }
    }
    (readings, bad)
}

// ---- 1. Group by a key -----------------------------------------------------------------------

/// "Group by": one list of values per key. A BTreeMap keeps the keys sorted.
fn group_by<'a>(
    readings: &[Reading<'a>],
    key: impl Fn(&Reading<'a>) -> &'a str,
) -> BTreeMap<&'a str, Vec<f64>> {
    let mut groups: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
    for reading in readings {
        groups
            .entry(key(reading))
            .or_default()
            .push(reading.celsius);
    }
    groups
}

// ---- 2. Statistics ---------------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
struct Summary {
    count: usize,
    min: f64,
    max: f64,
    mean: f64,
    median: f64,
    p90: f64,
}

/// The average: the sum divided by the count.
fn mean(values: &[f64]) -> f64 {
    values.iter().sum::<f64>() / values.len() as f64
}

/// Returns None for an empty group, which has no minimum, mean or median.
fn summarize(values: &[f64]) -> Option<Summary> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    // f64 isn't `Ord` (NaN can't be ordered), so `sort()` doesn't compile.
    // `total_cmp` defines a complete order for every f64, NaN included.
    sorted.sort_by(f64::total_cmp);
    let count = sorted.len();
    Some(Summary {
        count,
        min: sorted[0],
        max: sorted[count - 1],
        mean: mean(&sorted),
        median: median_of_sorted(&sorted),
        p90: percentile_of_sorted(&sorted, 90.0),
    })
}

/// The middle value; with an even count, the average of the two middle ones.
fn median_of_sorted(sorted: &[f64]) -> f64 {
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 {
        sorted[mid]
    } else {
        (sorted[mid - 1] + sorted[mid]) / 2.0
    }
}

/// Nearest-rank percentile: the smallest value that at least `p`% of the values
/// are less than or equal to. p90 = "90% of readings were at most this".
fn percentile_of_sorted(sorted: &[f64], p: f64) -> f64 {
    let rank = (p / 100.0 * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

// ---- 3. A moving average: smoothing a series ----------------------------------------------

/// Each output is the average of 3 neighbouring readings. `array_windows`
/// hands out the windows as `[a, b, c]` arrays.
fn moving_average_3(values: &[f64]) -> Vec<f64> {
    values
        .array_windows::<3>()
        .map(|window| mean(window))
        .collect()
}

fn main() {
    let (readings, bad) = parse(include_str!("../data/temperatures.csv"));
    println!(
        "{} readings from {} labs, {bad} unusable lines\n",
        readings.len(),
        group_by(&readings, |r| r.lab).len()
    );

    println!("1. Per lab: group by, then summarise");
    println!("    lab     count   min    max    mean  median   p90");
    for (lab, values) in group_by(&readings, |r| r.lab) {
        let s = summarize(&values).expect("every group has values");
        println!(
            "    {lab:<6} {:>5} {:>6.1} {:>6.1} {:>6.1} {:>6.1} {:>6.1}",
            s.count, s.min, s.max, s.mean, s.median, s.p90
        );
    }

    println!("\n2. Why the mean can mislead: Lab 2 had one heater fault (95.0 °C)");
    let lab2 = &group_by(&readings, |r| r.lab)["Lab 2"];
    let s = summarize(lab2).expect("Lab 2 has readings");
    println!(
        "    mean   {:.1} °C ← pulled up by a single reading",
        s.mean
    );
    println!(
        "    median {:.1} °C ← the typical value, barely affected",
        s.median
    );

    println!("\n3. Group by a computed key: the hour (all labs together)");
    for (hour, values) in group_by(&readings, |r| &r.timestamp[..13]) {
        let s = summarize(&values).expect("every hour has values");
        println!(
            "    {hour}:00  {:>2} readings, median {:.1} °C",
            s.count, s.median
        );
    }

    println!("\n4. Moving average over 3 readings smooths Lab 1's series");
    let lab1 = &group_by(&readings, |r| r.lab)["Lab 1"];
    let rounded = |v: &[f64]| {
        v.iter()
            .map(|x| format!("{x:.1}"))
            .collect::<Vec<_>>()
            .join(" ")
    };
    println!("    raw:      {}", rounded(lab1));
    println!("    smoothed: {}", rounded(&moving_average_3(lab1)));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn median_odd_and_even() {
        assert_eq!(median_of_sorted(&[1.0, 2.0, 9.0]), 2.0);
        assert_eq!(median_of_sorted(&[1.0, 2.0, 3.0, 9.0]), 2.5);
    }

    #[test]
    fn percentile_nearest_rank() {
        let values: Vec<f64> = (1..=10).map(f64::from).collect();
        assert_eq!(percentile_of_sorted(&values, 90.0), 9.0);
        assert_eq!(percentile_of_sorted(&values, 100.0), 10.0);
        assert_eq!(percentile_of_sorted(&values, 0.0), 1.0);
    }

    #[test]
    fn summary_of_nothing_is_none() {
        assert_eq!(summarize(&[]), None);
    }

    #[test]
    fn one_value_and_identical_values() {
        let one = summarize(&[21.5]).unwrap();
        assert_eq!(
            (one.min, one.max, one.mean, one.median, one.p90),
            (21.5, 21.5, 21.5, 21.5, 21.5)
        );
        let same = summarize(&[7.0; 100]).unwrap();
        assert_eq!((same.median, same.p90), (7.0, 7.0));
    }

    #[test]
    fn unusable_lines_are_counted_not_used() {
        let (readings, bad) = parse(
            "h\n2025-01-01 12:00,A,20.0\n2025-01-01 12:00,A,NaN\n2025-01-01 12:00,A,inf\n2025-01-01 12:00,A\n2025-01-01 12:00,A,warm\n\n2025-01-01 12:00,A,22.0\n",
        );
        assert_eq!(readings.len(), 2);
        assert_eq!(bad, 4); // NaN, inf, a missing value, a word; the empty line is just skipped
        assert_eq!(
            summarize(&readings.iter().map(|r| r.celsius).collect::<Vec<_>>())
                .unwrap()
                .mean,
            21.0
        );
    }

    #[test]
    fn unsorted_input_gives_the_same_statistics() {
        let sorted = summarize(&[1.0, 2.0, 3.0, 4.0, 100.0]).unwrap();
        let shuffled = summarize(&[100.0, 3.0, 1.0, 4.0, 2.0]).unwrap();
        assert_eq!(sorted, shuffled);
    }

    #[test]
    fn an_outlier_moves_the_mean_not_the_median() {
        let normal = summarize(&[20.0, 21.0, 22.0, 23.0, 24.0]).unwrap();
        let with_outlier = summarize(&[20.0, 21.0, 22.0, 23.0, 240.0]).unwrap();
        assert_eq!(normal.median, with_outlier.median);
        assert!(with_outlier.mean > 2.0 * normal.mean);
    }

    #[test]
    fn group_by_collects_per_key() {
        let (readings, _) =
            parse("h\n2025-01-01 12:00,A,1.0\n2025-01-01 12:00,B,5.0\n2025-01-01 12:00,A,3.0\n");
        let groups = group_by(&readings, |r| r.lab);
        assert_eq!(groups["A"], [1.0, 3.0]);
        assert_eq!(groups.keys().copied().collect::<Vec<_>>(), ["A", "B"]); // sorted
    }

    #[test]
    fn moving_average_has_two_fewer_values() {
        assert_eq!(moving_average_3(&[3.0, 6.0, 9.0, 12.0]), [6.0, 9.0]);
        assert!(moving_average_3(&[1.0, 2.0]).is_empty());
    }
    #[test]
    fn malformed_timestamps_are_counted_before_byte_slicing() {
        let (readings, rejected) = parse("h\nt,A,20\nééééééé,A,20\n2025-01-01 12:00,A,20,extra\n");
        assert!(readings.is_empty());
        assert_eq!(rejected, 3);
    }

    #[test]
    fn calendar_and_clock_ranges_are_validated() {
        assert!(valid_timestamp("2024-02-29 23:59"));
        for text in [
            "2023-02-29 12:00",
            "2025-04-31 12:00",
            "2025-01-01 24:00",
            "2025-01-01 23:60",
            "0000-01-01 12:00",
        ] {
            assert!(!valid_timestamp(text));
        }
    }
}
