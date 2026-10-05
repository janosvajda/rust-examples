// Lesson 4: thiserror and anyhow.
//
// Two popular crates that remove the boilerplate from lesson 3:
//   thiserror  for LIBRARY code: precise error enums that callers can match on.
//              It generates Display, Error, source() and From for you.
//   anyhow     for APPLICATION code: one easy error type for "anything went
//              wrong", plus context messages explaining what you were doing.

// ---- Library code: a precise error enum with thiserror -------------------------

mod config {
    use std::num::ParseIntError;
    use thiserror::Error;

    /// The same ConfigError as lesson 3. Each `#[error("…")]` becomes the
    /// Display message; fields can be used in it with `{name}`.
    #[derive(Debug, Error, PartialEq)]
    pub enum ConfigError {
        #[error("setting `{key}` is missing")]
        Missing { key: String },

        /// A field named `source` (or marked `#[source]`) becomes the
        /// underlying cause returned by `Error::source()`.
        #[error("setting `{key}` must be a number")]
        NotANumber { key: String, source: ParseIntError },

        #[error("setting `{key}` is {value}, but the maximum is {max}")]
        OutOfRange { key: String, value: u32, max: u32 },
    }

    #[derive(Debug, PartialEq)]
    pub struct ServerConfig {
        pub port: u32,
        pub workers: u32,
    }

    fn get_number(text: &str, key: &str, max: u32) -> Result<u32, ConfigError> {
        let raw = text
            .lines()
            .filter_map(|line| line.split_once('='))
            .find(|(k, _)| k.trim() == key)
            .map(|(_, v)| v.trim())
            .ok_or_else(|| ConfigError::Missing {
                key: key.to_string(),
            })?;
        let value: u32 = raw.parse().map_err(|source| ConfigError::NotANumber {
            key: key.to_string(),
            source,
        })?;
        if value > max {
            return Err(ConfigError::OutOfRange {
                key: key.to_string(),
                value,
                max,
            });
        }
        Ok(value)
    }

    pub fn load(text: &str) -> Result<ServerConfig, ConfigError> {
        Ok(ServerConfig {
            port: get_number(text, "port", 65_535)?,
            workers: get_number(text, "workers", 64)?,
        })
    }
}

