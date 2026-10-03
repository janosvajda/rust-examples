<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: select and timeouts

## The idea in one sentence

`join!` waits for **all** futures. `select!` waits for the **first** one, and `timeout` gives a future a deadline. Whatever doesn't finish is **cancelled** simply by being dropped.

## `select!`: the first one wins

```rust
tokio::select! {
    answer = ask_server("Europe mirror", 150) => answer,
    answer = ask_server("US mirror", 400) => answer,
}
```

Both futures run concurrently. As soon as one finishes, its branch runs and the **other future is dropped**:

```text
answer from Europe mirror after 152 ms
```

The US request never finishes. It's cancelled, and nobody waits the extra 250 ms.

## `timeout`: a deadline

```rust
timeout(Duration::from_millis(300), ask_server("slow server", 900)).await
// → Err(Elapsed): gave up at 300 ms
```

`timeout` returns `Ok(result)` if the future finished in time, or an error if the deadline passed first. It's `select!` between your future and a timer, packaged up. The test checks that a 5-second request with a 200 ms deadline takes exactly 200 ms.

Every network call in real code should have a timeout. Without one, a server that never answers makes your program wait forever.

## `select!` in a loop: work until told to stop

```rust
loop {
    tokio::select! {
        _ = ticker.tick() => { /* periodic work */ }
        _ = &mut stop      => { /* clean up */ return ticks; }
    }
}
```

A common shape for background workers: do something every 100 ms, but react **immediately** when a stop signal arrives. The stop signal is a `oneshot` channel (lesson 4). `&mut stop` lets the same receiver be checked again on every loop iteration instead of being used up by the first `select!`.

## Cancellation: just drop the future

In Rust, **cancelling a future means dropping it.** A future only makes progress when polled. Once it's dropped, it's never polled again, and its `Drop` code runs to clean up, for example closing a connection.

```rust
let slow = ask_server("never awaited", 10_000);
drop(slow);     // cancelled: it never ran at all
```

**Be careful where you are when you're cancelled.** A future can be dropped at **any** `.await` point. If it was halfway through something, say it had read a message from a channel but not yet processed it, that work is lost. Keep each `.await` in a `select!` branch at a point where stopping is safe. Tokio's documentation calls this *cancellation safety*.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: Running things concurrently](../02-running-concurrently/) · Next: [Lesson 4: Channels and shared state](../04-channels-and-shared-state/)
