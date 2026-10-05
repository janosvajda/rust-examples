<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Cleaning and validation

## The idea in one sentence

Real data is messy, so before you calculate anything, clean it: fix what can be fixed, reject what can't, remove duplicates, and **report every change**, because data that silently changes or disappears can't be trusted.

## The messy data

`data/messy_readings.csv` looks like many real exports:

```text
timestamp,sensor,value,unit
2024-09-10 12:00,A12,23.5,Celsius
2024-09-10 12:05, a12 ,24.1,celsius          ← spaces, lower case
2024-09-10 12:10,A12,75.2,°F                  ← a different unit
2024-09-10 12:15,A12,,Celsius                 ← no value
2024-09-10 12:20,A12,n/a,Celsius              ← "no value", written as text
2024-09-10 12:25,A12,-300,Celsius             ← colder than absolute zero
                                              ← an empty line
2024-09-10 12:30,B22,760,mmHg
2024-09-10 12:35,B22,101.3,kPa                ← a different unit
2024-09-10 12:40,B22,755                      ← a missing column
2024-09-10 12:45,B22,750,mHg                  ← a typo in the unit
2024-09-10 12:00,A12,23.5,Celsius             ← the same row again
2024-09-10 12:50,C33,7.1,pH
2024-09-10 12:55,C33,15.2,pH                  ← outside this dataset's accepted pH range, 0–14
2024-09-10 13:00,C33,6.9,PH                   ← different capitalisation
2024-09-10 12:30,B22,765,mmHg                 ← same sensor and time as line 9, different value
2024-09-10 13:05,C33,inf,pH                   ← "infinity": parses as an infinite float, not a finite measurement
```

## The result: every row accounted for

```text
16 rows in, 7 clean readings out

Fixed along the way (5):
    line  3: sensor ` a12 ` → `A12`
    line  3: unit `celsius` → `Celsius`
    line  4: 75.2 °F → 24.0 Celsius
    line 10: 101.3 kPa → 759.8 mmHg
    line 16: unit `PH` → `pH`

Rejected (8):
    line  5: the value is missing
    line  6: the value is missing
    line  7: -300 Celsius is outside the accepted range (≥ -273.15, absolute zero)
    line 11: expected 4 fields, found 3
    line 12: unknown unit `mHg`
    line 15: 15.2 pH is outside the accepted range (0 to 14)
    line 17: conflicts with line 9: 765 here, 760 there (kept the first)
    line 18: `inf` is not a valid number

Duplicates dropped (1): lines [13]
```

7 kept + 8 rejected + 1 duplicate = 16 rows. Nothing vanished without an explanation, and a test checks exactly that.

## The steps

### 1. Normalise: many spellings, one meaning

The same thing gets written in many ways: `Celsius`, `celsius`, `C`, `°C`. Pick **one canonical form** and convert everything to it:

```rust
match text.trim().to_lowercase().as_str() {
    "celsius" | "c" | "°c" => Some((Unit::Celsius, same)),
    "fahrenheit" | "f" | "°f" => Some((Unit::Celsius, fahrenheit_to_celsius)),
    "mmhg" => Some((Unit::MmHg, same)),
    "kpa" => Some((Unit::MmHg, kpa_to_mmhg)),
    "ph" => Some((Unit::Ph, same)),
    _ => None,                                   // unknown: reject, don't guess
}
```

After this, the unit is an `enum Unit`, not a string. A typo like `mHg` can't sneak further into the program: it's either a known `Unit` or a rejected row. Converting `°F` to Celsius at the start means every later calculation compares like with like.

### 2. Treat "no value" as no value

`""`, `n/a`, `NA` and `-` all mean "nothing was measured". Never turn them into `0`: a missing temperature isn't 0 °C, and averaging in fake zeros quietly ruins every result.

Watch out for `NaN` and `inf`, too. Rust's `"NaN".parse::<f64>()` and `"inf".parse::<f64>()` **succeed**. And infinity even passes a check like `value >= -273.15`. So the cleaner accepts only finite numbers: `.filter(|v| v.is_finite())`.

### 3. Validate: is the value accepted?

A value can parse perfectly and still fall outside the application's accepted range:

| Unit | Accepted values | Why |
|---|---|---|
| Celsius | ≥ -273.15 | nothing is colder than absolute zero |
| mmHg | > 0 | this dataset's rule: our sensors can't measure a perfect vacuum |
| pH | 0 to 14 | this dataset's rule; very strong acids and bases can go beyond it |

