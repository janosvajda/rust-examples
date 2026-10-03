// Singleton pattern: the app's configuration is loaded once, the first time
// anything asks for it, and every part of the program shares that one copy.
// A second example shows a global ID generator that many threads use at once.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;

// ---- Example 1: configuration, created once on first use -----------------

#[derive(Debug)]
struct AppConfig {
    app_name: String,
    max_connections: u32,
    debug: bool,
}

impl AppConfig {
    /// Builds the configuration from a lookup function. Taking the lookup as
    /// a parameter (instead of reading environment variables directly) lets
    /// the tests feed in their own values.
    fn load(lookup: impl Fn(&str) -> Option<String>) -> Self {
        println!("  (loading configuration… this should appear only once)");
        AppConfig {
            app_name: lookup("APP_NAME").unwrap_or_else(|| "rust-examples".to_string()),
            max_connections: lookup("MAX_CONNECTIONS")
                .and_then(|value| value.parse().ok())
                .unwrap_or(10),
            debug: lookup("DEBUG").is_some_and(|value| value == "1"),
        }
    }
}

/// The one shared configuration.
///
/// `OnceLock` starts empty. The first call to `get_or_init` runs the closure
/// and stores the result. Every later call, from any thread, gets the same
/// stored value. If two threads call it at the same moment, only one of them
/// runs the closure; the other waits for it.
static CONFIG: OnceLock<AppConfig> = OnceLock::new();

/// The only way to get the configuration. `&'static` means the reference is
/// valid for the whole life of the program.
fn config() -> &'static AppConfig {
    CONFIG.get_or_init(|| AppConfig::load(|key| std::env::var(key).ok()))
}

// ---- Example 2: a global counter that many threads share -----------------

/// Hands out unique request IDs: 1, 2, 3, …
///
/// A plain `static mut` counter would be unsafe: two threads could read the
/// same value at the same time. An atomic integer does "read, add one, write
/// back" as one indivisible step, so every caller gets a different number.
static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_request_id() -> u64 {
    // `fetch_add` returns the old value and increments in one step.
    // `Relaxed` is enough: we only need each number to be unique, not to
    // order this with other memory operations.
    NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
}

fn main() {
    println!("Before first use: nothing loaded yet.");

    println!("\nFirst call to config():");
    println!("  app name: {}", config().app_name);

    println!("\nLater calls reuse the same instance:");
    println!("  max connections: {}", config().max_connections);
    println!("  debug: {}", config().debug);

    // Four threads ask for the config and for request IDs at the same time.
    println!("\nFour threads at once:");
    let handles: Vec<_> = (0..4)
        .map(|worker| {
            thread::spawn(move || {
                let id = next_request_id();
                // `{:p}` prints the address: the same for every thread,
                // because they all share one AppConfig.
                println!("  worker {worker}: request #{id}, config at {:p}", config());
                id
            })
        })
        .collect();

    let mut ids: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    ids.sort();
    println!("  request IDs handed out: {ids:?} (all different)");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    #[test]
    fn config_is_the_same_instance_every_time() {
        let first = config();
        let second = config();
        // Same address in memory, not just equal values.
        assert!(std::ptr::eq(first, second));
    }

    #[test]
    fn config_is_shared_across_threads() {
        let main_thread = config() as *const AppConfig as usize;
        let other_thread = thread::spawn(|| config() as *const AppConfig as usize)
            .join()
            .unwrap();
        assert_eq!(main_thread, other_thread);
    }

    #[test]
    fn load_reads_values_and_falls_back_to_defaults() {
        let values = HashMap::from([("APP_NAME", "shop"), ("DEBUG", "1")]);
        let loaded = AppConfig::load(|key| values.get(key).map(|v| v.to_string()));
        assert_eq!(loaded.app_name, "shop");
        assert_eq!(loaded.max_connections, 10); // not set: default
        assert!(loaded.debug);
    }

    #[test]
    fn request_ids_are_unique_across_threads() {
        let handles: Vec<_> = (0..8)
            .map(|_| thread::spawn(|| (0..1000).map(|_| next_request_id()).collect::<Vec<_>>()))
            .collect();
        let mut all = HashSet::new();
        for handle in handles {
            for id in handle.join().unwrap() {
                assert!(all.insert(id), "id {id} was handed out twice");
            }
        }
        assert_eq!(all.len(), 8000);
    }
}
