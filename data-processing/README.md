<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Data processing

Reading, cleaning, summarising and reporting on data is one of the most common jobs a program does, and Rust is very good at it: fast, memory-efficient, and strict about types, so messy data can't quietly turn into wrong results.

This course follows one kind of data all the way through: **readings from lab sensors**. Each lesson adds one skill: from parsing a file, through cleaning and statistics, to a pipeline on every CPU core, live data from the network, combining tables and working with time.

## The lessons

| # | Lesson | You'll learn | Crates |
|---|---|---|---|
| 1 | [CSV by hand](01-csv-by-hand/) | parsing lines into structs, quoted fields, why `split(',')` isn't enough, failing loudly instead of silently | none |
| 2 | [Big files: streaming](02-streaming-big-files/) | processing a 60 MB file in about 1.7 MB of memory: `BufRead`, reusing a buffer, `impl BufRead` for testability | none |
| 3 | [Cleaning and validation](03-cleaning-and-validation/) | normalising units and spellings, "no value" markers, impossible values, duplicates, accounting for every row | none |
| 4 | [Aggregation and statistics](04-aggregation-and-statistics/) | group by, mean vs median, percentiles, sorting floats, moving averages | none |
| 5 | [JSON with serde](05-json-with-serde/) | typed JSON in and out, optional fields and defaults, enums as validation, precise errors, JSON Lines | `serde`, `serde_json` |
| 6 | [A processing pipeline](06-processing-pipeline/) | small testable stages, fold and merge, running on every core with rayon, why parallel float sums differ | `rayon` |
| 7 | [Live data over the network](07-live-data-over-the-network/) | a TCP client and server that survive everything: timeouts, retries with backoff, resuming, duplicates (idempotency), losses, messy data, authentication, authorization, bounded memory | none |
| 8 | [Joining datasets](08-joining-datasets/) | combining tables by a key, inner / left / anti joins, the rows that don't match, hash join vs nested loops | none |
| 9 | [Time and windows](09-time-and-windows/) | timestamps as numbers, leap years, out-of-order data, finding gaps, tumbling windows, why time zones need a crate | none |

Lessons 5 and 6 use the two crates almost every Rust data project relies on, on purpose. All the others use only the standard library.

## The ideas that run through the course

- **Never lose data silently.** A missing file, a bad row, a fixed value: each is reported, counted or rejected with a reason.
- **Types are validation.** Once text has become a `Unit` enum or an `f64`, the rest of the program can trust it.
- **Keep a summary, not the data**, whenever the question allows it.
- **Small stages** are easy to test, change and parallelise.
- **Plan for the edge cases.** Every lesson ends with an **Edge cases** table listing what can go wrong in real data, what the code does about it, and the test that proves it. A few traps come up again and again:
  - `"NaN".parse::<f64>()` **succeeds**;
  - one invalid UTF-8 byte stops a `for line in input.lines()` loop that uses `line?`;
  - a line with no newline can fill the memory;
  - `collect()` into a `HashMap` silently keeps only the last of two equal keys.

## The course in one table

| Lesson | Idea |
|---|---|
| 1. CSV by hand | parse carefully (quotes!), and fail loudly, never silently |
| 2. Streaming | keep a summary, not the data: memory stays flat |
| 3. Cleaning | normalise, validate, deduplicate, and account for every row |
| 4. Statistics | group by a key; the median is often more honest than the mean |
| 5. JSON with serde | describe the shape once; the struct is the validation |
| 6. A pipeline | small stages, fold and merge, and the same code on every core |
| 7. Live data | process each piece as it arrives; bad input and disconnects are normal |
| 8. Joins | an index, not nested loops; the unmatched rows are the interesting ones |
| 9. Time | timestamps as numbers; look for the data that's missing |

## Where to go next

These tools are worth knowing about. They're left out of the course because each is a large subject, or a large dependency, of its own:

| Need | Look at |
|---|---|
| CSV with every edge case (different separators, line breaks in fields, encodings) | the `csv` crate, which works with serde |
| a database: queries, indexes and joins done for you | SQLite with `rusqlite`, or `sqlx` for PostgreSQL and MySQL |
| very large tables, analysed column by column | Apache Arrow and Parquet files: the `arrow` and `parquet` crates |
| DataFrames, like pandas in Python | `polars`: powerful, but a very large dependency (see [Fewer dependencies](../software-engineering-with-ai/08-fewer-dependencies/)) |
| time zones and calendars | `jiff` or `chrono` |
| compressed files (`.gz`) | `flate2`: its decoder is a `Read`, so wrap it in a `BufReader` and stream it like [lesson 2](02-streaming-big-files/) |

## Run a lesson

```bash
cd data-processing/04-aggregation-and-statistics
cargo run
cargo test
```

Lessons 2, 6 and 8 process millions of lines or comparisons, so run them with `cargo run --release`.
