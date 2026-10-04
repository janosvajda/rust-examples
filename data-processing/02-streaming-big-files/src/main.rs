// Lesson 2: big files, streaming.
//
// `fs::read_to_string` loads a whole file into memory. That's fine for a
// config file, and impossible for a 50 GB log. Streaming reads ONE line at a
// time, updates a small summary, and forgets the line. Memory stays the same
// whether the file has a thousand lines or a billion.

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::time::Instant;

/// Everything we remember about one sensor: a few numbers, never the lines.
#[derive(Debug, Clone, Copy, PartialEq)]
struct SensorStats {
    count: u64,
    min: f64,
    max: f64,
    sum: f64,
}

impl SensorStats {
    fn new(value: f64) -> Self {
        SensorStats { count: 1, min: value, max: value, sum: value }
    }

    fn add(&mut self, value: f64) {
        self.count += 1;
        self.min = self.min.min(value);
        self.max = self.max.max(value);
        self.sum += value;
    }

    fn mean(&self) -> f64 {
        self.sum / self.count as f64
    }
}

// ---- 1. Make a big file to work with ----------------------------------------------------

/// Writes `lines` readings like `2024-09-10T12:00:07,A12,23.41`. A tiny
/// pseudo-random generator keeps the data the same on every run, with no crate.
fn generate_log(path: &Path, lines: u64) -> io::Result<()> {
    let sensors = ["A12", "B22", "C33", "D44", "E55"];
    let mut out = BufWriter::new(File::create(path)?); // buffers writes: far fewer system calls
    let mut seed: u64 = 42;
    writeln!(out, "timestamp,sensor,value")?;
    for i in 0..lines {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let sensor = sensors[(seed >> 33) as usize % sensors.len()];
        let value = 15.0 + (seed >> 40) as f64 / (1u64 << 24) as f64 * 15.0; // 15.0 to 30.0
        writeln!(out, "2024-09-10T{:02}:{:02}:{:02},{sensor},{value:.2}", i / 3600 % 24, i / 60 % 60, i % 60)?;
    }
    out.flush()
}

// ---- 2. Stream it: one line at a time ------------------------------------------------------

/// No real line in this file is anywhere near this long. A "line" without a
/// newline for gigabytes (a corrupt or wrong file) must not fill the memory.
const MAX_LINE_BYTES: usize = 1024;

/// The result: statistics per sensor, and how many lines couldn't be used.
#[derive(Debug, Default, PartialEq)]
struct Summary {
    sensors: BTreeMap<String, SensorStats>,
    bad_lines: u64,
}

/// Reads the next line as raw bytes into `buf`, at most `MAX_LINE_BYTES` of it.
/// Returns `None` at the end of the input, and `Some(false)` for a line that was
/// too long (skipped in chunks, never stored whole).
fn next_line(input: &mut impl BufRead, buf: &mut Vec<u8>) -> io::Result<Option<bool>> {
    buf.clear();
    if Read::take(&mut *input, MAX_LINE_BYTES as u64 + 1).read_until(b'\n', buf)? == 0 {
        return Ok(None);
    }
    if buf.ends_with(b"\n") || buf.len() <= MAX_LINE_BYTES {
        return Ok(Some(true));
    }
    loop {
        // too long: throw away the rest of this line, a chunk at a time
        let chunk = input.fill_buf()?;
        let Some(end) = chunk.iter().position(|&b| b == b'\n') else {
            let len = chunk.len();
            input.consume(len);
            if len == 0 {
                return Ok(Some(false));
            }
            continue;
        };
        input.consume(end + 1);
        return Ok(Some(false));
    }
}

/// Summarises any `BufRead` source: a file, standard input, or bytes in a test.
/// One buffer is reused for every line, so memory doesn't grow with the file.
fn summarize(mut input: impl BufRead) -> io::Result<Summary> {
    let mut summary = Summary::default();
    let mut buf = Vec::with_capacity(MAX_LINE_BYTES + 1);
    next_line(&mut input, &mut buf)?; // skip the header
    while let Some(complete) = next_line(&mut input, &mut buf)? {
        // Raw bytes, then text: one line of invalid UTF-8 is a bad line, not the
        // end of the whole run (`BufRead::lines()` would return an error here).
        let parsed = std::str::from_utf8(&buf).ok().filter(|_| complete).and_then(|line| {
            let mut fields = line.trim_end().split(',');
            let (_time, sensor, value) = (fields.next()?, fields.next()?, fields.next()?);
            let value = value.parse::<f64>().ok().filter(|v| v.is_finite())?; // "NaN" parses!
            Some((sensor, value))
        });
        let Some((sensor, value)) = parsed else {
            summary.bad_lines += 1; // counted, never silently dropped
            continue;
        };
        match summary.sensors.get_mut(sensor) {
            Some(s) => s.add(value),
            None => {
                summary.sensors.insert(sensor.to_string(), SensorStats::new(value));
            }
        }
    }
    Ok(summary)
}

