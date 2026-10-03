<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: thiserror and anyhow

## The idea in one sentence

Two widely used crates take the boilerplate out of error handling: **`thiserror`** for precise error types in **libraries**, and **`anyhow`** for easy, informative errors in **applications**.

## Which one, when?

| | `thiserror` | `anyhow` |
|---|---|---|
| Use in | **library** code: code that others call | **application** code: `main`, command handlers, glue |
| Gives you | your own error **enum**, which callers can `match` on | one error type that holds **any** error |
| Callers can | react to each specific case | mostly report it: print, log, exit |
| Boilerplate removed | `Display`, `Error`, `source()`, `From` impls | defining error types at all |

They're often used **together**: each module returns its own `thiserror` enum, and the application wraps everything in `anyhow` with context. That's exactly what this example does.

## `thiserror`: lesson 3's error type, in a few lines

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("setting `{key}` is missing")]
    Missing { key: String },

    #[error("setting `{key}` must be a number")]
    NotANumber { key: String, source: ParseIntError },

    #[error("setting `{key}` is {value}, but the maximum is {max}")]
    OutOfRange { key: String, value: u32, max: u32 },
}
```

This generates exactly what lesson 3 wrote by hand, and prints exactly the same messages:

| Attribute | Generates |
|---|---|
| `#[derive(thiserror::Error)]` | `impl std::error::Error` |
| `#[error("… {key} …")]` | `impl Display`, using the variant's fields in the message |
| a field named `source`, or marked `#[source]` | `Error::source()` returning it, so the error chain works |
| `#[from]` on a field | `impl From<ThatType>`, so `?` converts automatically, **and** marks it as the source |

```rust
#[derive(Debug, thiserror::Error)]
enum StartupError {
    #[error("invalid configuration")]
    Config(#[from] config::ConfigError),
    #[error("could not read the configuration file")]
    Io(#[from] std::io::Error),
}
```

`thiserror` adds nothing at runtime. It only writes the code you would otherwise write yourself.

## `anyhow`: any error, plus context

```rust
use anyhow::{Context, Result};

fn start(path: &str) -> Result<ServerConfig> {          // = Result<ServerConfig, anyhow::Error>
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config file {path}"))?;
    let config = config::load(&text).context("parsing the config")?;
    …
}
```

- **`anyhow::Result<T>`** accepts any error type through `?`, with no `From` impls needed.
- **`.context("…")`** wraps the error with a message saying *what you were doing*. The original error is kept as its cause, so nothing is lost. Use **`.with_context(|| …)`** when building the message costs something, like `format!`, so it's only built if there's an error.
- **`bail!("…")`** returns early with a new error. **`anyhow!("…")`** creates one without returning.

## Printing an `anyhow::Error`

How much you see depends on the format, exactly as printed by the demo:

| Format | Output |
|---|---|
| `{error}` | parsing the config *(only the outermost message)* |
| `{error:#}` | parsing the config: setting \`port\` must be a number: invalid digit found in string *(the whole chain, one line)* |
| `{error:?}` | the message plus a numbered list, shown below. This is also what you get when `main` returns an `anyhow::Error`. |

```text
parsing the config

Caused by:
    0: setting `port` must be a number
    1: invalid digit found in string
```

## Getting the original error back

An `anyhow::Error` can still be inspected. **`downcast_ref::<T>()`** returns `Some(&T)` if the original error, even beneath context layers, is a `T`:

```rust
if let Some(config_error) = error.downcast_ref::<config::ConfigError>() { … }
```

If you find yourself downcasting a lot, that part of the code probably wants a `thiserror` enum instead.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: Custom error types](../03-custom-error-types/) · Next: [Lesson 5: Everyday patterns](../05-error-handling-patterns/)
