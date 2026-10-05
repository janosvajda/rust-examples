<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: JSON with serde

## The idea in one sentence

Describe your data's shape once as a Rust struct, add `#[derive(Deserialize, Serialize)]`, and serde writes all the reading and writing code for you: typed, fast, and with error messages that say exactly what's wrong and where.

## Why a crate this time

Lessons 1–4 use only the standard library. JSON is different: its rules (escapes, Unicode, nesting, numbers) are big enough that writing a parser yourself is a project of its own. **serde** is the standard answer in Rust: almost every Rust program that reads or writes JSON, TOML, YAML or many binary formats uses it. It's one of the dependencies worth having (see [Fewer dependencies](../../software-engineering-with-ai/08-fewer-dependencies/)).

serde comes in two parts: `serde` describes **how** a type maps to data, and a format crate like `serde_json` handles the **format**.

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## 1. JSON → structs

The data, `data/readings.json`:

```json
{ "sensorId": "C33", "takenAt": "2024-09-10T12:10", "value": 7.1, "unit": "ph", "location": "Lab 3", "note": "probe cleaned" }
```

The struct describing it:

```rust
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]    // sensorId ↔ sensor_id; no unknown fields
struct Reading {
    sensor_id: String,
    taken_at: String,
    value: f64,
    unit: Unit,                             // an enum: only known units allowed
    location: String,
    note: Option<String>,                   // may be missing → None
    #[serde(default)]
    calibrated: bool,                       // may be missing → false
}

let readings: Vec<Reading> = serde_json::from_str(text)?;
```

One call reads the whole array into typed structs. Each attribute answers a real-world question:

| Attribute or type | Solves |
|---|---|
| `#[serde(rename_all = "camelCase")]` | JSON uses `sensorId`, Rust uses `sensor_id`: each side keeps its own naming style |
| `Option<String>` | the field may be absent (or `null`): you get `None` |
| `#[serde(default)]` | the field may be absent: you get the type's default (`false`, `0`, empty) |
| an `enum` field | only listed values are accepted: `"kelvin"` is an error at the door, not a bad string later |
| `deny_unknown_fields` | a field the struct doesn't have is an **error**. Without it, serde silently ignores unknown fields, so a typo like `"calibrate": true` would vanish, and `calibrated` would quietly be `false` |

The enum is the same idea as [lesson 3](../03-cleaning-and-validation/): turn text into a type that can only hold valid values, as early as possible.

## 2. Structs → JSON

The same derive works in the other direction:

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SensorSummary {
    sensor_id: String,
    unit: Unit,
    readings: usize,
    average: f64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    notes: Vec<String>,                    // left out entirely when empty
}

println!("{}", serde_json::to_string_pretty(&summaries)?);
```

```json
[
  { "sensorId": "A12", "unit": "celsius", "readings": 2, "average": 23.8 },
  …
  { "sensorId": "C33", "unit": "ph", "readings": 2, "average": 7.25, "notes": ["probe cleaned"] }
]
```

(Shown compactly here. `to_string_pretty` puts each field on its own line, and `to_string` writes it all on one line.)

## 3. Errors say what and where

Every one of these is the real message serde_json gives:

```text
expected `,` or `}` at line 1 column 21
invalid type: string "warm", expected f64 at line 1 column 52
unknown variant `kelvin`, expected one of `celsius`, `mmhg`, `ph` at line 1 column 65
missing field `location` at line 1 column 68
unknown field `calibrate`, expected one of `sensorId`, `takenAt`, `value`, `unit`, `location`, `note`, `calibrated` at line 1 column 96
```

A syntax error, a wrong type, a value that isn't allowed, a missing field and a typo. Each one names the problem and the position, and the last one even lists the fields it expected. The struct definition **is** the validation: you don't write a single `if`.

## 4. When you don't know the shape: `serde_json::Value`

Sometimes the JSON's shape isn't fixed, or you only need one field from a huge document. `serde_json::Value` holds any JSON, and you index into it:

```rust
let value: serde_json::Value = serde_json::from_str(line)?;
println!("{}", value["sensor"]);                 // "A12"
```

It's flexible, but every access might find nothing (`Value::Null`), and the compiler can't help you. Prefer a struct whenever you know the shape.

## 5. JSON Lines: JSON you can stream

Reading into a `Vec<Reading>` keeps the whole array in memory at once. (serde can process a big array one element at a time, but it takes considerably more code.) **JSON Lines** (`.jsonl`) is the simple alternative: one complete JSON object on each line:

```text
{"sensor":"A12","value":23.5}
{"sensor":"B22","value":760}
```

Each line can be parsed on its own, so it streams exactly like [lesson 2](../02-streaming-big-files/): read a line, parse it, update a summary, forget it. Logs and data exports often use it for that reason.

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| a syntax error | an error with line and column | (the demo) |
| a value of the wrong type (`"warm"` for a number) | an error naming the expected type | `unknown_units_and_wrong_types_are_rejected` |
| an unknown enum value (`"kelvin"`) | an error listing the allowed values | same |
| a missing required field | an error naming the field | (the demo) |
| a missing **optional** field | `None`, or the `#[serde(default)]` value | `camel_case_json_becomes_a_struct` |
| an **unknown** field, such as a typo | an error, thanks to `deny_unknown_fields`; serde would ignore it otherwise | `tricky_json_is_rejected_with_a_reason` |
| the same field twice | an error: `duplicate field` | same |
| `NaN` | an error: JSON has no `NaN` or infinity | same |
| a number too big for `f64` (`1e400`) | an error: `number out of range` (Rust's `str::parse` would quietly make it infinity) | same |
| text after the JSON | an error: `trailing characters` | same |
| very deep nesting (100,000 `[`) | an error at depth 128, not a crashed program: serde_json limits recursion | `deep_nesting_is_an_error_not_a_crash` |
| an empty array | an empty `Vec`; the summary is empty too | `an_empty_array_is_fine` |
| reading, writing and reading again | the same data | `round_trip` |

**A trade-off with `deny_unknown_fields`:** it makes a program strict about what it accepts. When the sender adds a new field, a strict reader rejects every message until it's updated. Use it for input you control, such as config files and your own data. For data from systems that evolve independently, accept unknown fields, and log them instead.

## Group by the unit, as well as the sensor

Summaries are grouped by sensor **and unit**. If A12 reports both `20 celsius` and `760 mmhg`, you get two summaries: averaging them into "390" would mix two different physical quantities and mean nothing.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 4: Aggregation and statistics](../04-aggregation-and-statistics/) · Next: [Lesson 6: A processing pipeline](../06-processing-pipeline/)
