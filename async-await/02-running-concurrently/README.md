<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Running things concurrently

## The idea in one sentence

**Join futures to overlap operations within one task, or spawn tasks to schedule them independently.**

Ferris can keep several orders in progress while waiting for dishes. Hiring more waiters is a separate choice: concurrency is having several jobs in progress; parallelism is actually executing work at the same time.

Awaiting operation A and then starting B makes them sequential. But if A and B were already spawned, awaiting A's handle first does not prevent B from running.

## Choose how the work is managed

| Tool | Where work runs | Local borrows | When the owner is dropped |
|---|---|---|---|
| `join!` / `try_join!` | several futures in the current task | permitted when valid | owned child futures are dropped |
| `tokio::spawn` | an independent task | requires `Send + 'static` future and output | dropping its `JoinHandle` detaches the task |
| `JoinSet::spawn` | independent tasks tracked by a set | same spawning bounds | dropping the set requests cancellation of its tasks |

`join!` does not make its branches execute in parallel. If one branch blocks while being polled, the others cannot progress either. Spawned tasks may run in parallel on a multi-thread runtime; spawning alone does not guarantee parallel execution.

## join!: collect every answer

Using `fetch_weather` from `src/main.rs`:

```rust
let (rome, paris, budapest) = tokio::join!(
    fetch_weather("Rome"),
    fetch_weather("Paris"),
    fetch_weather("Budapest"),
);
println!("{rome} | {paris} | {budapest}");
```

The simulated waits are 400, 500 and 800 ms. They can overlap, so expect roughly 800 ms plus overhead, rather than 1,700 ms. Results stay in **argument order**, regardless of which operation finishes first.

Our helper uses `city.len()`: the delay counts **UTF-8 bytes**, not visible letters. It is just a convenient rule for short demo names.

`join!` waits for every branch even if one returns `Err`. **`try_join!`** works with `Result` outputs and returns when it observes an error, dropping its remaining owned branch futures:

```rust
let prices = tokio::try_join!(fetch_price("coffee"), fetch_price("yacht"));
assert_eq!(prices, Err(String::from("no price for yacht")));
```

That does not undo earlier work, and dropping a branch that only contains a spawned task's handle does not stop that background task.

## spawn: schedule an independent task

```rust
use std::time::Duration;

let handle = tokio::spawn(async {
    tokio::time::sleep(Duration::from_millis(200)).await;
    "background work done"
});
println!("the caller can continue");
let result = handle.await.expect("demo task should finish successfully");
assert_eq!(result, "background work done");
```

`spawn` schedules the task without waiting for its result. It **does not synchronously poll** the future inside the `spawn` call. Exact start time and execution thread depend on the scheduler and runtime configuration.

`handle.await` returns `Result<T, JoinError>`. Under normal panic-unwinding settings, Tokio reports a task panic through `JoinError`; calling `unwrap()` on that error then panics in the caller. Cancellation can also produce `JoinError`. With `panic = "abort"`, a panic ends the process instead.

A dropped handle leaves its task running while the runtime can drive it. Runtime shutdown can cancel unfinished async tasks. Keep and await handles when completion matters.

## Owned values and static borrows

This example **does not compile**:

```rust
let name = String::from("Ferris");
tokio::spawn(async { println!("{name}") }); // borrows this local String
```

```text
error[E0373]: async block may outlive the current function
```

Give the task ownership:

```rust
let name = String::from("Ferris");
let task = tokio::spawn(async move { format!("hello, {name}") });
assert_eq!(task.await.unwrap(), "hello, Ferris");
```

`'static` means no stored borrow can expire; it does not mean the task must live forever. Suitable `&'static` references, such as string literals, are allowed. `move` transfers or copies captured values; capturing a reference still leaves a reference.

## JoinSet: collect tasks as they complete

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();
for city in ["Oslo", "Lisbon", "Rome", "Amsterdam"] {
    set.spawn(fetch_weather(city)); // city is a &'static str
}
while let Some(result) = set.join_next().await {
    println!("finished: {}", result.unwrap());
}
```

A `JoinSet` tracks any number of tasks with the **same output type**. `join_next()` yields completed task results rather than preserving insertion order. Oslo and Rome each have a 400 ms simulated delay, so either may be reported first. Do not depend on an order for ties.

Unlike dropping a single `JoinHandle`, dropping a `JoinSet` cancels the async tasks it still owns. Use `set.shutdown().await` if you need to request cancellation and wait for the set's tasks to finish shutting down.

## Give the kitchen a capacity limit

Starting a task for every item in a huge list can consume excessive memory or overload a server. The runnable `fetch_with_limit` helper keeps at most `limit` tasks in its set:

```rust
let cities = ["Oslo", "Rome", "Paris"].map(String::from).to_vec();
let results = fetch_with_limit(cities, 2).await;
assert_eq!(results.len(), 3);
```

Before spawning another task when the set is full, it awaits and removes a completed result. Thus at most two lookups can be active here. The helper requires a positive limit and expects successful tasks. The input list and collected results still occupy memory; limiting tasks is not a limit on the whole program's memory.

## Async closures: reusable async work

```rust
async fn compare<T>(a: &str, b: &str, fetch: impl AsyncFn(&str) -> T) -> (T, T) {
    tokio::join!(fetch(a), fetch(b))
}

let note = String::from("checked just now");
let (rome, budapest) = compare("Rome", "Budapest", async |city| {
    format!("{} ({note})", fetch_weather(city).await)
}).await;
println!("{rome} | {budapest}");
```

An `async |city| ...` closure returns a future when called. Here the futures borrow the captured `note` and their `city` arguments.

An ordinary closure returning `async move { ... }` can borrow through references passed into that block; `move` does not require deeply owned data. Async closures additionally support futures borrowing from the closure's **own captured storage**, which ordinary returned async blocks cannot generally express.

| Trait | Calling style |
|---|---|
| `AsyncFn` | through `&self`; returned futures may coexist |
| `AsyncFnMut` | through `&mut self`; lending calls cannot have overlapping mutable borrows |
| `AsyncFnOnce` | takes the closure by value; supports one call |

Rust 1.85 stabilised async closures and these traits. [Release announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)

## Try it

Guess whether Oslo or Rome must appear first, then run:

```bash
cargo run
cargo test
```

[Tokio join](https://docs.rs/tokio/latest/tokio/macro.join.html), [spawn](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html), [JoinHandle](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html), [JoinSet](https://docs.rs/tokio/latest/tokio/task/struct.JoinSet.html)

Previous: [Lesson 1: What async is](../01-what-is-async/) · Next: [Lesson 3: select and timeouts](../03-select-and-timeouts/)
