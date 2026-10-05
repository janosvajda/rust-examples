<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Async pitfalls

## The idea in one sentence

**Async works well when tasks return control promptly, ownership is clear, and background work has a managed lifetime.**

A waiter who blocks the restaurant doorway for 200 ms stops other waiters using that doorway. Giving that waiter an “async” badge does not remove the blockage.

## 1. Blocking inside async code

A task only pauses at an `.await` that has to wait. Code that runs for a long time *between* awaits, a big calculation or a blocking call like `std::thread::sleep`, keeps the thread busy, and every other task on that thread waits until it finishes.

The demo's `heavy_calculation` actually calls `std::thread::sleep` for 200 ms. It **simulates blocking work**, rather than measuring CPU calculations. Four joined calls on a current-thread runtime take roughly 800 ms of blocking waits. Offloading them can overlap those waits, but elapsed time depends on scheduling and available resources.

```rust
let answer = tokio::task::spawn_blocking(|| {
    std::thread::sleep(std::time::Duration::from_millis(200));
    42
}).await.unwrap();
assert_eq!(answer, 42);
```

`spawn_blocking` uses a separate blocking thread pool, including when the async runtime has only one scheduler thread. It is suitable for finite synchronous work. A started closure cannot be interrupted by `abort()`; it must finish or cooperate with its own stop mechanism.

For an async timer use `tokio::time::sleep`. For ordinary files, `tokio::fs` exposes async APIs but currently performs blocking file operations on Tokio's blocking pool. It does not make the underlying disk operation vanish.

For many CPU-heavy jobs, bound concurrency or use a suitable CPU pool such as Rayon. Spawning huge numbers of CPU jobs onto the blocking pool can overload the machine. For long-lived blocking workers, a dedicated thread can be a better fit.

There is usually **no compiler error** for accidentally blocking the runtime. Our tests verify task progress through coordination, not a claim that every computer finishes within 600 ms.

## 2. A standard mutex guard across an await

This example **does not compile** because its future is passed to `tokio::spawn`:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0_u32));
tokio::spawn(async move {
    let mut guard = counter.lock().unwrap();
    tokio::task::yield_now().await;
    *guard += 1;
});
```

```text
error: future cannot be sent between threads safely
```

`std::sync::MutexGuard` is not `Send`: its lock must be released on the acquiring thread. The spawned future retains that guard across a possible suspension, so it cannot satisfy `tokio::spawn`'s `Send` requirement.

Release it before awaiting:

```rust
use std::sync::{Arc, Mutex};

let counter = Arc::new(Mutex::new(0_u32));
let task = tokio::spawn(async move {
    {
        let mut guard = counter.lock().unwrap();
        *guard += 1;
    }
    tokio::task::yield_now().await;
});
task.await.unwrap();
```

A block makes the guard's lifetime clear. If you need a guard across an await, choose an appropriate async mutex and still think about lock contention and deadlocks. Local non-`Send` code can compile with a standard guard across an await; compiling does not make that design deadlock-free.

## 3. Send is a property of the whole spawned future

This example **does not compile**:

```rust
tokio::spawn(async {
    let shared = std::rc::Rc::new(5);
    tokio::task::yield_now().await;
    println!("{shared}");
});
```

```text
error: future cannot be sent between threads safely
```

`tokio::spawn` requires the task's future, and its result, to be `Send + 'static`: the task may move to another thread, even if this runtime happens to have only one. Everything the task captures, and every variable still alive at an `.await`, is part of the future, so it must all be `Send`.

An `Rc` created inside the task can be used if it is gone before any suspension:

```rust
let task = tokio::spawn(async {
    {
        let local = std::rc::Rc::new(5);
        println!("{local}");
    }
    tokio::task::yield_now().await;
});
task.await.unwrap();
```

For shared ownership between threads, consider `Arc`. **`RefCell<T>` can itself be `Send` when `T: Send`**, but it is not `Sync`; sharing one through `Arc<RefCell<T>>` does not make it thread-safe. Moving a value and sharing references to it are different operations.

