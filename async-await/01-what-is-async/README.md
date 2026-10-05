<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: What async is

## The idea in one sentence

**Async lets a task pause while waiting, so its thread can work on other runnable tasks.**

## Ferris's restaurant

Ferris takes an order, passes it to the kitchen and serves another table while the food cooks. Ferris does not need a second waiter just to stand beside every oven.

Similarly, one thread can handle several network operations while the network is doing the waiting. This helps with many waiting operations; it does not make one thread perform several calculations at the same instant.

## From an async function to a result

This complete program works with this lesson's Tokio dependency:

```rust
use std::time::Duration;

async fn make_coffee() -> &'static str {
    println!("making coffee");
    tokio::time::sleep(Duration::from_millis(300)).await;
    "coffee"
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let future = make_coffee(); // body has not run
    println!("order created");
    let coffee = future.await; // poll the future until it completes
    println!("got {coffee}");
}
```

The messages appear in this order: `order created`, `making coffee`, `got coffee`.

| Piece | Job |
|---|---|
| `async fn` | calling it creates a future; its body runs when polled |
| `Future` | an interface for checking progress toward a result |
| `.await` | polls the operation and may pause the surrounding async code if it is pending |
| runtime | schedules tasks and supplies timer/I/O services |

Only the **body** waits: the function's arguments are still calculated when you make the call. (And not every future is unstarted work: the `JoinHandle` you get from `tokio::spawn`, lesson 2, stands for a task that's already running.)

Discarding the future from `make_coffee()` without polling it means the body never makes coffee:

```rust
make_coffee(); // deliberate warning: unused future; body never runs
```

```text
warning: unused implementer of `Future` that must be used
```

When you need the result, await it or use a tool such as `join!` or `spawn` to drive the operation.

## An await is not always a pause

```rust
let answer = std::future::ready(42).await;
assert_eq!(answer, 42); // ready immediately; this await need not suspend
```

If the awaited value is already ready, the code simply continues, without pausing. Only when it has to wait (`Pending`) does the task pause and let the runtime run other tasks. So adding `.await`s to a long calculation doesn't automatically give other tasks a turn: if everything it awaits is ready, it never pauses at all.

## Under the hood: polling and the doorbell

A future's `poll` method returns one of two answers:

- **`Poll::Ready(value)`:** the result is available.
- **`Poll::Pending`:** not ready; arrange a wakeup so the task can be polled again.

The **waker** is like the kitchen's doorbell. It tells the scheduler that a task should be checked again; it does not itself execute the task or promise immediate service.

```text
poll coffee → Pending
    the thread can work on other ready tasks
timer becomes ready → wake the waiting task
poll coffee → Ready("coffee")
```

The runtime doesn't keep asking a waiting future "ready yet?" in a loop; that would waste the CPU. It waits for the doorbell: a timer expiring, or data arriving on a network connection. A ring means "check again", not "it's definitely ready".

The runnable [implementation](src/main.rs) includes a **`Timed<F>` wrapper** that measures an async operation. Using its `Timed` and `make_coffee` definitions inside async code:

```rust
let (coffee, elapsed) = Timed::new(make_coffee()).await;
println!("{coffee} ready after {} ms", elapsed.as_millis());
```

How `Timed` works:

- `Timed::new` only stores the inner future; nothing runs yet.
- The first time `Timed` is polled, it starts the clock. Then it polls the inner future and passes on its answer.
- If the inner future says `Pending`, `Timed` says `Pending` too. The inner future has already arranged its own wakeup (here, Tokio's timer rings the doorbell), so `Timed` doesn't need to.
- When the inner future says `Ready(output)`, `Timed` returns the output together with the elapsed time.

The time is measured with **Tokio's clock**, from the first poll to the result, including all the waiting: it isn't CPU time. (Using Tokio's clock lets the tests pause time, see the course overview.)

The inner future is stored as `Pin<Box<F>>`. Some futures must not move in memory once they've been polled (they can contain references into themselves), and **pinning** promises they won't. Putting the future in a `Box` and pinning it there is the simplest safe way. [Pinning a boxed future](https://doc.rust-lang.org/std/pin/struct.Pin.html#pinning-a-value-inside-a-box)

Usually you write `async fn` and let the compiler create the state needed to resume your function. You can write `Future` implementations when building lower-level libraries. After a future returns `Ready`, its caller must not poll it again.

## Who starts the runtime?

Rust's standard library defines `Future` but does not provide a general async executor or Tokio's timers. These examples choose Tokio.

The `#[tokio::main(...)]` macro creates a **normal synchronous entry point** that builds a runtime and drives the async body. A bare entry point **does not compile**:

```rust
async fn main() {}
```

```text
error[E0752]: `main` function is not allowed to be `async`
```

You can also build a runtime manually and call `block_on`, as lesson 5 does. Simple futures can run on other executors; a Tokio timer specifically needs Tokio's timer support.

## Waiting one after another or together

Using `download` from the runnable lesson:

```rust
let a = download("page", 300).await;
let b = download("image", 300).await; // starts after a finishes
println!("{a}, {b}");
```

```rust
let (a, b) = tokio::join!(download("page", 300), download("image", 300));
println!("{a}, {b}"); // both timer waits can overlap
```

These are fake downloads implemented with sleeps. Expect roughly **600 ms** of sequential waiting or **300 ms** with overlapping waits, plus scheduling and printing overhead. The demo explicitly uses a current-thread runtime: overlapping waits do not require parallel execution.

## Async and threads work together

| Question | Useful distinction |
|---|---|
| Many operations waiting for network responses? | async tasks can share a smaller number of threads |
| Long calculations? | use suitable CPU worker threads or a CPU pool |
| Does a future have an OS thread's stack? | no separate OS thread stack per future; its state still uses memory |
| Does async mean parallel? | no; parallel execution requires available execution resources |

There is no universal maximum task count or fixed future size. Both depend on the work, stored data, runtime and computer. Ordinary file operations may still use blocking threads behind an async interface.

## Try it

Predict which message prints before the coffee starts, then run:

```bash
cargo run
cargo test
```

The timer tests use Tokio's paused clock. [Future's polling contract](https://doc.rust-lang.org/std/future/trait.Future.html), [Tokio runtime guide](https://tokio.rs/tokio/tutorial)

Next: [Lesson 2: Running things concurrently](../02-running-concurrently/)
