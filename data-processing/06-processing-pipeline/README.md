<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: A processing pipeline

## The idea in one sentence

Build a data job as a chain of small, separate stages (read, parse, convert, aggregate, report), and because the middle stages handle each line independently, the whole chain can run on every CPU core by changing `iter` to `par_iter`.

## The pipeline

```text
  read  ─►  parse & validate  ─►  convert units  ─►  aggregate  ─►  write a report
 (lines)      (lesson 1, 3)        (lesson 3)        (lesson 2, 4)     (CSV file)
```

Every stage is a plain function:

| Stage | Function | In → out |
|---|---|---|
| read | `read_lines(path)` | a file → `Vec<String>` |
| parse & validate | `parse(line)` | `&str` → `Option<(Reading, unit)>` |
| convert units | `to_celsius(…)` | a reading in °C or °F → a reading in °C, or `None` |
| aggregate | `Totals::add`, `Totals::merge` | readings → statistics per sensor + a count of rejected lines |
| write a report | `write_report(path, &totals)` | statistics → a CSV file |

Small stages are easy to understand, easy to **test one at a time** (each has its own test), and easy to change. Adding a stage, like "drop readings from sensors under maintenance", means writing one function and adding one step to the chain.

The input is 4,000,000 generated lines with mixed units (every 7th line in Fahrenheit) and a broken sensor that writes `ERR` every 1,000th line:

```text
time,sensor,value,unit
0,G77,72.32,F
1,H88,29.33,C
…
999,G77,ERR,C
```

## Sequential and parallel: one word apart

```rust
fn run_sequential(lines: &[String]) -> Totals {
    lines.iter()
        .map(|line| process(line))
        .fold(Totals::default(), Totals::add)
}

fn run_parallel(lines: &[String]) -> Totals {
    lines.par_iter()                                 // ← rayon: split the work across cores
        .map(|line| process(line))
        .fold(Totals::default, Totals::add)          // each thread builds its own Totals
        .reduce(Totals::default, Totals::merge)      // then they're combined
}
```

On an Apple M2 with 8 cores (`cargo run --release`):

```text
sequential:   183.03ms
parallel:      32.59ms  on 8 cores, 5.6× faster
```

Your numbers will differ. The speedup is less than 8× because some work can't be split, like reading the file and merging the results, and the cores share memory bandwidth.

### What made it parallelisable

- **Each line is processed on its own.** `process(line)` doesn't look at other lines or change anything shared. That's why the lines can be handed out to different threads in any order.
- **Partial results can be merged.** Each thread folds its share into its **own** `Totals`, so no locks are needed while working. At the end, `merge` combines two partial results, with sums added, minimums of minimums, and so on. This shape, **fold then reduce**, is also how big data systems like MapReduce and Spark split work across thousands of machines.

The [concurrency course](../../concurrency/05-data-parallelism/) explains how rayon divides the work.

## A surprise: the sums aren't identical

```text
A12 sum, sequential: 11249965.0288889613
A12 sum, parallel:   11249965.0288888868
```

The same numbers, added up, giving two different results. Floating-point addition is **not associative**: `(a + b) + c` can differ from `a + (b + c)` in the last digits, because each addition rounds. The parallel version adds in a different order, so it rounds differently.

What that means in practice:
- Counts, minimums and maximums are **exact** either way, and the program checks that they're identical.
- Sums and means agree to many digits, but not all. Compare them with a **tolerance**, as the tests do, never with `==`.
- Where exact totals matter, like money, use **integers**: count cents, not euros as `f64`.

## When the data doesn't fit in memory

`read_lines` loads all lines first, so that rayon can share them out. For files bigger than memory, combine this with [lesson 2](../02-streaming-big-files/):
- read a **chunk** of, say, 100,000 lines;
- process the chunk in parallel into a `Totals`;
- `merge` it into the running total;
- repeat.

`merge` makes this easy: it doesn't care whether the partial results came from threads or from chunks.

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| a broken value (`ERR`, a word) | rejected and counted in `rejected` | `aggregate_counts_rejections` |
| `NaN`, `inf`, `1e400` | rejected: `inf` would even pass "≥ -273.15", so `is_finite()` checks first | `nan_infinity_and_broken_text_are_rejected` |
| an unknown unit | rejected, never guessed | `each_stage_on_its_own` |
| a line that isn't valid UTF-8 | read as bytes, damaged characters become `�`, and the line is rejected and counted; the run goes on | `invalid_utf8_lines_are_counted_not_fatal` |
| invalid bytes in a field the pipeline doesn't use | the line is still accepted: damage that doesn't affect the result isn't treated as an error | (the same test, with the time field) |
| Windows line endings | handled | same |
| empty input | empty totals, sequential and parallel alike | `empty_input` |
| results depending on the thread count | counts, min and max are exact; sums agree within a tolerance | `parallel_gives_the_same_answer` |
| partial results from different threads | `merge` combines them, including sensors only one thread saw | `merge_combines_partial_results` |

**Memory:** this lesson loads all lines before processing, so that rayon can split them across cores. That's the price of this simple version: it needs memory for the whole file. For bigger files, use the chunked approach above.

## Run it

```bash
cargo run --release
cargo test
```

Previous: [Lesson 5: JSON with serde](../05-json-with-serde/) · Next: [Lesson 7: Live data over the network](../07-live-data-over-the-network/)
