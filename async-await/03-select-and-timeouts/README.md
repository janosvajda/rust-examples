<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: select and timeouts

## The idea in one sentence

**`select!` chooses a ready, matching branch; `timeout` limits how long you wait for an operation.**

Ferris asks two kitchens for a dish and accepts one reply. Taking the other order off Ferris's list does not magically undo food already cooked. Async cancellation has that same limit: it can stop future work without undoing past effects.

## select!: choose one result

Using the simulated `ask_server` helper:

```rust
let reply = tokio::select! {
    answer = ask_server("Europe mirror", 150) => answer,
    answer = ask_server("US mirror", 400) => answer,
};
println!("{reply}");
```

Both branch futures are polled in the **current task**, so their timer waits can overlap. Once a ready result matches its branch pattern, that branch's handler runs. The other **owned branch future** is dropped. Here the shorter timer normally wins; the test verifies the 150 ms result using paused Tokio time.

This chooses a completion, not necessarily a successful result. An `Err` can win too. If several branches are ready, default polling order is randomised: the first written branch does not always win. Patterns and `if` conditions can disable branches; an `else` branch handles the case where none remain enabled, otherwise `select!` panics.

Dropping a temporary borrow such as `&mut operation` does not drop the operation stored outside the macro. That distinction lets you reuse a future.

Many futures from `async fn` need pinning before they can be awaited through a mutable reference. `tokio::pin!` keeps the future's stored value in place; the pinned handle can still move. Here a short timer can finish first, and we continue waiting for the **same** operation:

```rust
let operation = ask_server("saved operation", 100);
tokio::pin!(operation);
let reply = tokio::select! {
    reply = &mut operation => reply,
    _ = tokio::time::sleep(std::time::Duration::from_millis(10)) => operation.await,
};
assert_eq!(reply, "answer from saved operation");
```

## timeout: a cooperative deadline

```rust
use std::time::Duration;
use tokio::time::timeout;

let result = timeout(
    Duration::from_millis(300),
    ask_server("slow server", 900),
).await;
assert!(result.is_err());
```

The result is `Ok(answer)` on completion or `Err(Elapsed)` when the timeout expires. Our `ask_with_deadline` helper turns the timeout error into a friendly message.

A timeout is **not an alarm that interrupts arbitrary code**. Tokio must get control back to check it. A blocking call or endless computation inside the future can exceed the limit without yielding; the timeout may then fail to report that overrun. Completion right at the deadline also needs a deliberate policy rather than assumptions about ties.

Network operations usually need a deliberate timeout policy: a per-request deadline, idle timeout or longer-lived connection can require different choices. Our examples perform no real network requests.

## A worker that stops cleanly

The runnable worker waits for a timer or a `oneshot` stop message. This complete helper excerpt shows its control flow:

```rust
use std::time::Duration;
use tokio::sync::oneshot;

async fn worker(mut stop: oneshot::Receiver<()>) -> u32 {
    let mut ticker = tokio::time::interval(Duration::from_millis(100));
    let mut ticks = 0_u32;
    loop {
        tokio::select! {
            biased;
            _ = &mut stop => return ticks,
            _ = ticker.tick() => ticks = ticks.saturating_add(1),
        }
    }
}
```

`&mut stop` borrows the same receiver on each iteration. A winning tick drops only that temporary borrow, leaving the receiver available next time. The receiver completes on either a sent message or a dropped sender; this worker treats both as shutdown.

`biased;` checks branches in written order. Putting shutdown first gives an already-ready stop priority over a tick. This does not interrupt a branch that is already doing work.

**Fun timer surprise:** `interval`'s first tick is immediate. The test that stops after 250 ms counts ticks at 0, 100 and 200 ms. Missed ticks use Tokio's default `Burst` policy, so delayed workers can get catch-up ticks; use another policy when that would be unsuitable. This counter saturates at `u32::MAX` rather than wrapping.

## Dropping an operation and dropping a handle differ

| What you drop | What happens |
|---|---|
| unpolled future from our `ask_server` | its async body never starts; captured values are still dropped |
| partially polled owned operation future | its stored state and live values are dropped; it receives no further polls |
| `JoinHandle` from `tokio::spawn` | the handle detaches; the task can keep running |
| `JoinSet` | requests cancellation of its tracked tasks |

An operation's destructor can release its owned resources. It cannot `.await` asynchronous cleanup, undo a sent payment, or guarantee that a remote server stops processing a request. Graceful shutdown often means sending a stop request and then awaiting the worker's handle, as the demo does.

## A timeout around a task handle stops waiting

```rust
use std::time::Duration;

let mut task = tokio::spawn(async {
    tokio::time::sleep(Duration::from_millis(200)).await;
    "background result"
});
if tokio::time::timeout(Duration::from_millis(20), &mut task).await.is_err() {
    task.abort(); // request cancellation of this async task
    match task.await {
        Err(error) if error.is_cancelled() => println!("cancelled"),
        Ok(result) => println!("already completed: {result}"),
        Err(error) => println!("task failed: {error}"),
    }
}
```

Borrowing the handle keeps it available after the timeout. `abort()` requests cancellation; awaiting the handle waits for task termination and cleanup. Completion can race with cancellation, so handle both outcomes. A non-yielding async task cannot be interrupted mid-poll this way, and an already-running `spawn_blocking` closure cannot be aborted.

## Cancellation safety: can we restart without losing progress?

In a loop, a losing branch may be dropped and recreated. For **`mpsc::Receiver::recv`**, losing that race does not remove a message from the queue. But a larger future that receives a message and then awaits processing can lose the in-progress message when cancelled.

Likewise, cancelling `read_exact` or `write_all` can leave partial I/O. Check each API's cancellation contract. Put work that must finish outside the repeatedly cancelled receive branch, or design an explicit recovery or shutdown path.

## Try it

Predict whether a worker receives a first tick before 100 ms, then run:

```bash
cargo run
cargo test
```

[select and cancellation safety](https://docs.rs/tokio/latest/tokio/macro.select.html), [timeout](https://docs.rs/tokio/latest/tokio/time/fn.timeout.html), [interval](https://docs.rs/tokio/latest/tokio/time/fn.interval.html), [task cancellation](https://docs.rs/tokio/latest/tokio/task/struct.JoinHandle.html#method.abort)

Previous: [Lesson 2: Running things concurrently](../02-running-concurrently/) · Next: [Lesson 4: Channels and shared state](../04-channels-and-shared-state/)
