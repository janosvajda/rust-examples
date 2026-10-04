<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: CSV by hand

## The idea in one sentence

CSV looks like "split each line at the commas", but a field can contain commas and quotes, so reading it correctly takes a little more care, and a missing file or a bad row must be an error, never silently empty data.

## The data

`data/sensor_readings.csv` holds readings from sensors in four labs:

```text
Timestamp,SensorID,Reading,Unit,Location,Experiment
2024-09-10 12:00,A12,23.5,Celsius,Lab 1,Temperature Monitoring
2024-09-10 12:05,B22,760,mmHg,Lab 2,Pressure Measurement
…
2024-09-10 12:30,D44,21.8,Celsius,"Lab 4, cold room",Temperature Monitoring
2024-09-10 12:35,D44,22.0,Celsius,"Lab 4, cold room","Temperature Monitoring, ""backup"" sensor"
```

The first line is a **header** with the column names. The last two rows are the interesting ones.

## 1. Why `split(',')` isn't enough

The location `Lab 4, cold room` contains a comma. CSV handles that by putting the field in **double quotes**. A naive split doesn't know about quotes:

```text
split(','):    7 fields: [… "Celsius", "\"Lab 4", " cold room\"", "Temperature Monitoring"]
split_fields:  6 fields: [… "Celsius", "Lab 4, cold room", "Temperature Monitoring"]
```

Seven fields instead of six: every column after the location is shifted, and the data is silently wrong.

The quoting rules are short:

| Written in the file | Field value |
|---|---|
| `Lab 1` | `Lab 1` |
| `"Lab 4, cold room"` | `Lab 4, cold room`: commas inside quotes belong to the field |
| `"say ""hi"""` | `say "hi"`: inside quotes, `""` means one `"` |

`split_fields` follows them, one character at a time, remembering whether it's inside quotes:

```rust
match (c, in_quotes) {
    ('"', false) if field.is_empty() => in_quotes = true,          // a quoted field starts
    ('"', true) if chars.peek() == Some(&'"') => { field.push('"'); chars.next(); }   // "" → "
    ('"', true) => in_quotes = false,                               // the quoted part ends
    (',', false) => fields.push(std::mem::take(&mut field)),       // end of a field
    _ => field.push(c),
}
```

A small **state machine**: the meaning of a character depends on the state (`in_quotes`) the parser is in.

## 2. From fields to a struct

```rust
let [timestamp, sensor_id, value, unit, location, experiment] = fields else {
    return Err(format!("expected 6 fields, found {}", fields.len()));
};
let value = value.trim().parse::<f64>().map_err(|_| format!("`{value}` is not a number"))?;
```

A **slice pattern** with `let … else` checks the number of fields and names them in one step. Then each field gets its real type: the reading becomes an `f64`. Working with a `Reading` struct instead of a list of strings means the compiler knows what every field is.

## 3. Errors: never silently empty

The original version of this example had a quiet bug. It opened the file like this:

```rust
if let Ok(file) = File::open(file_path) { … }   // a missing file → nothing happens
```

And it looked for the file in the wrong folder. So `cargo run` printed **nothing at all** and exited successfully. No error, no data, nothing to tell you why.

Now every problem is an error that says what and where:

```text
can't open no/such/file.csv: No such file or directory (os error 2)
line 5: `warm` is not a number
```

The file path is built from `env!("CARGO_MANIFEST_DIR")`, the crate's own folder, so it works no matter which folder you run `cargo run` from. (The [error handling course](../../error-handling/) explains `?`, `map_err` and `Box<dyn Error>`.)

## 4. A pipeline: filter, then transform

```rust
readings
    .iter()
    .filter(|r| r.unit == "Celsius")
    .map(|r| Reading { value: r.value * 1.8 + 32.0, unit: String::from("Fahrenheit"), ..r.clone() })
    .collect()
```

```text
2024-09-10 12:00 A12  74.3 °F (Lab 1)
2024-09-10 12:30 D44  71.2 °F (Lab 4, cold room)
```

Every data processing job has this shape: **read, keep what you need, change it, output it**. The rest of the course builds on it.

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| a comma inside a field | handled with quotes: `"Lab 4, cold room"` | `quoted_fields_may_contain_commas_and_quotes` |
| a quote inside a field | written as `""`, read as `"` | same |
| an empty field (`a,,c`) | an empty string, not a missing column | `plain_fields` |
| a quote that's never closed | an error, not a silently merged field | `unclosed_quote_is_an_error` |
| a line break **inside** a quoted field | **not supported** by this parser: a clear error ("never closed"), never wrong data. The `csv` crate supports it | `edge_cases_of_whole_files` |
| too few or too many fields | an error naming the line | `rows_become_readings` |
| a value that isn't a number | an error naming the value and the line | same |
| `NaN`, `inf`, `1e400` | rejected. They all parse successfully as `f64` (`1e400` becomes infinity), so `is_finite()` checks them | `nan_and_infinity_are_rejected` |
| Windows line endings (`\r\n`) | handled: `lines()` removes the `\r` too | `edge_cases_of_whole_files` |
| no newline after the last line | the last line is still read | same |
| an empty file, or only a header | an empty list: no rows is a valid answer | same |
| bytes that aren't UTF-8 text | an error that names the line | same |
| a missing file | an error with the path, never an empty list | `missing_file_is_an_error_not_an_empty_list` |
| run from another folder | works: the path comes from `CARGO_MANIFEST_DIR` | `the_data_file_reads_completely` |

Two choices here are deliberate. This lesson **stops at the first bad row**, which is right for small files you control. [Lesson 3](../03-cleaning-and-validation/) shows the alternative for big, messy files: keep going and collect every problem. And a byte-order mark (BOM, the invisible `\u{feff}` some Windows tools put at the start of a file) only affects the header line here, which is skipped. A program that reads header names should strip it.

## When to use a crate instead

Hand-written parsing is great for learning and for simple, known files. Real-world CSV has more corners: different separators, line breaks inside quoted fields, different text encodings, huge files. For those, the `csv` crate is the standard choice. It handles all of that and works with serde (lesson 5) to read rows straight into structs.

## Run it

```bash
cargo run
cargo test
```

Next: [Lesson 2: Big files: streaming](../02-streaming-big-files/) · Back to the [course overview](../)
