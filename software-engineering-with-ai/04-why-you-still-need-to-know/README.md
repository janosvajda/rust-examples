<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Why you still need to know what is what

> Written in October 2026. AI models get better at this every year. Check whether what you read here still holds.

## The idea in one sentence

An AI tends to write code that **looks** right and **makes the problem go away**, and only someone who knows the language and the problem can tell when that isn't the same as **good** code.

## Why "plausible" is the trap

A language model learns from enormous amounts of code, and it's very good at producing code that resembles it. Plausible code **compiles**, works on the **obvious** input, and **looks** like what an experienced programmer would write.

But "looks like good code" and "is good code" differ in exactly the places that need knowledge:
- the empty list;
- the huge input;
- the bad input;
- the algorithm that's fine for 100 items and hopeless for 100,000;
- the copy that wasn't needed.

When you ask an AI to "optimise" or "fix" something, it optimises towards what you can check. If **you** can't judge the result, you'll accept whatever looks finished.

This lesson's code has four functions, each in two versions:
- `plausible`: the kind of first draft an AI easily produces;
- `better`: what someone who knows the problem writes.

All eight compile, and all eight work on the first input you'd try.

## 1. The average that crashes and rounds

```rust
// plausible
pub fn average(numbers: &[i32]) -> i32 {
    numbers.iter().sum::<i32>() / numbers.len() as i32
}
```

`average(&[2, 4, 6])` is `4`. Ship it? Here are three problems, and you only see them if you know what to look for:

| Input | Result | Why |
|---|---|---|
| `[]` | panic: `attempt to divide by zero` | an empty list has length 0 |
| `[1, 2]` | `1` | integer division throws away the `.5` |
| `[i32::MAX, i32::MAX]` | panic: `attempt to add with overflow` (in a debug build) | the sum doesn't fit in an `i32` |

```rust
// better
pub fn average(numbers: &[i32]) -> Option<f64> {
    if numbers.is_empty() {
        return None;                       // no numbers, no average: say so
    }
    let sum: i64 = numbers.iter().map(|&n| i64::from(n)).sum();   // can't overflow
    Some(sum as f64 / numbers.len() as f64)                       // 1.5, not 1
}
```

**The knowledge needed:** integer division, overflow, and that "no answer" is a real case which `Option` expresses (see the [error handling course](../../error-handling/)).

## 2. The duplicate check that doesn't scale

```rust
// plausible: compare every pair
for i in 0..values.len() {
    for j in (i + 1)..values.len() {
        if values[i] == values[j] { return true; }
    }
}
```

It's correct. The tests pass. It's also **O(n²)**: for n values it makes about n²/2 comparisons. `cargo run` times both versions:

```text
   5000 values: plausible    69.54ms   better   885.50µs
  10000 values: plausible   251.58ms   better     1.71ms
  20000 values: plausible      1.06s   better     3.87ms
```

Your numbers will differ, because they depend on the computer. The **pattern** won't: each time the input doubles, the plausible version gets about **4×** slower, and the better one about **2×**. At a million values, the plausible version would take hours.

```rust
// better: remember what we've seen
let mut seen = HashSet::with_capacity(values.len());
!values.iter().all(|v| seen.insert(v))      // insert returns false for a repeat
```

**The knowledge needed:** complexity (O(n²) vs O(n)), and which data structure answers "have I seen this before?" (see the [hash table](../../data-structures/hashing/hash-table/)).

This is also why "ask the AI to make it faster" isn't enough. An AI may speed up the inner loop by 10% and keep the O(n²). Knowing to ask for a **different algorithm** is the 100× improvement.

## 3. The clone that silences the borrow checker

```rust
// plausible
pub fn longest_name(names: &[String]) -> String {
    let mut longest = String::new();
    for name in names {
        if name.len() > longest.len() {
            longest = name.clone();              // a new heap copy every time
        }
    }
    longest
}
```

`.clone()` is the most common way to make borrow-checker errors disappear, and AIs reach for it often. Here it copies text that the caller already owns. The function also can't tell "no names" apart from "the longest name is empty": both return `""`.

```rust
// better
pub fn longest_name(names: &[String]) -> Option<&str> {
    names.iter().max_by_key(|name| name.len()).map(String::as_str)
}
```

It borrows instead of copying, and returns `None` for an empty list.

### A surprise: who wins a tie?

Running both versions on `["Ada", "Grace", "Linus"]` gives:

```text
plausible: "Grace"
better:    Some("Linus")
```

"Grace" and "Linus" are both 5 letters long. The loop keeps the **first** longest name, while `max_by_key` returns the **last**.

Which is right? **Neither, until someone decides.** Nobody wrote down what should happen on a tie, so each version made a silent choice. An AI won't ask about this unless you do. A test that pins the decision makes it explicit (see `longest_name` in the tests).

**The knowledge needed:** ownership and borrowing (see the [ownership and borrowing course](../../ownership-and-borrowing/)), and the habit of asking "what about ties, duplicates and empty input?"

## 4. The parser that trusts its input

```rust
// plausible
pub fn parse_port(line: &str) -> u16 {
    line.split('=').nth(1).unwrap().parse().unwrap()
}
```

It works for `port=8080`. It panics for `port = 8080` (the space), `port=abc`, `port` and `port=70000`. Config files are written by people, so bad input isn't rare: it's **normal**.

```rust
// better
pub fn parse_port(line: &str) -> Result<u16, String> {
    let (key, value) = line.split_once('=').ok_or(format!("expected `port=<number>`, got `{line}`"))?;
    if key.trim() != "port" {
        return Err(format!("expected the key `port`, got `{}`", key.trim()));
    }
    value.trim().parse().map_err(|_| format!("`{}` is not a port number (0-65535)", value.trim()))
}
```

```text
better::parse_port("port=abc") = Err("`abc` is not a port number (0-65535)")
better::parse_port("port")     = Err("expected `port=<number>`, got `port`")
```

**The knowledge needed:** when to panic and when to return an error. The rule of thumb: bugs panic, expected failures return `Result`. See [panic vs Result](../../error-handling/01-panic-vs-result/).

## What the four examples have in common

| Example | Looks fine because… | Knowledge that spots the problem |
|---|---|---|
| average | the obvious input works | empty input, overflow, integer division |
| duplicates | it's correct | complexity, choosing a data structure |
| longest name | it compiles and borrows nothing | ownership, and asking about ties and empty input |
| parse port | the example config works | panic vs `Result`, real-world input |

None of this is exotic. It's ordinary programming knowledge, and it's exactly what lets you **review** AI-written code, and **ask** for the right thing in the first place ([Lesson 3](../03-prompting/)).

## Run it

```bash
cargo run      # shows every example, and times the duplicate check
cargo test     # proves where the plausible versions break
```

## What may change

AI models make fewer of these mistakes every year, and some of these examples may soon look old-fashioned. But new models will make new kinds of plausible mistakes, and **the ability to judge** is what catches them.

Previous: [Lesson 3: Prompting](../03-prompting/) · Next: [Lesson 5: Why Rust fits AI-assisted development](../05-why-rust-fits-ai/)
