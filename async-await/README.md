<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Async / Await

Imagine Ferris working as a restaurant waiter. While one table's food is in the oven, Ferris can take another order. **Async programming helps a program use waiting time in a similar way.**

It is especially useful when many operations wait for timers or network responses. It does not make a calculation require less work, and it does not automatically give every operation a new thread.

These five lessons use **Tokio**, a Rust async runtime. The weather lookups, downloads, server replies and kitchen are **simulations**: they use timers or messages, without making network requests. Lesson 5 deliberately includes blocking waits to show a common mistake.

## Ferris's cafe comic

<p align="center">
  <a href="async-await-comic.png">
    <img src="async-await-comic.png" alt="Six-panel Ferris cafe comic about futures, overlapping waits, task handles, timeouts, message channels and blocking work. A text version follows." width="960">
  </a>
</p>

[Open the full-size comic](async-await-comic.png). Read left to right, then down. The pictures are a restaurant analogy; the lessons explain what the Rust program actually does.

<details>
<summary>Read the comic as text, with a few extra details</summary>

1. **A ticket is not coffee.** Ferris says, “I've made a ticket, not coffee!” Calling an `async fn` creates a future without running its body. Polling asks it to make progress and report whether its result is ready. An `.await` may pause the surrounding async code if the operation is pending; an already-ready operation continues immediately.

2. **Overlap the waits.** Ferris can keep two orders in progress while the kettle heats. `tokio::join!` polls its futures within one task and waits for all of them to finish. Their waits can overlap even on one thread. This is **concurrency**; executing instructions simultaneously is **parallelism**.

3. **Keep the task receipt.** `tokio::spawn` schedules a task and returns a `JoinHandle`. Await the handle to observe the task's result, including a possible `JoinError`. Dropping the handle detaches the task; it does not cancel it. A task still needs a running runtime to progress. A `JoinSet` has a different rule: dropping the set aborts its async tasks.

4. **Race a reply and a timer.** Ferris asks, “Which is ready?” `tokio::select!` chooses a ready branch whose result matches its pattern. Its owned losing futures are dropped; borrowing a future can let you keep it for later. Dropping an owned `JoinHandle` still detaches its task. A timeout is **cooperative**: it needs the executor to regain control, so it cannot interrupt blocking code or guarantee a strict wall-clock limit.

