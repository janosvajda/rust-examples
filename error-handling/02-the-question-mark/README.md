<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: The `?` operator

## The idea in one sentence

`value?` means **"give me the success value, or return the error from this function right now"**. It's how errors travel up through your code without a `match` at every step.

## What `?` does, exactly

```rust
let x: i32 = a.parse()?;
```

is shorthand for:

```rust
let x: i32 = match a.parse() {
    Ok(value) => value,                    // success: unwrap and carry on
    Err(error) => return Err(error.into()), // failure: convert, then return early
};
```

The demo has both versions of the same function side by side, and a test checks they behave identically.

Notice the `.into()`: `?` **converts** the error into the function's error type using the `From` trait. That's what makes the next sections work.

## Errors travel up

```rust
fn parse_settings(text: &str) -> Result<HashMap<String, u32>, String> {
    for line in text.lines() {
        let (key, value) = parse_line(line)?;   // a bad line stops everything here…
        …
    }
}
```

```text
  main ──► parse_settings ──► parse_line ──► .parse::<u32>()
       ◄── Err(…)         ◄── Err(…)     ◄── Err(ParseIntError)
```

Each `?` hands the error one level up. The function that finally *handles* it, with a `match`, a fallback, or a message to the user, can be far away from where it happened.

## Making things fit `?`

`?` needs a `Result` (or an `Option`) whose error type can convert into the function's error type. Two helpers fix most mismatches:

| You have | You want | Use |
|---|---|---|
| `Option<T>` | `Result<T, E>` | `.ok_or(error)?` or `.ok_or_else(\|\| error)?` |
| `Result<T, E1>` | `Result<T, E2>` | `.map_err(\|e\| convert(e))?` |

```rust
let (key, value) = line.split_once('=').ok_or(format!("missing '=' in \"{line}\""))?;
let number = value.parse::<u32>().map_err(|e| format!("bad number for {key}: {e}"))?;
```

## `?` on `Option`

In a function that returns `Option`, `?` on an `Option` returns `None` early:

```rust
fn initials(full_name: &str) -> Option<String> {
    let first = words.next()?.chars().next()?;   // no words? → None
    …
}
```

You can't mix them, though. `?` on an `Option` inside a function returning `Result` doesn't compile:

```text
error[E0277]: the `?` operator can only be used on `Result`s, not `Option`s, in a function that returns `Result`
```

Convert with `.ok_or(…)` first.

## Different error types: `Box<dyn Error>`

```rust
fn read_number(path: &str) -> Result<i32, Box<dyn Error>> {
    let text = std::fs::read_to_string(path)?;   // can fail with io::Error
    let number = text.trim().parse::<i32>()?;    // can fail with ParseIntError
    Ok(number)
}
```

Two different error types in one function: with `-> Result<i32, ParseIntError>`, the file read doesn't fit:

```text
error[E0277]: `?` couldn't convert the error to `ParseIntError`
```

`Box<dyn Error>` means "any type that implements the `Error` trait", and every standard error type converts into it automatically. It's the quick solution. Lesson 3 shows how to build your own error type when callers need to tell the errors apart.

## `?` in `main` and in tests

`?` only works in a function that returns `Result` or `Option`:

```text
error[E0277]: the `?` operator can only be used in a function that returns `Result` or `Option` (or another type that implements `FromResidual`)
```

`main` can return one:

```rust
fn main() -> Result<(), Box<dyn Error>> {
    let n: i32 = "4x2".parse()?;
    Ok(())
}
```

If `main` returns `Err`, Rust prints it and exits with code **1**:

```text
Error: ParseIntError { kind: InvalidDigit }
```

Tests can return `Result` too, so you can use `?` inside them. An `Err` fails the test.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: panic vs Result](../01-panic-vs-result/) · Next: [Lesson 3: Custom error types](../03-custom-error-types/)