/// A wrapper error, like AppError in lesson 3. `#[from]` generates the
/// `From` impl, so `?` converts automatically, AND marks it as the source.
#[derive(Debug, thiserror::Error)]
enum StartupError {
    #[error("invalid configuration")]
    Config(#[from] config::ConfigError),

    #[error("could not read the configuration file")]
    Io(#[from] std::io::Error),
}

fn load_from_file(path: impl AsRef<std::path::Path>) -> Result<config::ServerConfig, StartupError> {
    let text = std::fs::read_to_string(path)?; // io::Error → StartupError::Io
    Ok(config::load(&text)?) // ConfigError → StartupError::Config
}

// ---- Application code: anyhow -------------------------------------------------------

use anyhow::{Context, Result, anyhow, bail};

/// `anyhow::Result<T>` is `Result<T, anyhow::Error>`, and `anyhow::Error`
/// accepts any error type through `?`. `.context(…)` adds a message saying
/// what we were trying to do, and keeps the original error as its cause.
fn start(path: impl AsRef<std::path::Path>) -> Result<config::ServerConfig> {
    let path = path.as_ref();
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("reading config file {}", path.display()))?;
    let config = config::load(&text).context("parsing the config")?;

    if config.port < 1024 {
        // `bail!` returns early with a new error built from a message.
        bail!(
            "this example requires port 1024 or above; got {}",
            config.port
        );
    }
    Ok(config)
}

fn parse_percent(text: &str) -> Result<u8> {
    let value: u8 = text.parse().context("not a whole number")?;
    // `anyhow!` creates an error from a message without returning.
    (value <= 100)
        .then_some(value)
        .ok_or_else(|| anyhow!("{value} is more than 100%"))
}

fn main() {
    println!("1. thiserror: the same messages as lesson 3, without the boilerplate");
    for text in [
        "port=8080\nworkers=8",
        "workers=8",
        "port=x\nworkers=8",
        "port=80\nworkers=99",
    ] {
        match config::load(text) {
            Ok(c) => println!("    ok: {c:?}"),
            Err(e) => println!("    error: {e}"),
        }
    }

    let dir = std::env::temp_dir();
    let bad_number = dir.join("rust-examples-anyhow-bad.txt");
    let low_port = dir.join("rust-examples-anyhow-low.txt");
    std::fs::write(&bad_number, "port=eighty\nworkers=2").unwrap();
    std::fs::write(&low_port, "port=80\nworkers=2").unwrap();

    println!("\n2. thiserror with #[from]: `?` converts automatically");
    if let Err(e) = load_from_file("/no/such/config.txt") {
        println!(
            "    {e} (variant: {})",
            if matches!(e, StartupError::Io(_)) {
                "Io"
            } else {
                "Config"
            }
        );
    }

    println!("\n3. anyhow: context explains what was happening");
    for path in [
        bad_number.as_path(),
        low_port.as_path(),
        std::path::Path::new("/no/such/config.txt"),
    ] {
        if let Err(error) = start(path) {
            // `{error}` shows only the outermost message;
            // `{error:#}` shows the whole chain on one line.
            println!("    {error:#}");
        }
    }

    println!("\n4. The full report, as printed when main returns an anyhow::Error");
    if let Err(error) = start(&bad_number) {
        // `{:?}` on an anyhow::Error prints the message and a "Caused by:" list.
        for line in format!("{error:?}").lines() {
            println!("    {line}");
        }
    }

    println!("\n5. Getting the original error back: downcast_ref");
    // `downcast_ref` looks through the context layers to the original error.
    if let Err(error) = start(&bad_number)
        && let Some(config_error) = error.downcast_ref::<config::ConfigError>()
    {
        println!("    it was a ConfigError: {config_error:?}");
    }

    println!("\n6. anyhow! and bail!");
    for text in ["40", "140", "forty"] {
        match parse_percent(text) {
            Ok(p) => println!("    {text:>5} → {p}%"),
            Err(e) => println!("    {text:>5} → error: {e:#}"),
        }
    }

    std::fs::remove_file(&bad_number).unwrap();
    std::fs::remove_file(&low_port).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::error::Error as _;

    #[test]
    fn thiserror_generates_display() {
        let error = config::load("port=80\nworkers=99").unwrap_err();
        assert_eq!(
            error.to_string(),
            "setting `workers` is 99, but the maximum is 64"
        );
    }

    #[test]
    fn thiserror_generates_source() {
        let error = config::load("port=x").unwrap_err();
        assert_eq!(
            error.source().unwrap().to_string(),
            "invalid digit found in string"
        );
    }

    #[test]
    fn from_attribute_lets_question_mark_convert() {
        assert!(matches!(
            load_from_file("/no/such/file"),
            Err(StartupError::Io(_))
        ));
    }

    #[test]
    fn anyhow_context_wraps_the_cause() {
        let error = start("/no/such/file").unwrap_err();
        assert_eq!(error.to_string(), "reading config file /no/such/file");
        // The original io::Error is still there, as the cause.
        assert!(error.downcast_ref::<std::io::Error>().is_some());
    }

    #[test]
    fn alternate_format_shows_the_chain() {
        let error = parse_percent("forty").unwrap_err();
        assert_eq!(
            format!("{error:#}"),
            "not a whole number: invalid digit found in string"
        );
    }

    #[test]
    fn anyhow_macro_builds_an_error() {
        assert_eq!(
            parse_percent("140").unwrap_err().to_string(),
            "140 is more than 100%"
        );
        assert_eq!(parse_percent("40").unwrap(), 40);
    }
}
