<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: What async is

## The idea in one sentence

**Async** lets one thread work on many tasks at once by switching between them whenever one is **waiting**, for a timer, the network or a file, instead of sitting idle.

## An everyday picture

A waiter in a restaurant doesn't stand next to the kitchen waiting for one table's food before taking the next order. They take an order, hand it to the kitchen, take the next table's order, bring drinks, and come back when a dish is ready. One waiter, many tables, because most of each table's time is spent *waiting*.

A program that downloads ten web pages spends almost all its time waiting for the network. Async lets one thread handle all ten downloads while they wait, instead of needing ten threads that each sit blocked.

## The three pieces

| Piece | What it is |
|---|---|
| **`async fn`** | a function that returns a **future** instead of running immediately |
| **future** | a value describing work that can pause and resume, i.e. "the coffee, once it's made" |
| **`.await`** | "run this future until it's done; while it waits, let other tasks run" |
| **runtime** (tokio) | the engine that runs futures: it polls them and wakes them up when what they wait for is ready |

## Futures are lazy

```rust
let future = make_coffee();   // nothing happens yet: no "making coffee…"
let coffee = future.await;    // NOW it runs
```

Calling an `async fn` only **creates** the future. It doesn't start the work. Forgetting `.await` is a classic mistake, and the compiler warns you:

```text
warning: unused implementer of `Future` that must be used
  = note: futures do nothing unless you `.await` or poll them
```

`.await` only works inside an `async` function or block:

```text
error[E0728]: `await` is only allowed inside `async` functions and blocks
```

## Under the hood: polling

A future is anything implementing the `Future` trait, which has one method, `poll`. The runtime calls `poll` repeatedly, and each time the future answers:

- **`Poll::Ready(value)`**: done, here's the result;
- **`Poll::Pending`**: not yet. The future has arranged to be woken up when it can make progress, and the runtime will poll it again then.

```text
  runtime ──poll──► future: Pending  (waiting for the timer; wake me when it fires)
     … the runtime runs other tasks meanwhile …
  timer fires ──wake──► runtime ──poll──► future: Ready("coffee")
```

The demo includes a hand-written future, `CountToThree`, that prints each poll and is ready on the third. You'll never need to write one yourself: `async fn` turns your code into a future like this automatically, splitting it at every `.await`.

## You need a runtime

Rust's standard library defines futures but doesn't include a runtime to run them. You choose one. **tokio** is by far the most widely used:

```rust
#[tokio::main]
async fn main() { … }
```

`#[tokio::main]` starts a tokio runtime and runs `main` on it. Without a runtime, `main` can't be async:

```text
error[E0752]: `main` function is not allowed to be `async`
```

## Why it's worth it

```text
sequential: page (300 ms), image (300 ms) took 604 ms
concurrent: page (300 ms), image (300 ms) took 301 ms
```

(One real run. The exact milliseconds vary slightly every time.) Awaiting one download, then the other, takes about 600 ms. `tokio::join!` runs both futures at once, so their waits overlap and it takes 300 ms, **on a single thread**. The tests check the exact timings with tokio's paused clock.

## Async or threads?

| | Threads | Async |
|---|---|---|
| Best for | **CPU-heavy** work: calculations, compression | **waiting-heavy** work: network, files, timers |
| Cost per task | a whole OS thread, with its own stack (often megabytes) | a small future, usually bytes to kilobytes |
| How many at once | thousands | hundreds of thousands |
| Switching | the operating system decides, at any moment | the task decides, only at `.await` |

## Run it

```bash
cargo run
cargo test
```

Next: [Lesson 2: Running things concurrently](../02-running-concurrently/)
