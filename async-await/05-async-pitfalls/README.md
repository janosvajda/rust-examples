<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Async pitfalls

## The idea in one sentence

Almost everyone hits the same five problems when starting with async Rust. Each one has a clear symptom and a standard fix.

## 1. Blocking the runtime

An async task gives its thread back **only at an `.await`**. Code that runs for a long time without awaiting holds the thread, and every other task on that thread is stuck waiting. That includes `std::thread::sleep`, a heavy calculation, or a synchronous file or database call.

The demo runs four 200 ms jobs on a single-threaded runtime, two ways:

```text
blocking inside async:   810 ms     ← the four jobs ran one after another
moved to spawn_blocking: 205 ms     ← the four jobs ran at the same time
```

**Fix:** move blocking work off the async threads with **`tokio::task::spawn_blocking`**, which runs it on a separate pool of threads meant for blocking. For waiting, use the async versions: `tokio::time::sleep`, not `std::thread::sleep`; `tokio::fs`, not `std::fs` in hot paths.

**There's no compiler error for this one.** The program just becomes mysteriously slow under load. If async code is slow, look for blocking calls first.

## 2. Holding a `std` Mutex lock across `.await`

```rust
let mut guard = counter.lock().unwrap();   // std::sync::Mutex
some_call().await;                          // the task may move to another thread here
*guard += 1;
```

Inside `tokio::spawn`:

```text
error: future cannot be sent between threads safely
```

A `std` lock guard must be released on the same thread that took it, but `tokio::spawn` may move the task to another thread at the `.await`. Even on one thread, holding a lock while waiting blocks everyone else who needs it.

**Fix:** release the lock before awaiting, by putting it in its own block:

```rust
{
    let mut guard = counter.lock().unwrap();
    *guard += 1;
}                                           // lock released here
some_call().await;
```

Or, if you really must hold it across the `.await`, use `tokio::sync::Mutex` (lesson 4).

## 3. `Rc` and other non-`Send` types in spawned tasks

```rust
let shared = Rc::new(5);
tokio::spawn(async move { ….await; println!("{shared}") });
```

```text
error: future cannot be sent between threads safely
```

Same cause: a spawned task may move between threads at any `.await`, so everything it holds across an `.await` must be `Send` (ownership course, lesson 8). **Fix:** `Arc` instead of `Rc`, `Mutex` instead of `RefCell`.

## 4. Recursion needs a box

```rust
async fn countdown(n: u32) { if n > 0 { countdown(n - 1).await } }
```

```text
error[E0733]: recursion in an async fn requires boxing
```

A future stores everything it needs across its `.await`s **inside itself**. A future that contains a call to itself would contain itself, which contains itself… and would be infinitely large. **Fix:** put the recursive call on the heap with `Box::pin`:

```rust
let mut rest = Box::pin(countdown(n - 1)).await;
```

## 5. Forgetting `.await`

```rust
save_to_disk();          // looks like a call; does nothing at all
```

```text
warning: unused implementer of `Future` that must be used
  = note: futures do nothing unless you `.await` or poll them
```

Futures are lazy (lesson 1). Treat this warning as an error: it always means a bug.

## The course in one table

| You want to… | Use |
|---|---|
| wait for something without blocking a thread | `.await` |
| run several futures at once and wait for all | `join!`, `try_join!` |
| start independent background work | `tokio::spawn`, `JoinSet` |
| take whichever finishes first | `select!` |
| give up after a while | `timeout` |
| send work or events between tasks | `mpsc` (and `oneshot` for replies) |
| share data between tasks | `Arc<Mutex<T>>` |
| run blocking or CPU-heavy code | `spawn_blocking` |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 4: Channels and shared state](../04-channels-and-shared-state/) · Back to the [course overview](../)
