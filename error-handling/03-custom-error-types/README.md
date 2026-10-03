<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Custom error types

## The idea in one sentence

A custom error type, usually an **enum**, tells the caller not just *that* something failed but *what* failed, so they can react to each case differently.

## Why `String` and `Box<dyn Error>` aren't always enough

With `Result<T, String>`, all a caller can do with the error is print it. They can't tell "the port setting is missing" apart from "the port is out of range" without parsing the message text, which is fragile. An enum makes each failure a separate case the compiler knows about:

```rust
enum ConfigError {
    Missing     { key: String },
    NotANumber  { key: String, source: ParseIntError },
    OutOfRange  { key: String, value: u32, max: u32 },
}
```

Now a caller can handle one case specially and pass the rest on:

```rust
let workers = match load_config(text) {
    Ok(config) => config.workers,
    Err(ConfigError::OutOfRange { max, .. }) => max,   // clamp instead of failing
    Err(other) => return Err(other),
};
```

## The three ingredients of a good error type

| Ingredient | What it's for |
|---|---|
| **`#[derive(Debug)]`** | developer-facing output, e.g. `{:?}` and `unwrap` panic messages. Required by the `Error` trait. |
| **`impl Display`** | the message for **people**: one clear line, like ``setting `workers` is 99, but the maximum is 64`` |
| **`impl std::error::Error`** | makes it a "real" error: it works with `?`, fits in `Box<dyn Error>`, and can point to its **cause** with `source()` |

## Error chains: `source()`

Errors often wrap other errors. "Invalid configuration" happened *because* "setting `port` must be a number", which happened *because* "invalid digit found in string". `source()` returns the next error in that chain:

```rust
impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::NotANumber { source, .. } => Some(source),
            _ => None,
        }
    }
}
```

The demo's `report` function follows `source()` all the way down:

```text
error: invalid configuration
        caused by: setting `port` must be a number
        caused by: invalid digit found in string
```

**Convention:** each error's own message describes only **its** level. It doesn't repeat its cause's message, because whoever prints the chain shows the causes.

## `From`: how `?` converts errors

Lesson 2 said `?` calls `.into()` on the error. Implement `From` and `?` does the conversion for you:

```rust
impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self { AppError::Config(error) }
}

fn start_server(path: &str) -> Result<ServerConfig, AppError> {
    let text = std::fs::read_to_string(path)?;   // io::Error   → AppError::Io
    let config = load_config(&text)?;            // ConfigError → AppError::Config
    Ok(config)
}
```

This is the usual structure in a larger program: each part has its own error enum, and a top-level error wraps them all.

## That's a lot of code

Two `Display` implementations, two `Error` implementations and two `From` implementations, about 60 lines of boilerplate for two small enums. Lesson 4 shows how the `thiserror` crate writes all of it for you from a few attributes, with exactly the same result.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: The `?` operator](../02-the-question-mark/) · Next: [Lesson 4: thiserror and anyhow](../04-thiserror-and-anyhow/)
