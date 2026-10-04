<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 8: Joining datasets

## The idea in one sentence

When data is split across tables, a **join** combines them through a shared key, and the two things that matter most are what happens to the rows that **don't** match, and using an index (a `HashMap`) instead of a loop inside a loop.

## Two tables, one key

`data/readings.csv` says **which** sensor measured what:

```text
time,sensor,value
12:00,A12,23.5
12:05,X99,19.0      ← a sensor nobody registered
…
```

`data/sensors.csv` says **where** each sensor is:

```text
sensor,lab,room
A12,Lab 1,Microscopy
E55,Lab 1,Microscopy      ← installed, but never sent a reading
…
```

The `sensor` column appears in both: it's the **key** that connects them. Keeping the location in its own table, instead of repeating it in every reading, means it's written once and fixed in one place when a sensor moves. Databases call this **normalisation**.

## 1. Build an index first

```rust
fn index_by_id<'s, 'a>(sensors: &'s [Sensor<'a>]) -> Result<HashMap<&'a str, &'s Sensor<'a>>, Vec<&'a str>> {
    let mut index = HashMap::with_capacity(sensors.len());
    let mut duplicates = Vec::new();
    for sensor in sensors {
        if index.insert(sensor.id, sensor).is_some() {
            duplicates.push(sensor.id);           // this id was already there
        }
    }
    if duplicates.is_empty() { Ok(index) } else { Err(duplicates) }
}
```

One pass over the sensors builds a map from id to sensor. After that, finding the sensor for any reading is a single lookup, no matter how many sensors there are. Every join below uses this index.

**The key must be unique.** The shortest version, `sensors.iter().map(|s| (s.id, s)).collect()`, has a silent flaw. If A12 is registered twice, in Microscopy and again in the Cold room, `collect()` keeps the **last** one without a word, and every A12 reading gets joined to a location that may be wrong. `insert` returns the value it replaced, so the version above can notice and report the duplicate instead.

## 2. The kinds of join

They differ only in **what happens to the rows without a match**:

| Join | Keeps | Use it when |
|---|---|---|
| **inner** | only readings whose sensor is known | you only care about complete, matched data |
| **left** | every reading; the sensor is `None` when unknown | no reading may be lost |
| **anti** | only the rows that *don't* match | you're looking for problems |

```rust
// inner: drop what doesn't match
readings.iter().filter_map(|r| Some((r, *index.get(r.sensor)?))).collect()

// left: keep everything, Option for the missing side
readings.iter().map(|r| (r, index.get(r.sensor).copied())).collect()
```

```text
inner join: 8 of 9 readings matched
left join:  12:05 X99     19  location: None
```

The inner join **silently dropped** the X99 reading. Sometimes that's what you want, but be aware of it ([lesson 3](../03-cleaning-and-validation/): never lose data without knowing it). Rust makes the left join's gap visible: the location is an `Option`, so the compiler won't let you forget that it might be missing.

## 3. The rows that don't match are the interesting ones

```text
readings from unregistered sensors: ["X99"]
registered sensors with no readings: ["E55"]
```

Both are real problems a person should look at. X99 might be a new sensor someone forgot to register, or a typo in a sensor's configuration. E55 might be broken, unplugged or out of battery. **Anti joins** find them. The second one uses a `HashSet` of the sensors that did report, then keeps the registered sensors that aren't in it.

## 4. After the join: group by the other table's columns

```text
readings per room:
    Lab 1 / Microscopy: 3
    Lab 4 / Cold room: 2
```

The readings don't know about rooms; the sensors table does. After joining, you can group ([lesson 4](../04-aggregation-and-statistics/)) by any column from either table. That's the usual reason to join in the first place.

## 5. Why the index matters

Without an index, the obvious way to join is a loop inside a loop: for every reading, search through all the sensors.

```text
200,000 readings × 2,000 sensors:
    nested loops: 200,100,000 comparisons, 496.44ms
    hash join:        200,000 lookups,       2.88ms
```

The timings are from an Apple M2 with `cargo run --release`, so yours will differ. The **number of comparisons** won't: it's the real story. The nested loop does about `readings × sensors ÷ 2` comparisons, so doubling both tables makes it **4× slower**. The hash join does about `readings + sensors` steps, so doubling both makes it about **2× slower**. This is the O(n·m) vs O(n + m) difference from [Why you still need to know](../../software-engineering-with-ai/04-why-you-still-need-to-know/), and why every database builds an index for joins.

## Edge cases

| Edge case | What happens | Test |
|---|---|---|
| a reading for an unknown sensor | dropped by the inner join, kept with `None` by the left join, listed by the anti join | `inner_join_drops_unknown_sensors`, `left_join_keeps_every_reading`, `anti_joins_find_both_kinds_of_mismatch` |
| a sensor with no readings | listed by the other anti join | `anti_joins_find_both_kinds_of_mismatch` |
| the **same key twice** in the lookup table | an error listing the ids, instead of silently keeping the last | `a_sensor_registered_twice_is_an_error` |
| stray spaces around keys (`"A12 "`) | trimmed, so they still match | `keys_with_stray_spaces_still_match` |
| an empty table | joins to nothing; the left join still keeps every reading | `empty_tables_join_to_nothing` |
| `NaN` or `inf` readings | skipped when parsing | (in `parse_readings`) |
| big tables | a hash join: about `n + m` steps instead of `n × m` | `nested_loops_and_hash_join_agree` |

Two more that matter in real data, and aren't in this example:
- **Different spellings of the same key** (`a12` vs `A12`, `A-12` vs `A12`). Normalise both tables the same way **before** joining ([lesson 3](../03-cleaning-and-validation/)). A join only matches keys that are exactly equal.
- **Many-to-many joins.** If the key is unique on neither side, each match combines with every other match: 1,000 rows × 1,000 rows on the same key give 1,000,000 output rows. When a join's output is unexpectedly huge, check whether the key really is unique where it should be.

## Run it

```bash
cargo run --release
cargo test
```

Previous: [Lesson 7: Live data over the network](../07-live-data-over-the-network/) · Next: [Lesson 9: Time and windows](../09-time-and-windows/)
