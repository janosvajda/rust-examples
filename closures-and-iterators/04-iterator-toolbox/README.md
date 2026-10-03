<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: The iterator toolbox

## The idea in one sentence

An iterator chain is a **source**, any number of **lazy adapters**, and one **consumer**, and nothing runs until the consumer asks for items.

## The shape of every chain

```rust
let big: Vec<u32> = sales.iter()          // SOURCE: where items come from
    .map(|s| s.amount)                    // ADAPTER: transform each item
    .filter(|&a| a >= 100)                // ADAPTER: keep some items
    .collect();                           // CONSUMER: run it, build the result
```

### Adapters are lazy

`map` and `filter` don't do anything themselves. They just wrap the iterator in another iterator. The work happens when the consumer pulls items through the chain, **one item at a time, through every step**. The demo records each `map` call to show it only happens once `collect` runs. A chain without a consumer does nothing, and the compiler warns:

```text
warning: unused `Map` that must be used
  = note: iterators are lazy and do nothing unless consumed
```

Laziness is also why endless iterators are fine: `(0..).step_by(2).take(5)` only ever produces five numbers. A test checks the `map` runs exactly 3 times for `.take(3)`.

## The toolbox

**Transform**

| Method | Does | Example |
|---|---|---|
| `map(f)` | change each item | `.map(\|s\| s.amount)` |
| `filter(f)` | keep items where `f` is true | `.filter(\|&a\| a >= 100)` |
| `filter_map(f)` | map and drop the `None`s in one step | `.filter_map(\|s\| s.parse().ok())` |
| `flat_map(f)` | map each item to *several*, flattened | `.flat_map(\|s\| s.chars())` |

**Position and grouping**

| Method | Does |
|---|---|
| `enumerate()` | pairs each item with its index: `(0, a), (1, b)…` |
| `zip(other)` | pairs items from two iterators, stopping at the shorter one |
| `windows(n)` *(slices)* | overlapping groups: `[1,2], [2,3], [3,4]` |
| `chunks(n)` *(slices)* | non-overlapping groups: `[1,2], [3,4], [5]` |

**Cut the stream**

| Method | Does |
|---|---|
| `take(n)` / `skip(n)` | the first `n` items / everything after them |
| `take_while(f)` / `skip_while(f)` | while a condition holds |
| `step_by(n)` | every `n`-th item |

**Consumers that answer questions**

| Method | Returns |
|---|---|
| `sum()`, `product()`, `count()` | a number |
| `min()`, `max()`, `min_by_key(f)`, `max_by_key(f)` | `Option` of the item |
| `any(f)`, `all(f)` | `bool`; stops as soon as the answer is known |
| `find(f)`, `position(f)` | `Option` of the first match / its index |
| `fold(start, f)` | anything: builds a result step by step |

**Combine streams:** `chain(other)` (one after the other), `rev()` (backwards), `peekable()` (look at the next item without taking it).

## `collect` builds whatever you ask for

```rust
let set: HashSet<&str> = sales.iter().map(|s| s.product).collect();
let text: String = sentence.chars().map(|c| c.to_ascii_uppercase()).collect();
let (north, south): (Vec<_>, Vec<_>) = sales.iter().partition(|s| s.region == "north");
let totals: Result<Vec<i32>, _> = texts.iter().map(|t| t.parse()).collect();  // error-handling lesson 5
```

`collect` can build a `Vec`, `HashSet`, `HashMap`, `String`, `Result`, `Option` and more. It decides which from the **type you ask for**. If you don't say, it can't decide:

```text
error[E0283]: type annotations needed
```

Write the type on the variable (`let v: Vec<_> = …`) or on the call: `.collect::<Vec<_>>()`.

## Iterators or loops?

Use whichever reads better. A chain like `.filter().map().sum()` compiles to the same machine code as a hand-written loop. Iterators are a *zero-cost abstraction*. They can even be faster, because they don't need bounds checks on every index. Loops are clearer when the body has several steps, early exits or complicated state.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: Storing and returning closures](../03-storing-and-returning-closures/) · Next: [Lesson 5: Writing your own iterators](../05-writing-iterators/)
