<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Running things concurrently

## The idea in one sentence

`.await` on its own runs one thing at a time. To overlap work you either **join** several futures inside one task, or **spawn** separate tasks that the runtime runs independently.

## Two tools, two situations

| | `join!` / `try_join!` | `tokio::spawn` / `JoinSet` |
|---|---|---|
| Runs as | several futures inside **your current task** | **separate tasks**, scheduled independently |
| Can use other threads | no: one task is on one thread at a time | yes: tokio may run tasks on different threads |
| Number of futures | fixed, known when you write the code | any number, decided at runtime |
| Can borrow local variables | ✓ | ✗ must own their data (`move`, `'static`) |
| If the caller stops waiting | the futures are cancelled with it | tasks keep running in the background |

## `join!`: all of them, at the same time

```rust
let (a, b, c) = tokio::join!(fetch_weather("Rome"), fetch_weather("Paris"), fetch_weather("Budapest"));
```

All three futures run concurrently, and `join!` waits until **all** are done. The total time is that of the **slowest** one, not the sum:

```text
took 803 ms: as long as the slowest (Budapest, 800 ms), not the sum
```

**`try_join!`** is the version for futures that return `Result`. It stops at the first `Err` and returns it, cancelling the rest.

## `tokio::spawn`: an independent task

```rust
let handle = tokio::spawn(async {
    sleep(Duration::from_millis(200)).await;
    "background work done"
});
// …main keeps going…
let result = handle.await.unwrap();
```

A spawned task **starts immediately** and runs in the background, maybe on another thread. `spawn` returns a `JoinHandle`, and awaiting it gives you the task's result. If the task panics, the panic doesn't crash your code: it comes back as an error from the handle. A test checks this.

### Spawned tasks must own their data

```rust
let name = String::from("Ferris");
tokio::spawn(async { println!("{name}") });          // ✗
```

```text
error[E0373]: async block may outlive the current function, but it borrows `name`,
              which is owned by the current function
```

A spawned task can outlive the function that started it, exactly like a thread (ownership course, lesson 8). The fix is the same: `async move { … }` to give it ownership, or `Arc` to share.

## `JoinSet`: many tasks, results as they finish

```rust
let mut set = JoinSet::new();
for city in cities {
    set.spawn(async move { fetch_weather(city).await });
}
while let Some(result) = set.join_next().await {
    println!("finished: {}", result.unwrap());     // in the order they FINISH
}
```

Use a `JoinSet` when the number of tasks is only known at runtime. `join_next()` hands back results **in the order they finish**, not the order they were started. The demo starts Oslo, Lisbon, Rome, Amsterdam and gets Rome first, because the shortest name is the fastest fake download.

## Concurrency vs parallelism

- **Concurrency:** several tasks *in progress* at the same time, taking turns. One thread is enough. That's `join!`.
- **Parallelism:** several tasks *executing* at the same instant, on different CPU cores. That needs several threads. tokio's default runtime has one worker thread per core, so spawned tasks can run in parallel.

Async is mainly about concurrency: not wasting time while waiting.

## Run it

```bash
cargo run
cargo test
```

The tests use tokio's paused clock, so they check exact timings, such as "join takes exactly 500 ms of virtual time", while finishing instantly.

Previous: [Lesson 1: What async is](../01-what-is-async/) · Next: [Lesson 3: select and timeouts](../03-select-and-timeouts/)