When a task genuinely needs non-`Send` state across awaits, use a **`LocalSet`** and `spawn_local`:

```rust
let local = tokio::task::LocalSet::new();
local.run_until(async {
    let shared = std::rc::Rc::new(5);
    let task = tokio::task::spawn_local(async move {
        tokio::task::yield_now().await;
        *shared
    });
    assert_eq!(task.await.unwrap(), 5);
}).await;
```

These tasks stay on the thread driving the local set. `spawn_local` still requires `'static`; it does not allow arbitrary short-lived captured borrows. Keep driving or awaiting local tasks when their completion matters.

## 4. Recursive futures need indirection

This example **does not compile**:

```rust
async fn countdown(n: u32) {
    if n > 0 {
        countdown(n - 1).await;
    }
}
```

```text
error[E0733]: recursion in an async fn requires boxing
```

A generated future needs space for the inner future it awaits. Direct recursion would require a type containing itself without an end: a box inside a box inside a box…

Heap indirection gives the stored recursive future a finite-size pointer:

```rust
const MAX_COUNTDOWN_DEPTH: u32 = 256;

async fn countdown(n: u32) -> Result<Vec<u32>, &'static str> {
    if n > MAX_COUNTDOWN_DEPTH { return Err("recursive countdown exceeds 256 steps"); }
    async fn append(n: u32, output: &mut Vec<u32>) {
        if n == 0 { return; }
        output.push(n);
        Box::pin(append(n - 1, output)).await;
    }
    let mut output = Vec::with_capacity(n as usize);
    append(n, &mut output).await;
    Ok(output)
}
assert_eq!(countdown(3).await.unwrap(), [3, 2, 1]);
```

`Box::pin` puts the inner future on the heap, so the outer future only holds a pointer to it: a fixed size, however deep the recursion goes. Boxing fixes the **type's size**, but deep recursion can still use a lot of stack, so the example refuses more than 256 steps.

For this particular countdown, an ordinary iterator is simpler: `(1..=n).rev().collect::<Vec<_>>()`. More complex algorithms may need recursion, an explicit work stack or another design.

## 5. Creating an operation but forgetting to drive it

```rust
async fn save_note() {
    println!("simulated save"); // no real file is written
}
save_note(); // deliberate unused-Future warning; body never runs
```

```text
warning: unused implementer of `Future` that must be used
```

When you intend to perform the operation:

```rust
save_note().await;
```

Investigate unused-future warnings. If discarding an operation is deliberate, `drop(future)` makes that explicit. Not every future means “unstarted work”: discarding a spawned task's `JoinHandle` detaches the already-scheduled task rather than cancelling it.

## Choose a tool

| You want to… | Consider |
|---|---|
| await an async operation | `.await`; it may continue immediately if ready |
| collect several concurrent results | `join!` or `try_join!` |
| schedule and manage independent tasks | `tokio::spawn` with retained handles, or `JoinSet` |
| choose a ready branch | `select!` |
| limit waiting time | `timeout`, with cooperative operations |
| send work and receive replies | `mpsc` and `oneshot` |
| coordinate shared access | an appropriate mutex, or a service owning the data |
| offload finite blocking work | `spawn_blocking` |
| run non-`Send` local tasks | `LocalSet` and `spawn_local` |

## Try it

Guess whether `spawn_blocking` still uses extra threads when the async scheduler is single-threaded, then run:

```bash
cargo run
cargo test
```

[Blocking pool](https://docs.rs/tokio/latest/tokio/task/fn.spawn_blocking.html), [spawn and Send](https://docs.rs/tokio/latest/tokio/task/fn.spawn.html), [LocalSet](https://docs.rs/tokio/latest/tokio/task/struct.LocalSet.html), [Tokio file I/O](https://docs.rs/tokio/latest/tokio/fs/index.html), [Pin](https://doc.rust-lang.org/std/pin/index.html)

Previous: [Lesson 4: Channels and shared state](../04-channels-and-shared-state/) · Back to the [course overview](../)
