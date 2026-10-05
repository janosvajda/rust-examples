<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 9: Time and windows

## The idea in one sentence

Turn timestamps from text into numbers as early as possible. Then sorting, measuring durations, grouping into time windows and finding where data is **missing** all become simple arithmetic.

## Why text isn't enough

`"2024-09-11 00:05:00"` is fine for printing. But how many minutes lie between it and `"2024-09-10 23:55:00"`? Text can't answer: the date changed, the hour went from 23 to 00, and next time a month might end or a leap day might sit in between.

A number can. This lesson stores a time as **seconds since 1970-01-01 00:00:00 UTC**, the **Unix timestamp** that almost every computer system uses inside:

```rust
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Timestamp(i64);
```

Comparing, sorting and subtracting are now simple integer operations: ten minutes across midnight is 600 seconds.

## 1. Parsing: only real dates

```text
2024-09-10 23:55:00 → 1726012500 seconds since 1970
2024-02-29 12:00:00 → 1709208000 seconds since 1970
2023-02-29 12:00:00 → error: 2023-02-29 doesn't exist
2024-13-01 00:00:00 → error: 2024-13-01 doesn't exist
```

`Timestamp::parse` checks that the date exists: month 1–12, and the right number of days for that month, with **leap years**. A year divisible by 4 is a leap year, except years divisible by 100, except years divisible by 400. So 2024 and 2000 were leap years, and 1900 wasn't.

Turning a date into a day count uses a well-known algorithm by Howard Hinnant. Its trick is to count each year from **March**, so the awkward leap day falls at the very end of the year. `Display` converts back, and a test checks that both directions agree, including dates before 1970 and far in the future. Other tests compare against values from Python's `datetime`, an independent implementation.

## 2. Out of order: sort by the number

Data doesn't always arrive in order. Sensors buffer, networks delay, files get merged:

```text
2 readings arrived earlier than the one before them
sorted: 2024-09-10 23:30:00 → 2024-09-11 01:30:00
span: 120 minutes, across midnight
```

`readings.sort_by_key(|r| r.at)` fixes it. Sorting **text** timestamps only works if every one has exactly the same format, like `2024-09-10 09:05`. One `2024-9-10 9:05` sorts in the wrong place. Numbers always sort correctly.

## 3. Gaps: when the data stops

The sensor reports every 5 minutes. If two neighbouring readings are further apart, data is missing:

```rust
sorted.array_windows()
    .filter(|[a, b]| b.at.0 - a.at.0 > expected_interval)
    .map(|[a, b]| (a.at, b.at, (b.at.0 - a.at.0) / expected_interval - 1))
```

```text
silent from 2024-09-11 00:15:00 to 2024-09-11 00:50:00: 35 minutes, 6 readings missing
```

Missing data is **invisible** unless you look for it: no row says "nothing happened here". A gap could be a power cut, a network problem or a dead battery, and every average across it is based on less data than it seems.

## 4. Tumbling windows

A **window** groups readings by time: "the average per 15 minutes". The window a reading belongs to is just its timestamp rounded down:

```rust
fn window_start(at: Timestamp, size: i64) -> Timestamp {
    Timestamp(at.0 - at.0.rem_euclid(size))      // 00:07:30 → 00:00:00 for 15 minutes
}
```

Then group by the window start, exactly like grouping by lab in [lesson 4](../04-aggregation-and-statistics/):

```text
2024-09-11 00:00:00  readings: 3, average 21.73 °C
2024-09-11 00:15:00  readings: 1, average 21.50 °C  ← incomplete window
2024-09-11 00:45:00  readings: 2, average 22.45 °C  ← incomplete window
2024-09-11 01:00:00  readings: 3, average 22.33 °C
```

Two things to notice:
- **The 00:30 window is missing completely.** A window with no readings has nothing to group, so it never appears. If a report or chart needs every window, generate the list of all windows first, then fill in the data.
- **Incomplete windows** have fewer readings than expected, so their average is less reliable. The program marks them. Hiding that would make every number look equally trustworthy.

These windows are **tumbling**: fixed and non-overlapping. Each reading belongs to exactly one. **Sliding** windows overlap, like the moving average in [lesson 7](../07-live-data-over-the-network/).

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| a date that doesn't exist (Feb 29 in 2023, April 31) | rejected | `impossible_dates_are_rejected` |
| leap years, including 1900 (not one) and 2000 (one) | correct | same |
| an hour of 24, a minute of 60 | rejected | same |
| no zero padding (`2024-9-1 9:05:00`) | accepted, and means the same moment | `formats_people_actually_write` |
| ISO 8601 with a `T`, or a time zone (`Z`, `+02:00`) | **rejected**, never misread. Times with an offset need converting to UTC, which needs a time zone library | same |
| an absurd year (`999999999999`) | rejected: it would overflow the seconds calculation | same |
| dates before 1970 | work: the timestamp is simply negative | `display_round_trips` |
| readings out of order | sorted by timestamp | (the demo) |
| two readings at the same moment | not a gap; both land in the same window | `window_boundaries_and_equal_timestamps` |
| a reading exactly on a window boundary (00:15:00) | belongs to the window that **starts** there | same |
| fewer than two readings | no gaps can be found | same |
| `NaN` or `inf` as a value | rejected | `nan_values_are_rejected` |
| a gap in the data | found, with its length and the number of missing readings | `the_data_has_one_gap_of_six_readings` |

Edge cases this lesson doesn't detect, because they need information the data doesn't have:
- **A gap before the first or after the last reading.** If the sensor should have reported from 22:00 but the file starts at 23:30, only an *expected* start and end time reveal it.
- **Wrong clocks.** A sensor whose clock is set wrong produces timestamps in the future, or the past. A plausibility check, such as "not later than now", catches the worst cases.
- **Leap seconds.** Unix time pretends they don't exist: every day has exactly 86,400 seconds. That's almost always what you want, and it's why this arithmetic stays simple.

## What this lesson leaves out: time zones

Everything here is in **UTC**, a single clock with no daylight saving time. That's how data should be stored. Local time adds hard problems:
- In spring, one hour doesn't exist on the clock.
- In autumn, one hour happens **twice**, so "01:30" is ambiguous.
- The rules change by country and by year.

Don't write that yourself. For time zones and calendars, use a crate: `jiff` has time zones built in; `chrono` needs a second crate, `chrono-tz`, for named zones like `Europe/Budapest`. Store and calculate in UTC, and convert to local time only to show it to a person.

## The calendar rules

The parser accepts years 1 to 9999, treats every time as UTC, and checks hours (0–23), minutes and seconds (0–59). A leap second written as `:60` is rejected rather than quietly turned into the next minute. It's a parser for this lesson's format, not for every date format in the world.

Each row needs exactly three fields, a sensor ID, and a temperature that's a finite number. Window sizes and the expected interval between readings must be positive. Windows with no readings are left out.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 8: Joining datasets](../08-joining-datasets/) · Back to the [course overview](../)