fn main() -> io::Result<()> {
    let path = std::env::temp_dir().join("rust-examples-sensor-log.csv");
    let lines = 2_000_000;

    println!("1. Generating a log with {lines} lines…");
    generate_log(&path, lines)?;
    let size = std::fs::metadata(&path)?.len();
    println!("    {} ({:.1} MB)", path.display(), size as f64 / 1_000_000.0);

    println!("\n2. Streaming it, one line at a time");
    let start = Instant::now();
    let summary = summarize(BufReader::new(File::open(&path)?))?;
    println!("    took {:.2?}; memory used: one line buffer + {} small summaries", start.elapsed(), summary.sensors.len());
    println!("    sensor     count     min     max    mean");
    for (sensor, s) in &summary.sensors {
        println!("    {sensor:<6} {:>9} {:>7.2} {:>7.2} {:>7.2}", s.count, s.min, s.max, s.mean());
    }

    println!("    bad lines: {}", summary.bad_lines);

    println!("\n3. The same idea works on any input, messy input included");
    let sample = b"timestamp,sensor,value\nt,X1,10.0\nt,X1,20.0\nnot a valid line\nt,X1,NaN\n\xff\xfe\nt,Y2,5.5";
    let summary = summarize(&sample[..])?;
    for (sensor, s) in &summary.sensors {
        println!("    {sensor}: readings {}, mean {}", s.count, s.mean());
    }
    println!("    bad lines: {} (garbage, NaN, invalid UTF-8)", summary.bad_lines);

    std::fs::remove_file(&path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarizes_per_sensor() {
        let input = "timestamp,sensor,value\nt,A,1.0\nt,B,10.0\nt,A,3.0\n";
        let stats = summarize(input.as_bytes()).unwrap().sensors;
        assert_eq!(stats["A"], SensorStats { count: 2, min: 1.0, max: 3.0, sum: 4.0 });
        assert_eq!(stats["A"].mean(), 2.0);
        assert_eq!(stats["B"].count, 1);
    }

    #[test]
    fn counts_bad_lines_and_handles_a_missing_final_newline() {
        let input = "header\ngarbage\nt,A,oops\nt,A,inf\nt,A,2.5";
        let summary = summarize(input.as_bytes()).unwrap();
        assert_eq!(summary.sensors["A"].count, 1);
        assert_eq!(summary.bad_lines, 3);
    }

    #[test]
    fn invalid_utf8_and_windows_line_endings() {
        let input = b"header\r\nt,A,1.0\r\n\xff\xfe\r\nt,A,3.0\r\n";
        let summary = summarize(&input[..]).unwrap();
        assert_eq!(summary.sensors["A"].count, 2); // the run went on past the bad bytes
        assert_eq!(summary.bad_lines, 1);
    }

    #[test]
    fn a_gigantic_line_is_skipped_without_storing_it() {
        let mut input = b"header\nt,A,1.0\n".to_vec();
        input.extend(std::iter::repeat_n(b'x', 50_000_000)); // 50 MB with no newline…
        input.extend(b"\nt,A,3.0\n"); // …then normal lines again
        let summary = summarize(&input[..]).unwrap();
        assert_eq!(summary.sensors["A"].count, 2);
        assert_eq!(summary.bad_lines, 1);
    }

    #[test]
    fn empty_input_gives_an_empty_summary() {
        assert_eq!(summarize("".as_bytes()).unwrap(), Summary::default());
        assert_eq!(summarize("header only\n".as_bytes()).unwrap(), Summary::default());
    }

    #[test]
    fn generated_file_round_trips() {
        let path = std::env::temp_dir().join("rust-examples-stream-test.csv");
        generate_log(&path, 1_000).unwrap();
        let summary = summarize(BufReader::new(File::open(&path).unwrap())).unwrap();
        let stats = summary.sensors;
        assert_eq!(summary.bad_lines, 0);
        assert_eq!(stats.values().map(|s| s.count).sum::<u64>(), 1_000);
        assert!(stats.values().all(|s| s.min >= 15.0 && s.max <= 30.0));
        std::fs::remove_file(&path).unwrap();
    }
}