5. **Tickets and replies.** `mpsc` allows many senders and one receiver. A `oneshot` channel carries **at most one message**. If its sender is dropped without sending, awaiting its receiver returns an error. “One shot” describes the limit, not a delivery guarantee. [Tokio's oneshot documentation](https://docs.rs/tokio/1.53.1/tokio/sync/oneshot/index.html)

6. **Keep the waiter available.** Ferris says, “No blocking naps on the runtime thread!” Use `tokio::time::sleep` for an async timer. `tokio::task::spawn_blocking` runs blocking work on the blocking pool; once a job starts, calling `abort()` cannot stop it. Keep CPU-heavy work bounded, because the pool and computer have limited resources.

The comic introduces these ideas; it does not show every rule. For example, lesson 4 also explains bounded queues and shared state, and lesson 5 explains locks and local tasks.

Sources for the other captions: [Rust futures](https://doc.rust-lang.org/std/future/trait.Future.html), [Tokio join](https://docs.rs/tokio/1.53.1/tokio/macro.join.html), [task handles](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html), [select](https://docs.rs/tokio/1.53.1/tokio/macro.select.html), [timeouts](https://docs.rs/tokio/1.53.1/tokio/time/fn.timeout.html) and [blocking tasks](https://docs.rs/tokio/1.53.1/tokio/task/fn.spawn_blocking.html).

</details>

## Four words to know

| Word | Restaurant picture | In Rust |
|---|---|---|
| **Future** | an order with work still to do | a value that can be polled for a result |
| **Task** | one ongoing assignment | an independently scheduled async computation |
| **Thread** | a worker who can execute instructions | an operating-system thread |
| **Runtime** | the scheduling system and readiness notifications | drives tasks and provides services such as timers and network I/O |

Calling an `async fn` creates a future without running its body. `.await` polls the operation: if it is ready, execution continues; if it returns `Pending`, the current task can suspend. **An `.await` is a possible pause, not a promise to switch tasks.**

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [What async is](01-what-is-async/) | lazy async bodies, polling, wakeups, runtimes and overlapping waits |
| 2 | [Running things concurrently](02-running-concurrently/) | `join!`, `try_join!`, spawning, `JoinSet`, async closures and limiting tasks |
| 3 | [select and timeouts](03-select-and-timeouts/) | choosing a ready branch, cooperative deadlines, cancellation and shutdown |
| 4 | [Channels and shared state](04-channels-and-shared-state/) | message queues, replies, channel closure, backpressure and mutexes |
| 5 | [Async pitfalls](05-async-pitfalls/) | blocking, locks, `Send`, local tasks, recursive futures and forgotten awaits |

Read them in order. If ownership and borrowing are new, start with the [ownership course](../ownership-and-borrowing/).

## Run a lesson

From the repository root:

```bash
cargo run -p what-is-async
cargo test -p what-is-async
```

Or enter a lesson:

```bash
cd async-await/02-running-concurrently
cargo run
cargo test
```

| Folder | Cargo package |
|---|---|
| `01-what-is-async` | `what-is-async` |
| `02-running-concurrently` | `running-concurrently` |
| `03-select-and-timeouts` | `select-and-timeouts` |
| `04-channels-and-shared-state` | `async-channels-and-shared-state` |
| `05-async-pitfalls` | `async-pitfalls` |

Cargo reads each lesson's `Cargo.toml`. The examples use Rust 2024 and Tokio 1.x; this review checked Rust **1.99** with the locked Tokio **1.53.1** dependency. Printed timings and the order of equally fast tasks can vary.

Rust blocks that use lesson helpers belong inside an async function unless they show a complete program. Examples marked **does not compile** deliberately demonstrate an error. The complete runnable programs are in each `src/main.rs`.

## Common diagnostics

| Message | What to check | Lesson |
|---|---|---|
| unused implementer of `Future` that must be used | an async operation may have been created and discarded instead of awaited | 1, 5 |
| **E0728** `await` is only allowed inside `async` functions and blocks | move the await into async code | 1 |
| **E0752** `main` function is not allowed to be `async` | use a normal main that drives a future, or a runtime macro | 1 |
| **E0373** async block may outlive the current function | a spawned task borrows a local value; consider `async move` | 2 |
| future cannot be sent between threads safely | `tokio::spawn` needs a `Send` future and output | 5 |
| **E0733** recursion in an async fn requires boxing | introduce indirection for the recursive future | 5 |

Exact diagnostic wording can change between compiler releases. Commented error examples include the context needed to try them; when replacing a function, disable its working definition to avoid a duplicate-name error.

## Tests use a pretend clock, not a faster computer

Several tests use `#[tokio::test(start_paused = true)]`. When the runtime has no runnable work, it can advance **Tokio's clock** to the next timer. That makes timer examples fast and predictable without waiting hundreds of real milliseconds.

This does **not** pause `std::time::Instant`, speed up calculations or skip `std::thread::sleep`. Lesson 5 uses coordination between tasks to test responsiveness without guessing how fast the computer should be.

## A little history

Rust stabilised `async`/`.await` in **Rust 1.39, on 7 November 2019**. Async closures and the `AsyncFn` family followed in **Rust 1.85, on 20 February 2025**. Before convenient async syntax, programmers could build the same kinds of systems by arranging callbacks and futures more explicitly. [Rust 1.39 announcement](https://blog.rust-lang.org/2019/11/07/Rust-1.39.0/), [Rust 1.85 announcement](https://blog.rust-lang.org/2025/02/20/Rust-1.85.0/)
