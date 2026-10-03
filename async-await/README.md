<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Async / Await

<p align="center">
  <a href="async-await-comic.png">
    <img src="async-await-comic.png" alt="Comic poster summarising the five async / await lessons" width="100%">
  </a>
</p>

Async Rust lets one thread juggle thousands of tasks that spend most of their time **waiting**, for the network, timers or files, without a thread per task. This course explains how it works, from what a future is to the mistakes everyone makes at first. It uses **tokio**, the most widely used async runtime.

Every compiler error and warning quoted in these READMEs is the real output of Rust 1.99. Timings come from real runs. The tests use tokio's paused clock, so they check exact timings and still finish instantly.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [What async is](01-what-is-async/) | futures, laziness, `.await`, polling, why you need a runtime, async vs threads |
| 2 | [Running things concurrently](02-running-concurrently/) | `join!`, `try_join!`, `tokio::spawn`, `JoinSet`, concurrency vs parallelism |
| 3 | [select and timeouts](03-select-and-timeouts/) | racing futures, deadlines, stop signals, cancellation by dropping |
| 4 | [Channels and shared state](04-channels-and-shared-state/) | `mpsc`, `oneshot`, backpressure, `Arc<Mutex<T>>`, `std` vs `tokio` mutex |
| 5 | [Async pitfalls](05-async-pitfalls/) | blocking the runtime, locks across `.await`, `Send`, recursion, forgotten `.await` |

## The errors you'll meet

| Message | In plain words | Lesson |
|---|---|---|
| unused implementer of `Future` that must be used | you forgot `.await`; nothing ran | 1, 5 |
| **E0728** `await` is only allowed inside `async` functions and blocks | make the function `async` | 1 |
| **E0752** `main` function is not allowed to be `async` | add `#[tokio::main]` | 1 |
| **E0373** async block may outlive the current function | a spawned task borrows local data; use `async move` | 2 |
| future cannot be sent between threads safely | a lock guard or `Rc` held across `.await` in a spawned task | 5 |
| **E0733** recursion in an async fn requires boxing | use `Box::pin(…)` for the recursive call | 5 |

## Run a lesson

```bash
cd async-await/02-running-concurrently
cargo run
cargo test
```

Or from the repository root: `cargo run -p running-concurrently`. The package names are in each lesson's `Cargo.toml`.
