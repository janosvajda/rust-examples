// Lesson 8: joining datasets.
//
// Data often lives in several tables: readings say WHICH sensor measured,
// another table says WHERE each sensor is. Joining combines them through a
// shared key (the sensor id). The interesting parts are the rows that DON'T
// match, and doing it fast: with a HashMap, not with a loop inside a loop.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::Instant;

#[derive(Debug, Clone, PartialEq)]
struct Sensor<'a> {
    id: &'a str,
    lab: &'a str,
    room: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
struct Reading<'a> {
    time: &'a str,
    sensor: &'a str,
    value: f64,
}

#[derive(Debug, PartialEq)]
struct ParseError {
    line: usize,
    reason: &'static str,
}
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.reason)
    }
}
impl std::error::Error for ParseError {}

fn fields<'a>(text: &'a str, header: &str) -> Result<Vec<[&'a str; 3]>, ParseError> {
    let mut lines = text.lines();
    if lines.next() != Some(header) {
        return Err(ParseError {
            line: 1,
            reason: "unexpected or missing header",
        });
    }
    lines
        .enumerate()
        .map(|(i, line)| {
            let values: Vec<_> = line.split(',').map(str::trim).collect();
            let Ok(values): Result<[&str; 3], _> = values.try_into() else {
                return Err(ParseError {
                    line: i + 2,
                    reason: "expected exactly three fields",
                });
            };
            if values.iter().any(|v| v.is_empty()) {
                return Err(ParseError {
                    line: i + 2,
                    reason: "empty field",
                });
            }
            Ok(values)
        })
        .collect()
}

fn parse_sensors(text: &str) -> Result<Vec<Sensor<'_>>, ParseError> {
    Ok(fields(text, "sensor,lab,room")?
        .into_iter()
        .map(|[id, lab, room]| Sensor { id, lab, room })
        .collect())
}

fn parse_readings(text: &str) -> Result<Vec<Reading<'_>>, ParseError> {
    fields(text, "time,sensor,value")?
        .into_iter()
        .enumerate()
        .map(|(i, [time, sensor, value])| {
            let value = value
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or(ParseError {
                    line: i + 2,
                    reason: "value must be a finite number",
                })?;
            Ok(Reading {
                time,
                sensor,
                value,
            })
        })
        .collect()
}

// ---- 1. The index: look up a sensor by id in one step ---------------------------------------

/// Build once, look up many times. This is what makes a "hash join" fast.
///
/// A key must be unique. `collect()` into a HashMap would silently keep the
/// LAST of two sensors with the same id, and every reading would be joined
/// to a location that may be wrong. So duplicates are an error, listed by id.
fn index_by_id<'s, 'a>(
    sensors: &'s [Sensor<'a>],
) -> Result<HashMap<&'a str, &'s Sensor<'a>>, Vec<&'a str>> {
    let mut index = HashMap::with_capacity(sensors.len());
    let mut duplicates = Vec::new();
    for sensor in sensors {
        if index.insert(sensor.id, sensor).is_some() {
            duplicates.push(sensor.id);
        }
    }
    if duplicates.is_empty() {
        Ok(index)
    } else {
        Err(duplicates)
    }
}

// ---- 2. Kinds of join -------------------------------------------------------------------------------

/// Inner join: only readings whose sensor is known. Unknown sensors are dropped.
fn inner_join<'r, 's, 'a>(
    readings: &'r [Reading<'a>],
    index: &HashMap<&'a str, &'s Sensor<'a>>,
) -> Vec<(&'r Reading<'a>, &'s Sensor<'a>)> {
    readings
        .iter()
        .filter_map(|r| Some((r, *index.get(r.sensor)?)))
        .collect()
}

/// Left join: EVERY reading, with `None` where the sensor is unknown.
fn left_join<'r, 's, 'a>(
    readings: &'r [Reading<'a>],
    index: &HashMap<&'a str, &'s Sensor<'a>>,
) -> Vec<(&'r Reading<'a>, Option<&'s Sensor<'a>>)> {
    readings
        .iter()
        .map(|r| (r, index.get(r.sensor).copied()))
        .collect()
}

