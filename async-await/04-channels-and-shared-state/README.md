<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Channels and shared state

## The idea in one sentence

Tasks communicate either by **sending messages** through channels, or by **sharing data** behind `Arc<Mutex<T>>`. Channels are often the simpler design.

## Channels: tokio's three most common kinds

| Channel | Shape | Use it for |
|---|---|---|
| **`mpsc`** | many senders → one receiver | a queue of work or events: orders to a kitchen, log lines to a writer |
| **`oneshot`** | one sender → one receiver, **one** message | a single reply: "here's the answer to your request" |
| **`broadcast`** | many senders → **every** receiver gets each message | notifications everyone should see: "config changed", "shutting down" |

## `mpsc`: many waiters, one kitchen

```rust
let (sender, receiver) = mpsc::channel(8);           // room for 8 waiting messages
tokio::spawn(waiter("Anna", …, sender.clone()));     // each producer gets a Sender
tokio::spawn(waiter("Ben",  …, sender));
tokio::spawn(kitchen(receiver));                     // one consumer

// in the kitchen:
while let Some(event) = orders.recv().await { … }
```

- **`send(...).await`** waits if the channel is full. This is *backpressure*: fast producers automatically slow down to match a slow consumer, instead of filling memory.
- **`recv().await`** waits for the next message, and returns **`None` once every `Sender` has been dropped** and the channel is empty. That's how the kitchen knows the shift is over without a special "stop" message. A test checks this.

## `oneshot`: request and reply

A request can carry its own reply channel. The service answers on it, and the asker awaits the answer:

```rust
struct PriceRequest {
    item: String,
    reply: oneshot::Sender<Option<u32>>,
}

let (reply, answer) = oneshot::channel();
service.send(PriceRequest { item, reply }).await?;
let price = answer.await?;
```

Together, `mpsc` + `oneshot` make a small **service**: one task owns some data, and everyone else asks it questions through messages. No locks are needed, because only that one task ever touches the data.

## Shared state: `Arc<Mutex<T>>`

```rust
let counter = Arc::new(Mutex::new(0));          // tokio::sync::Mutex
let mut count = counter.lock().await;           // waits for its turn without blocking a thread
*count += 1;
```

It's the same idea as with threads (ownership course, lesson 8): `Arc` for shared ownership, `Mutex` for one-at-a-time access.

### `std::sync::Mutex` or `tokio::sync::Mutex`?

| | `std::sync::Mutex` | `tokio::sync::Mutex` |
|---|---|---|
| Waiting for the lock | blocks the thread | `.await`: the thread runs other tasks meanwhile |
| Hold the lock across an `.await` | ✗ (see lesson 5) | ✓ that's what it's for |
| Speed | faster | a little slower |

**Rule of thumb:** use `std::sync::Mutex` when the lock is held briefly with no `.await` inside, which is the usual case and often faster. Use `tokio::sync::Mutex` only when you must hold the lock *across* an `.await`.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: select and timeouts](../03-select-and-timeouts/) · Next: [Lesson 5: Async pitfalls](../05-async-pitfalls/)
