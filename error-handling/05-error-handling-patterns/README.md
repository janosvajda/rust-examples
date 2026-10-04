<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Everyday patterns

## The idea in one sentence

A handful of patterns cover most of the error handling in real Rust code, and each has a short, idiomatic form.

## 1. Transform results without `match`: combinators

| Method | Runs when | Does |
|---|---|---|
| `.map(f)` | `Ok` | changes the success value |
| `.map_err(f)` | `Err` | changes the error, e.g. to add detail or convert the type |
| `.and_then(f)` | `Ok` | runs another step that can **also fail** |
| `.ok()` | always | `Result` → `Option`, throwing the error away **on purpose** |
| `.ok_or(e)` | always | `Option` → `Result`, supplying the error |

```rust
text.parse::<f64>()
    .map_err(|e| format!("\"{text}\" is not a temperature: {e}"))
    .and_then(|c| if c < -273.15 { Err(…) } else { Ok(c) })
    .map(|c| c * 9.0 / 5.0 + 32.0)
```

Read it top to bottom: parse, explain a parse failure, check it's physically possible, convert. Each step is skipped automatically once something has failed. Use `?` when you'd rather write the steps as separate lines. Both are idiomatic.

## 2. Many results at once

You have a list of inputs, and each one can fail. Three choices, depending on what one failure should mean:

| You want | Write | Result for `["10", "x", "30", "y"]` |
|---|---|---|
| **all or nothing**: stop at the first error | `.collect::<Result<Vec<_>, _>>()` | `Err(invalid digit…)`, and it stops at `"x"` |
| **keep going**: good values and a list of problems | loop and push into two `Vec`s | `[10, 30]` and two problems |
| **skip failures** silently | `.flat_map(\|s\| s.parse())` | `[10, 30]` |

`collect` into a `Result` is the surprising one: an iterator of `Result<T, E>` can be collected straight into `Result<Vec<T>, E>`.

## 3. `let … else`, let chains and `if let` guards

```rust
let Some(name) = users.get(&id) else {
    return String::from("Hello, guest!");
};
// `name` is usable from here on
```

Binds the value if the pattern matches, otherwise runs the `else` block, which **must** leave the current function or loop (`return`, `break`, `continue`, or a panic). It keeps the "happy path" unindented, without nesting the rest of the function inside an `if let`.

### Several steps in one condition: let chains

Often a value is only useful after several steps, each of which can fail: it must exist, it must parse, and it must be in range. Nested `if let`s march to the right:

```rust
if let Some(text) = settings.get("admin_port") {
    if let Ok(port) = text.parse::<u16>() {
        if port >= 1024 {
            …
```

A **let chain** joins them with `&&` into one condition:

```rust
if let Some(text) = settings.get("admin_port")
    && let Ok(port) = text.trim().parse::<u16>()
    && port >= 1024
{
    Some(port)
} else {
    None
}
```

The steps run from left to right, and the first one that fails sends you to `else`. Every name bound along the way (`text`, `port`) can be used in the steps after it and in the body. Let chains work in `if` and `while`, and need edition 2024.

### A match arm that needs one more step: `if let` guards

A match guard (`if condition`) can also be an `if let`. Then the arm only applies when that extra pattern matches too, and its bindings can be used in the arm:

```rust
match input.split_once(' ') {
    Some(("wait", seconds)) if let Ok(n) = seconds.parse::<u32>() => format!("waiting {n} s"),
    Some(("wait", seconds)) => format!("`{seconds}` is not a number of seconds"),
    Some((command, _)) => format!("unknown command `{command}`"),
    None => format!("`{input}` needs an argument"),
}
```

If `seconds` doesn't parse, the first arm doesn't apply, and matching simply continues with the next one, which reports the problem. Without the guard, the parsing and its error handling would have to be squeezed inside the arm's body.

## 4. Fallbacks

```rust
env_value.and_then(|v| v.parse().ok()).unwrap_or(8080)   // a bad or missing value → 8080
primary.or(backup)                                        // first source, else the second
```

Use fallbacks when a sensible default really exists. Silently replacing bad input with a default can hide mistakes, so be deliberate about it. `.ok()` makes the "I'm discarding this error" decision visible in the code.

## 5. Retrying

```rust
fn retry<T, E>(attempts: u32, mut operation: impl FnMut(u32) -> Result<T, E>) -> Result<T, E>
```

Some failures are temporary: a busy server, a dropped connection. `retry` calls the operation until it succeeds or runs out of attempts, then returns the **last** error. The `Err(error) if attempt >= attempts` match arm uses a **guard** to tell "out of attempts" apart from "try again". Real code usually also waits between attempts, often a little longer each time ("exponential backoff").

## The course in one table

| Situation | Use |
|---|---|
| a bug; something that can't happen | `panic!`, `assert!`, `expect("why it can't fail")` |
| failure is normal and the caller must decide | return `Result<T, E>` |
| "not there" is the whole story | return `Option<T>` |
| pass an error up | `?` |
| quick prototype, or `main` | `Box<dyn Error>` |
| a library: callers need to tell errors apart | your own enum, with `thiserror` |
| an application: report clearly what went wrong | `anyhow` with `.context(…)` |
| transform or combine results | `map`, `map_err`, `and_then`, `collect`, `let … else` |
| several fallible steps in one condition | let chains: `if let … && let … && …` |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 4: thiserror and anyhow](../04-thiserror-and-anyhow/) · Back to the [course overview](../)
