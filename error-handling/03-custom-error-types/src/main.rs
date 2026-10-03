// Lesson 3: custom error types.
//
// `String` and `Box<dyn Error>` say THAT something failed. A custom error
// type, usually an enum, also says WHAT failed, so callers can react to each
// case differently. Three things make a good error type:
//   1. an enum with one variant per kind of failure, carrying useful details;
//   2. `Display`, for the message a person reads;
//   3. `std::error::Error`, so it works with `?`, `Box<dyn Error>` and the
//      rest of the ecosystem, including `source()` for the underlying cause.

use std::error::Error;
use std::fmt;
use std::num::ParseIntError;

/// Everything that can go wrong while loading a server config.
/// `PartialEq` lets tests compare errors; every field type supports it.
#[derive(Debug, PartialEq)]
enum ConfigError {
    /// A required setting wasn't there at all.
    Missing { key: String },
    /// A setting was there, but wasn't a number. Keeps the original
    /// ParseIntError as the underlying cause.
    NotANumber { key: String, source: ParseIntError },
    /// A number, but outside the allowed range.
    OutOfRange { key: String, value: u32, max: u32 },
}

/// The message for people. One line, no debugging details.
impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Missing { key } => write!(f, "setting `{key}` is missing"),
            ConfigError::NotANumber { key, .. } => write!(f, "setting `{key}` must be a number"),
            ConfigError::OutOfRange { key, value, max } => {
                write!(f, "setting `{key}` is {value}, but the maximum is {max}")
            }
        }
    }
}

/// Makes it a "real" error. `source` exposes the cause, so tools can print
/// the whole chain: "must be a number" ← "invalid digit found in string".
impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::NotANumber { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq)]
struct ServerConfig {
    port: u32,
    workers: u32,
}

/// Looks up `key` in "key=value" lines and parses it, with precise errors.
fn get_number(text: &str, key: &str, max: u32) -> Result<u32, ConfigError> {
    let raw = text
        .lines()
        .filter_map(|line| line.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| v.trim())
        .ok_or_else(|| ConfigError::Missing { key: key.to_string() })?;

    let value: u32 = raw.parse().map_err(|source| ConfigError::NotANumber {
        key: key.to_string(),
        source,
    })?;

    if value > max {
        return Err(ConfigError::OutOfRange { key: key.to_string(), value, max });
    }
    Ok(value)
}

fn load_config(text: &str) -> Result<ServerConfig, ConfigError> {
    Ok(ServerConfig {
        port: get_number(text, "port", 65_535)?,
        workers: get_number(text, "workers", 64)?,
    })
}

// ---- `From`: letting `?` convert errors automatically ----------------------------

/// The application's top-level error: wraps the errors of its parts.
#[derive(Debug)]
enum AppError {
    Config(ConfigError),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Config(_) => write!(f, "invalid configuration"),
            AppError::Io(_) => write!(f, "could not read the configuration file"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Config(e) => Some(e),
            AppError::Io(e) => Some(e),
        }
    }
}

/// With these two impls, `?` turns a ConfigError or io::Error into an
/// AppError on its own: `?` always calls `From::from` on the error.
impl From<ConfigError> for AppError {
    fn from(error: ConfigError) -> Self {
        AppError::Config(error)
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::Io(error)
    }
}

fn start_server(path: &str) -> Result<ServerConfig, AppError> {
    let text = std::fs::read_to_string(path)?; // io::Error → AppError::Io
    let config = load_config(&text)?; // ConfigError → AppError::Config
    Ok(config)
}

/// Prints an error and every cause behind it, following `source()`.
fn report(error: &dyn Error) -> String {
    let mut message = error.to_string();
    let mut cause = error.source();
    while let Some(inner) = cause {
        message.push_str(&format!("\n        caused by: {inner}"));
        cause = inner.source();
    }
    message
}

fn main() {
    println!("1. Each failure is a different variant");
    let inputs = [
        "port=8080\nworkers=8",
        "port=8080",
        "port=eighty\nworkers=8",
        "port=8080\nworkers=500",
    ];
    for text in inputs {
        match load_config(text) {
            Ok(config) => println!("    ok: {config:?}"),
            Err(error) => println!("    error: {error}"),
        }
    }

    println!("\n2. Callers can react to specific cases");
    let workers = match load_config("port=80\nworkers=500") {
        Ok(config) => config.workers,
        Err(ConfigError::OutOfRange { max, .. }) => max, // clamp instead of failing
        Err(other) => panic!("can't continue: {other}"),
    };
    println!("    too many workers requested, using the maximum: {workers}");

    println!("\n3. `?` converts errors with From, and source() keeps the chain");
    let path = std::env::temp_dir().join("rust-examples-config.txt");
    std::fs::write(&path, "port=eighty\nworkers=4").unwrap();
    for file in [path.to_str().unwrap(), "/no/such/config.txt"] {
        if let Err(error) = start_server(file) {
            println!("    error: {}", report(&error));
        }
    }
    std::fs::remove_file(&path).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_config() {
        assert_eq!(
            load_config("port = 80\nworkers = 2"),
            Ok(ServerConfig { port: 80, workers: 2 })
        );
    }

    #[test]
    fn each_problem_is_its_own_variant() {
        assert!(matches!(load_config("workers=2"), Err(ConfigError::Missing { key }) if key == "port"));
        assert!(matches!(
            load_config("port=x\nworkers=2"),
            Err(ConfigError::NotANumber { .. })
        ));
        assert!(matches!(
            load_config("port=80\nworkers=99"),
            Err(ConfigError::OutOfRange { value: 99, max: 64, .. })
        ));
    }

    #[test]
    fn display_message_is_human_readable() {
        let error = load_config("port=80\nworkers=99").unwrap_err();
        assert_eq!(error.to_string(), "setting `workers` is 99, but the maximum is 64");
    }

    #[test]
    fn source_points_to_the_original_cause() {
        let error = load_config("port=x").unwrap_err();
        let cause = error.source().expect("NotANumber has a source");
        assert_eq!(cause.to_string(), "invalid digit found in string");
    }

    #[test]
    fn question_mark_converts_with_from() {
        let error = start_server("/no/such/file").unwrap_err();
        assert!(matches!(error, AppError::Io(_)));
    }

    #[test]
    fn report_walks_the_whole_chain() {
        let error = AppError::from(load_config("port=x").unwrap_err());
        let text = report(&error);
        assert!(text.starts_with("invalid configuration"));
        assert!(text.contains("caused by: setting `port` must be a number"));
        assert!(text.contains("caused by: invalid digit found in string"));
    }
}