/// Anti join, one way: readings from sensors that aren't registered.
fn unknown_sensors<'a>(
    readings: &[Reading<'a>],
    index: &HashMap<&'a str, &Sensor<'a>>,
) -> Vec<&'a str> {
    readings
        .iter()
        .filter(|r| !index.contains_key(r.sensor))
        .map(|r| r.sensor)
        .collect()
}

/// Anti join, the other way: registered sensors that never sent a reading.
fn silent_sensors<'a>(sensors: &[Sensor<'a>], readings: &[Reading<'a>]) -> Vec<&'a str> {
    let reporting: HashSet<&str> = readings.iter().map(|r| r.sensor).collect();
    sensors
        .iter()
        .filter(|s| !reporting.contains(s.id))
        .map(|s| s.id)
        .collect()
}

// ---- 3. After the join: group by a column from the OTHER table ---------------------------

/// Count readings per room: `room` comes from the sensors table, which the
/// readings alone don't know.
fn readings_per_room<'a>(
    joined: &[(&Reading<'a>, &Sensor<'a>)],
) -> BTreeMap<(&'a str, &'a str), usize> {
    let mut counts = BTreeMap::new();
    for (_, sensor) in joined {
        *counts.entry((sensor.lab, sensor.room)).or_insert(0) += 1;
    }
    counts
}

// ---- 4. Why the index matters: a nested-loop join for comparison ---------------------------

