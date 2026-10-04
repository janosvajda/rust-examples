<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Aggregation and statistics

## The idea in one sentence

Aggregating turns many rows into a few numbers that answer a question: you **group** the rows by a key, **summarise** each group with statistics, and choose those statistics carefully, because the average isn't always the typical value.

## The data

`data/temperatures.csv` has 36 readings: three labs, every 15 minutes from 8:00 to 10:45. In Lab 2, a heater fault at 9:15 produced one reading of **95.0 °C**.

## 1. Group by

"What's the temperature in each lab?" starts with putting each reading into its lab's group:

```rust
let mut groups: BTreeMap<&str, Vec<f64>> = BTreeMap::new();
for reading in readings {
    groups.entry(reading.lab).or_default().push(reading.celsius);
}
```

`entry(key).or_default()` gets the group's list, creating an empty one the first time a key appears. That one line is the heart of every "group by", in Rust as in SQL. A `BTreeMap` keeps the keys sorted, so the results print in a stable order.

The key doesn't have to be a column. `group_by` takes a closure, so you can group by anything you can compute from a row, such as the hour of the timestamp:

```rust
group_by(&readings, |r| r.lab)                  // per lab
group_by(&readings, |r| &r.timestamp[..13])     // per hour: "2024-09-10 08"
```

## 2. Summarise each group

```text
    lab     count   min    max    mean  median   p90
    Lab 1     12   21.0   22.6   21.9   22.1   22.5
    Lab 2     12   20.5   95.0   27.2   21.1   21.4
    Lab 3     12   18.0   18.9   18.5   18.6   18.8
```

| Statistic | Meaning | How |
|---|---|---|
| count, min, max | how many, the extremes | one pass, streamable ([lesson 2](../02-streaming-big-files/)) |
| mean | sum ÷ count, the "average" | one pass, streamable |
| **median** | the middle value: half are below, half above | needs **all values, sorted** |
| **p90** (90th percentile) | 90% of the values are at most this | needs all values, sorted |

### Sorting floats

The median and percentiles need sorted values, but this doesn't compile:

```text
error[E0277]: the trait bound `f64: Ord` is not satisfied
```

`sort()` needs a **total order**, where any two values can be compared. Floats have `NaN` ("not a number"), which isn't less than, equal to or greater than anything, so `f64` only implements `PartialOrd` (see the [standard traits lesson](../../traits-and-generics/05-standard-traits/)). The solution is `sort_by(f64::total_cmp)`, which defines a complete order for every float, `NaN` included.

### Median and percentile

```rust
fn median_of_sorted(sorted: &[f64]) -> f64 {
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 1 { sorted[mid] } else { (sorted[mid - 1] + sorted[mid]) / 2.0 }
}
```

With an even number of values there are two middle values, and the median is their average. For percentiles, this lesson uses the simple **nearest-rank** method: p90 of 10 sorted values is the 9th. Other definitions interpolate between values. They give slightly different numbers, which is fine as long as you use one consistently.

## 3. Why the mean can mislead

```text
Lab 2:
    mean   27.2 °C ← pulled up by a single reading
    median 21.1 °C ← the typical value, barely affected
```

One faulty reading out of twelve moves Lab 2's **mean** from 21.0 °C (without it) to 27.2 °C: more than 6 degrees too warm, for a lab that was never that warm. The **median** hardly notices, because it only cares which value is in the middle, not how far away the extreme ones are. The same is true for salaries, house prices and response times: one extreme value drags the mean along.

**Rule of thumb:** report the median for "typical", the mean when you need totals or the extremes matter, and a high percentile (p90, p99) for "how bad does it get". Response times on websites are usually reported as p95 or p99 for that reason.

## 4. Moving average: smoothing a series

Readings over time are noisy. A **moving average** replaces each point with the average of itself and its neighbours:

```rust
values.array_windows().map(|[a, b, c]| (a + b + c) / 3.0).collect()
```

```text
raw:      21.0 21.2 21.1 21.4 21.8 22.0 22.3 22.1 22.4 22.6 22.5 22.2
smoothed: 21.1 21.2 21.4 21.7 22.0 22.1 22.3 22.4 22.5 22.4
```

`array_windows` gives overlapping windows of three as arrays ([iterator toolbox](../../closures-and-iterators/04-iterator-toolbox/)). The smoothed series is two values shorter, because the first and last points don't have neighbours on both sides. The small dips disappear and the warming trend shows clearly. A wider window smooths more, but reacts more slowly to real changes.

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| a group with no values | `summarize` returns `None`: there's no minimum or median of nothing, and dividing by zero would give `NaN` | `summary_of_nothing_is_none` |
| a group with one value | every statistic is that value | `one_value_and_identical_values` |
| all values identical | median and percentiles equal that value | same |
| an even number of values | the median is the average of the two middle ones | `median_odd_and_even` |
| input in any order | sorted first, so the order doesn't matter | `unsorted_input_gives_the_same_statistics` |
| `NaN` or `inf` in the data | counted as unusable, never used. A single `NaN` would make every sum and mean `NaN` | `unusable_lines_are_counted_not_used` |
| a line with a missing or non-numeric value | counted as unusable | same |
| an outlier | visible as the gap between mean and median; kept, because removing data is a decision for a person | `an_outlier_moves_the_mean_not_the_median` |
| percentile at 0% or 100% | clamped to the smallest or largest value | `percentile_nearest_rank` |
| a moving average over fewer values than the window | no output, rather than a misleading average of too few values | `moving_average_has_two_fewer_values` |

Two limits to know about:
- **Sums of millions of floats** slowly lose precision, because every addition rounds. For very long series, use compensated (Kahan) summation, or sum integers, for example in hundredths of a degree.
- **Median and percentiles need all the values in memory.** For data too big for that, use approximate streaming algorithms such as t-digest, or compute exact values per group when each group fits.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: Cleaning and validation](../03-cleaning-and-validation/) · Next: [Lesson 5: JSON with serde](../05-json-with-serde/)
