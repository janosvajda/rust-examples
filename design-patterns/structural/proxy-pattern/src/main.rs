// Proxy pattern: a currency exchange-rate service is slow, so we put a
// caching proxy in front of it. The proxy looks exactly like the real
// service to the rest of the program, but answers repeated questions from
// its cache instead of asking the slow service again.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

/// What the rest of the program uses. Both the real service and the proxy
/// implement it, so they can be swapped without changing any other code.
trait ExchangeRates {
    /// How many units of `to` you get for one unit of `from`.
    fn rate(&self, from: &str, to: &str) -> Option<f64>;
}

// ---- The real subject: slow -------------------------------------------------

/// Pretends to call a remote service. Every call takes a while.
struct RemoteRateService {
    // Counts how many times the slow service was actually called.
    // `Cell` lets us update it through `&self`.
    calls: Cell<u32>,
}

impl RemoteRateService {
    fn new() -> Self {
        RemoteRateService { calls: Cell::new(0) }
    }
}

impl ExchangeRates for RemoteRateService {
    fn rate(&self, from: &str, to: &str) -> Option<f64> {
        self.calls.set(self.calls.get() + 1);
        thread::sleep(Duration::from_millis(200)); // the network is slow

        match (from, to) {
            ("EUR", "HUF") => Some(395.0),
            ("EUR", "USD") => Some(1.09),
            ("USD", "HUF") => Some(362.4),
            _ => None,
        }
    }
}

// ---- The proxy: same interface, adds caching --------------------------------

/// Stands in front of any `ExchangeRates` implementation and remembers answers.
struct CachingProxy<S: ExchangeRates> {
    service: S,
    // `RefCell` because `rate` takes `&self` (it's a read, as far as callers
    // are concerned), but the proxy needs to write to its cache.
    cache: RefCell<HashMap<(String, String), Option<f64>>>,
}

impl<S: ExchangeRates> CachingProxy<S> {
    fn new(service: S) -> Self {
        CachingProxy {
            service,
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl<S: ExchangeRates> ExchangeRates for CachingProxy<S> {
    fn rate(&self, from: &str, to: &str) -> Option<f64> {
        let key = (from.to_string(), to.to_string());

        // Already asked before? Answer immediately.
        if let Some(&cached) = self.cache.borrow().get(&key) {
            return cached;
        }

        // First time: ask the real service, then remember the answer.
        let answer = self.service.rate(from, to);
        self.cache.borrow_mut().insert(key, answer);
        answer
    }
}

// ---- Code that doesn't know (or care) whether it has a proxy ---------------

fn convert(rates: &impl ExchangeRates, amount: f64, from: &str, to: &str) -> Option<f64> {
    rates.rate(from, to).map(|rate| amount * rate)
}

fn main() {
    let rates = CachingProxy::new(RemoteRateService::new());

    let requests = [
        (100.0, "EUR", "HUF"),
        (50.0, "EUR", "HUF"), // same currencies: served from the cache
        (20.0, "EUR", "USD"),
        (10.0, "EUR", "HUF"), // cached again
        (5.0, "GBP", "JPY"),  // unknown: the "no answer" gets cached too
        (7.0, "GBP", "JPY"),
    ];

    for (amount, from, to) in requests {
        let start = Instant::now();
        let result = convert(&rates, amount, from, to);
        let elapsed = start.elapsed().as_millis();
        match result {
            Some(value) => println!("{amount:>6.2} {from} = {value:>9.2} {to}   ({elapsed} ms)"),
            None => println!("{amount:>6.2} {from} → {to}: no rate available   ({elapsed} ms)"),
        }
    }

    println!(
        "\n{} requests, but the slow service was only called {} times.",
        requests.len(),
        rates.service.calls.get()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fast fake service for tests, so they don't sleep.
    struct FakeService {
        calls: Cell<u32>,
    }

    impl ExchangeRates for FakeService {
        fn rate(&self, from: &str, _to: &str) -> Option<f64> {
            self.calls.set(self.calls.get() + 1);
            (from == "EUR").then_some(2.0)
        }
    }

    fn proxy() -> CachingProxy<FakeService> {
        CachingProxy::new(FakeService { calls: Cell::new(0) })
    }

    #[test]
    fn repeated_questions_hit_the_service_once() {
        let rates = proxy();
        for _ in 0..5 {
            assert_eq!(rates.rate("EUR", "HUF"), Some(2.0));
        }
        assert_eq!(rates.service.calls.get(), 1);
    }

    #[test]
    fn different_questions_are_cached_separately() {
        let rates = proxy();
        rates.rate("EUR", "HUF");
        rates.rate("EUR", "USD");
        rates.rate("EUR", "HUF");
        assert_eq!(rates.service.calls.get(), 2);
    }

    #[test]
    fn missing_answers_are_cached_too() {
        let rates = proxy();
        assert_eq!(rates.rate("XYZ", "HUF"), None);
        assert_eq!(rates.rate("XYZ", "HUF"), None);
        assert_eq!(rates.service.calls.get(), 1);
    }

    #[test]
    fn proxy_and_real_service_are_interchangeable() {
        // `convert` accepts either one.
        let direct = FakeService { calls: Cell::new(0) };
        let cached = proxy();
        assert_eq!(convert(&direct, 10.0, "EUR", "HUF"), Some(20.0));
        assert_eq!(convert(&cached, 10.0, "EUR", "HUF"), Some(20.0));
    }
}