/// Joins without an index: for each reading, search the whole sensor list.
/// Returns how many readings matched and how many comparisons it took.
fn nested_loop_join(readings: &[Reading<'_>], sensors: &[Sensor<'_>]) -> (usize, u64) {
    let (mut matched, mut comparisons) = (0, 0);
    for r in readings {
        for s in sensors {
            comparisons += 1;
            if s.id == r.sensor {
                matched += 1;
                break;
            }
        }
    }
    (matched, comparisons)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sensors = parse_sensors(include_str!("../data/sensors.csv"))?;
    let readings = parse_readings(include_str!("../data/readings.csv"))?;
    let index = match index_by_id(&sensors) {
        Ok(index) => index,
        Err(duplicates) => {
            eprintln!("can't join: sensor ids registered more than once: {duplicates:?}");
            std::process::exit(1);
        }
    };
    println!("{} sensors, {} readings\n", sensors.len(), readings.len());

    println!("1. Inner join: readings with a known sensor");
    let joined = inner_join(&readings, &index);
    for (r, s) in &joined {
        println!(
            "    {} {} {:>6}  {} / {}",
            r.time, r.sensor, r.value, s.lab, s.room
        );
    }
    println!(
        "    {} of {} readings matched",
        joined.len(),
        readings.len()
    );

    println!("\n2. Left join: every reading, unknown sensors marked");
    for (r, s) in left_join(&readings, &index)
        .iter()
        .filter(|(_, s)| s.is_none())
    {
        println!(
            "    {} {} {:>6}  location: {:?}",
            r.time,
            r.sensor,
            r.value,
            s.map(|s| s.room)
        );
    }

    println!("\n3. What didn't match (anti joins)");
    println!(
        "    readings from unregistered sensors: {:?}",
        unknown_sensors(&readings, &index)
    );
    println!(
        "    registered sensors with no readings: {:?}",
        silent_sensors(&sensors, &readings)
    );

    println!("\n4. Group by a column from the other table: readings per room");
    for ((lab, room), count) in readings_per_room(&joined) {
        println!("    {lab} / {room}: {count}");
    }

    println!("\n5. Hash join vs nested loops on bigger tables");
    let ids: Vec<String> = (0..2_000).map(|i| format!("S{i:04}")).collect();
    let big_sensors: Vec<Sensor> = ids
        .iter()
        .map(|id| Sensor {
            id,
            lab: "Lab",
            room: "Room",
        })
        .collect();
    let big_readings: Vec<Reading> = (0..200_000)
        .map(|i| Reading {
            time: "t",
            sensor: &ids[i * 7919 % ids.len()],
            value: 1.0,
        })
        .collect();

    let start = Instant::now();
    let (matched, comparisons) = nested_loop_join(&big_readings, &big_sensors);
    println!(
        "    nested loops: {matched} matched, {comparisons} comparisons, {:.2?}",
        start.elapsed()
    );

    let start = Instant::now();
    let big_index = index_by_id(&big_sensors).expect("generated ids are unique");
    let matched = inner_join(&big_readings, &big_index).len();
    println!(
        "    hash join:    {matched} matched, {} lookups, {:.2?}",
        big_readings.len(),
        start.elapsed()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data() -> (Vec<Sensor<'static>>, Vec<Reading<'static>>) {
        (
            parse_sensors(include_str!("../data/sensors.csv")).unwrap(),
            parse_readings(include_str!("../data/readings.csv")).unwrap(),
        )
    }

    #[test]
    fn inner_join_drops_unknown_sensors() {
        let (sensors, readings) = data();
        let joined = inner_join(&readings, &index_by_id(&sensors).unwrap());
        assert_eq!(joined.len(), readings.len() - 1); // X99 is unknown
        assert!(joined.iter().all(|(r, s)| r.sensor == s.id));
    }

    #[test]
    fn left_join_keeps_every_reading() {
        let (sensors, readings) = data();
        let joined = left_join(&readings, &index_by_id(&sensors).unwrap());
        assert_eq!(joined.len(), readings.len());
        assert_eq!(joined.iter().filter(|(_, s)| s.is_none()).count(), 1);
    }

    #[test]
    fn anti_joins_find_both_kinds_of_mismatch() {
        let (sensors, readings) = data();
        assert_eq!(
            unknown_sensors(&readings, &index_by_id(&sensors).unwrap()),
            ["X99"]
        );
        assert_eq!(silent_sensors(&sensors, &readings), ["E55"]);
    }

    #[test]
    fn group_by_a_joined_column() {
        let (sensors, readings) = data();
        let index = index_by_id(&sensors).unwrap();
        let per_room = readings_per_room(&inner_join(&readings, &index));
        assert_eq!(per_room[&("Lab 1", "Microscopy")], 3);
        assert_eq!(per_room[&("Lab 4", "Cold room")], 2);
    }

    #[test]
    fn a_sensor_registered_twice_is_an_error() {
        let sensors = parse_sensors(
            "sensor,lab,room\nA12,Lab 1,Microscopy\nB22,Lab 2,X\nA12,Lab 4,Cold room\n",
        )
        .unwrap();
        assert_eq!(index_by_id(&sensors).unwrap_err(), ["A12"]);
    }

    #[test]
    fn keys_with_stray_spaces_still_match() {
        let sensors = parse_sensors("sensor,lab,room\n A12 ,Lab 1,Microscopy\n").unwrap();
        let readings = parse_readings("time,sensor,value\n12:00,A12 ,20.5\n").unwrap();
        assert_eq!(
            inner_join(&readings, &index_by_id(&sensors).unwrap()).len(),
            1
        );
    }

    #[test]
    fn empty_tables_join_to_nothing() {
        let sensors = parse_sensors("sensor,lab,room\n").unwrap();
        let readings = parse_readings("time,sensor,value\n12:00,A12,20.5\n").unwrap();
        let index = index_by_id(&sensors).unwrap();
        assert!(inner_join(&readings, &index).is_empty());
        assert_eq!(left_join(&readings, &index).len(), 1); // the left join still keeps it
        assert_eq!(unknown_sensors(&readings, &index), ["A12"]);
    }

    #[test]
    fn nested_loops_and_hash_join_agree() {
        let (sensors, readings) = data();
        let (matched, _) = nested_loop_join(&readings, &sensors);
        assert_eq!(
            matched,
            inner_join(&readings, &index_by_id(&sensors).unwrap()).len()
        );
    }
    #[test]
    fn malformed_rows_are_errors_with_line_numbers() {
        for row in ["t,A,NaN", "t,A,inf", "t,A,x", "t,A,20,extra", "t,,20"] {
            assert_eq!(
                parse_readings(&format!("time,sensor,value\n{row}"))
                    .unwrap_err()
                    .line,
                2
            );
        }
        assert!(parse_sensors("sensor,lab,room\nA,Lab").is_err());
        assert!(parse_sensors("wrong header").is_err());
    }
}