Only absolute zero is a law of nature. The other two ranges are **rules for this dataset**, chosen for its sensors. Either way, an out-of-range value needs a person to look at it: it may be a sensor fault, a typo, or a real but unusual measurement.

### 4. Remove duplicates *after* cleaning, and catch conflicts

Two rows are the same reading if they have the same **sensor and time**. Check this after normalising, because ` a12 ` and `A12` are the same sensor. Then look at the values:

| Same sensor and time, and… | It's a… | So |
|---|---|---|
| the same value | **duplicate**: a harmless repeat, for example an export that ran twice | drop it, count it |
| a **different** value | **conflict**: one of them is wrong | keep the first, and **report** the other, so a person can decide |

```rust
if let Some(&(first_line, kept)) = seen.get(&key) {
    if kept == reading.value {
        report.duplicates.push(line_number);
    } else {
        report.problems.push((line_number, Problem::Conflict { first_line, kept, other: reading.value }));
    }
}
```

Quietly dropping a conflicting row would hide the fact that the data contradicts itself.

### 5. Keep going, and collect every problem

Lesson 1 stopped at the first bad row, which is right for a config file. For a data file with a million rows, one bad row shouldn't throw away the other 999,999. So `clean_row` returns a `Result`, and `clean` **collects** both sides: good readings into one list, problems with their line numbers into another. Each problem is an enum variant:

```rust
enum Problem {
    WrongFieldCount(usize),
    MissingValue,
    NotANumber(String),
    UnknownUnit(String),
    OutOfRange { value: f64, unit: Unit, allowed: &'static str },
    Conflict { first_line: usize, kept: f64, other: f64 },
}
```

That makes the report easy to count and sort, and lets tests check exactly which problem a row has. (The [error handling course](../../error-handling/03-custom-error-types/) explains error enums in detail.)

### 6. Report every fix

Rejecting is visible; **fixing** can be invisible, and that's dangerous. If the cleaner turns `75.2 °F` into `24.0 Celsius`, someone may later wonder why the file says 75.2. So every change is recorded too. A cleaning step that can't explain itself is a cleaning step nobody can check.

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| spaces and capitals in ids (` a12 `) | normalised to `A12`, and the fix is reported | `spellings_are_normalised` |
| the same unit spelled differently (`celsius`, `°C`, `PH`) | normalised, and reported | same |
| a different unit (`°F`, `kPa`) | converted, and reported | `units_are_converted` |
| an unknown unit or a typo (`mHg`) | rejected: never guessed | `each_problem_is_named` |
| no value: empty, `n/a`, `NA`, `-` | rejected as missing, never turned into 0 | same |
| `NaN`, `inf` | rejected: they parse as `f64`, but aren't measurements | same |
| a value outside the accepted range | rejected with the allowed range | same |
| a missing column | rejected with the count | same |
| an exact duplicate row | dropped and counted | `duplicates_and_conflicts_are_told_apart` |
| a **conflicting** duplicate (same key, different value) | reported as a problem, never dropped silently | same |
| empty lines | skipped | `the_messy_file_is_fully_accounted_for` |
| an empty file or only a header | nothing kept, nothing rejected | `empty_and_header_only_input` |
| **every** row | ends up kept, rejected or counted as a duplicate | `the_messy_file_is_fully_accounted_for` |

Not handled here: a decimal **comma** (`23,5`, common in many European countries) collides with the CSV separator. It produces the wrong number of fields, so it's rejected, never misread. Files from such systems usually use `;` as the separator, and a parser needs to be told which one.

## Two more checks

- **Check again after converting.** A number can be fine before conversion and not after: `1e308 kPa` becomes infinity when it's converted to another unit. So the cleaner checks that the value is finite *after* converting too.
- **Same time, different units is a conflict, not a duplicate.** `20 °C` and `20 mmHg` from the same sensor at the same moment can't both be right. Only identical values in the same unit count as a repeat. The first valid reading is kept, and the conflict is reported for a person to check.

The 0–14 range for pH is this example's own rule for its sensors, not part of pH's [official definition](https://goldbook.iupac.org/terms/view/P04524).

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: Big files: streaming](../02-streaming-big-files/) · Next: [Lesson 4: Aggregation and statistics](../04-aggregation-and-statistics/)
