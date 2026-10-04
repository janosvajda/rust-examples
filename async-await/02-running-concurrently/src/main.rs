// Lesson 2: running things concurrently.
//
// `.await` on its own runs one thing at a time. To overlap work you either
//   - join several futures inside ONE task:   tokio::join!, try_join!
//   - spawn separate TASKS the runtime runs independently: tokio::spawn, JoinSet

use std::time::Duration;
use tokio::task::JoinSet;
use tokio::time::{Instant, sleep};

/// Pretends to fetch the weather for a city. Some cities are slower.
async fn fetch_weather(city: &str) -> String {
    let millis = 100 * city.len() as u64;
    sleep(Duration::from_millis(millis)).await;
    format!("{city}: sunny")
}

/// Like fetch_weather, but can fail.
async fn fetch_price(item: &str) -> Result<u32, String> {
    sleep(Duration::from_millis(100)).await;
    match item {
        "coffee" => Ok(250),
        "cake" => Ok(400),
        _ => Err(format!("no price for {item}")),
    }
}

/// Runs the same async operation for two inputs at once. `impl AsyncFn(&str) -> T`
/// accepts an async closure. Because an `AsyncFn` is called through `&self`,
/// it can be called twice and both calls can run concurrently.
async fn compare<T>(a: &str, b: &str, fetch: impl AsyncFn(&str) -> T) -> (T, T) {
    tokio::join!(fetch(a), fetch(b))
}

fn ms(start: Instant) -> u128 {
    start.elapsed().as_millis()
}

#[tokio::main]
async fn main() {
    println!("1. join!: run several futures at once, wait for all");
    let start = Instant::now();
    let (a, b, c) = tokio::join!(fetch_weather("Rome"), fetch_weather("Paris"), fetch_weather("Budapest"));
    println!("    {a} | {b} | {c}");
    println!("    took {} ms: as long as the slowest (Budapest, 800 ms), not the sum", ms(start));

    println!("\n2. try_join!: stop at the first error");
    let ok = tokio::try_join!(fetch_price("coffee"), fetch_price("cake"));
    let failed = tokio::try_join!(fetch_price("coffee"), fetch_price("yacht"));
    println!("    {ok:?}");
    println!("    {failed:?}");

    println!("\n3. tokio::spawn: an independent task");
    // A spawned task starts running immediately, in the background, and may
    // run on another thread. `spawn` returns a JoinHandle to get its result.
    let handle = tokio::spawn(async {
        sleep(Duration::from_millis(200)).await;
        "background work done"
    });
    println!("    main keeps going while the task runs…");
    let result = handle.await.expect("the task panicked");
    println!("    {result}");

    println!("\n4. JoinSet: a dynamic number of tasks, results as they finish");
    let cities = ["Oslo", "Lisbon", "Rome", "Amsterdam"];
    let mut set = JoinSet::new();
    for city in cities {
        // `move` and owned data: the task may outlive this loop iteration,
        // so it can't borrow anything temporary.
        set.spawn(async move { fetch_weather(city).await });
    }
    while let Some(result) = set.join_next().await {
        println!("    finished: {}", result.unwrap()); // shortest names finish first
    }

    println!("\n5. Spawned tasks can't borrow local variables");
    let name = String::from("Ferris");
    // tokio::spawn(async { println!("{name}") });
    //     error[E0373]: async block may outlive the current function, but it borrows `name`,
    //                   which is owned by the current function
    let task = tokio::spawn(async move { format!("hello, {name}") }); // move ownership in
    println!("    {}", task.await.unwrap());

    println!("\n6. Async closures: pass async work to a function");
    let note = String::from("checked just now");
    let start = Instant::now();
    // The async closure borrows `note` from here, and its argument `city` too.
    let (rome, budapest) = compare("Rome", "Budapest", async |city| {
        format!("{} ({note})", fetch_weather(city).await)
    })
    .await;
    println!("    {rome} | {budapest}");
    println!("    took {} ms: both calls ran at the same time", ms(start));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(start_paused = true)]
    async fn join_takes_as_long_as_the_slowest() {
        let start = Instant::now();
        tokio::join!(fetch_weather("ab"), fetch_weather("abcde"));
        assert_eq!(start.elapsed(), Duration::from_millis(500));
    }

    #[tokio::test(start_paused = true)]
    async fn try_join_returns_the_error() {
        let result = tokio::try_join!(fetch_price("coffee"), fetch_price("boat"));
        assert_eq!(result, Err(String::from("no price for boat")));
    }

    #[tokio::test(start_paused = true)]
    async fn joinset_yields_results_in_finishing_order() {
        let mut set = JoinSet::new();
        for city in ["Amsterdam", "Rome", "Lisbon"] {
            set.spawn(fetch_weather(city));
        }
        let mut order = Vec::new();
        while let Some(result) = set.join_next().await {
            order.push(result.unwrap());
        }
        assert_eq!(order, ["Rome: sunny", "Lisbon: sunny", "Amsterdam: sunny"]);
    }

    #[tokio::test(start_paused = true)]
    async fn async_closure_runs_twice_concurrently() {
        let suffix = String::from("!");
        let start = Instant::now();
        let results = compare("ab", "abcde", async |city| fetch_weather(city).await + &suffix).await;
        assert_eq!(results, (String::from("ab: sunny!"), String::from("abcde: sunny!")));
        assert_eq!(start.elapsed(), Duration::from_millis(500)); // the slower one, not the sum
    }

    #[tokio::test]
    async fn spawned_task_returns_its_value() {
        let handle = tokio::spawn(async { 6 * 7 });
        assert_eq!(handle.await.unwrap(), 42);
    }

    #[tokio::test]
    async fn a_panicking_task_does_not_crash_the_caller() {
        let handle = tokio::spawn(async { panic!("boom") });
        // The panic is caught by tokio and returned as an error.
        assert!(handle.await.unwrap_err().is_panic());
    }
}
