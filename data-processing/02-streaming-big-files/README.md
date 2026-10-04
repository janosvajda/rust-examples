<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Big files: streaming

## The idea in one sentence

Don't load a big file into memory: read one line at a time, update a small summary, and forget the line, so the memory you need stays the same whether the file has a thousand lines or a billion.

## Two ways to read a file

```rust
let text = fs::read_to_string(path)?;        // the WHOLE file in memory at once
```

```rust
let mut buf = Vec::new();
while let Some(complete) = next_line(&mut input, &mut buf)? {   // ONE line in memory at a time
    …
}
```

The first is perfect for a config file. For a server log that grows by gigabytes a day, it needs as much memory as the file is large, and fails when the file is bigger than your RAM. The second is called **streaming**: memory use doesn't depend on the file size at all.

## The program

`cargo run` generates a sensor log of 2,000,000 lines, about 60 MB, in your temporary folder:

```text
timestamp,sensor,value
2024-09-10T00:00:00,E55,23.52
2024-09-10T00:00:01,B22,18.38
…
```

It then streams the log and keeps only a summary per sensor: count, minimum, maximum and sum.

```text
    sensor     count     min     max    mean
    A12       399496   15.00   30.00   22.50
    B22       400272   15.00   30.00   22.50
    …
```

On an Apple M2, with `cargo run --release`, streaming the 60 MB took about 0.13 seconds. The **whole program's** peak memory was about **1.7 MB**: less than 3% of the file's size. With 600 MB or 6 GB of data, it would still be about the same.

## What makes it streaming

### 1. Keep a summary, not the data

```rust
struct SensorStats { count: u64, min: f64, max: f64, sum: f64 }
```

Four numbers per sensor, updated with each line. The mean is computed at the end: `sum / count`. Any question you can answer by **updating a running result** works this way: counts, sums, minimum, maximum, mean, "the last value seen".

Some questions **can't** be streamed this simply. The **median** needs all the values, sorted ([lesson 4](../04-aggregation-and-statistics/)). For those, you either keep the data, or use approximate algorithms built for streams.

### 2. Reuse one buffer, and read bytes, not text

```rust
fn next_line(input: &mut impl BufRead, buf: &mut Vec<u8>) -> io::Result<Option<bool>> {
    buf.clear();
    if Read::take(&mut *input, MAX_LINE_BYTES as u64 + 1).read_until(b'\n', buf)? == 0 {
        return Ok(None);                                  // the end of the input
    }
    …
}
```

The same `Vec<u8>` is cleared and reused for every line, so the memory is allocated once. Three details make it robust:

- **Bytes first, then text.** The line is read as raw bytes and turned into text with `std::str::from_utf8` afterwards. The obvious `for line in input.lines()` returns an **error** for a line that isn't valid UTF-8, and with `line?` that error ends the whole run, possibly hours into a big file, because of one corrupt byte. Here it's just one bad line.
- **A maximum line length.** `read_line` keeps growing its buffer until it finds a newline. A broken file with gigabytes and no newline would fill the memory. `take(MAX_LINE_BYTES + 1)` stops reading at 1,025 bytes, and the rest of the over-long line is skipped in chunks without being stored.
- **Bad lines are counted, not just skipped.** The summary ends with `bad lines: 0`. If that number is large, something upstream is broken, and you'd want to know.

### 3. Buffer the reading and writing

`BufReader` reads the file in large blocks (8 KB by default) and hands out lines from memory. `BufWriter` does the same for writing. Without them, every small read or write would be a separate request to the operating system, which is far slower. Wrapping a `File` in a `BufReader` or `BufWriter` is almost always right.

### 4. Accept any input: `impl BufRead`

```rust
fn summarize(mut input: impl BufRead) -> io::Result<BTreeMap<String, SensorStats>>
```

`summarize` doesn't open a file itself. It takes **anything that can be read line by line**:

| Source | How to pass it |
|---|---|
| a file | `BufReader::new(File::open(path)?)` |
| standard input (`program < data.csv`) | `std::io::stdin().lock()` |
| text in a test | `"a,b,c\n".as_bytes()` (`&[u8]` implements `BufRead`) |

That makes the function easy to test with a few lines of text, while the real program streams gigabytes. It's the same idea as the generic functions in the [traits course](../../traits-and-generics/02-generics/).

### 5. Results come out sorted

The summary is a `BTreeMap`, not a `HashMap`, so the sensors print in alphabetical order. A `HashMap` would print them in a different order on every run.

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| a file far bigger than memory | streamed: about 1.7 MB of memory for 60 MB, the same for 60 GB | (the demo) |
| a malformed line | counted in `bad_lines`, the run continues | `counts_bad_lines_and_handles_a_missing_final_newline` |
| `NaN`, `inf` as a value | counted as bad: they parse as `f64`, so `is_finite()` checks them | same |
| a line that isn't valid UTF-8 | counted as bad, the run continues | `invalid_utf8_and_windows_line_endings` |
| Windows line endings (`\r\n`) | handled: `trim_end()` removes the `\r` | same |
| one "line" of 50 MB with no newline | skipped in chunks, never stored; counted as bad | `a_gigantic_line_is_skipped_without_storing_it` |
| no newline at the end of the file | the last line is still read | `counts_bad_lines_and_handles_a_missing_final_newline` |
| an empty file, or only a header | an empty summary, not an error | `empty_input_gives_an_empty_summary` |
| a new sensor appears halfway through | gets its own entry: the map grows with the number of **sensors**, never with the number of lines | `summarizes_per_sensor` |

The last row is the limit of "constant memory": if every line had a **different** key, for example a unique id, the map itself would grow with the file. Streaming keeps memory flat only when the summary has a fixed number of entries.

## Run it

```bash
cargo run --release     # optimised: much faster for millions of lines
cargo test
```

A debug build (`cargo run`) works too, just a lot slower.

Previous: [Lesson 1: CSV by hand](../01-csv-by-hand/) · Next: [Lesson 3: Cleaning and validation](../03-cleaning-and-validation/)
