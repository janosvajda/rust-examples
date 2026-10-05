<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Channels and shared state

## The idea in one sentence

**Tasks can send each other messages, or coordinate access to shared data.**

Ferris and Ben send order tickets to one kitchen. A tray holds a limited number of waiting tickets. When it fills, waiters pause until the kitchen takes one: this is **backpressure**.

## Pick the right channel

| Tokio channel | Shape | Useful example | Important detail |
|---|---|---|---|
| `mpsc` | many senders → one receiver | queue of kitchen orders | bounded `channel(n)` waits when full |
| `oneshot` | one sender → one receiver | reply to one question | at most one message; a sender can disappear without replying |
| `broadcast` | many senders → subscribers | notifications to several listeners | a slow receiver can lose old messages and get `Lagged` |
| `watch` | senders → receivers watching the latest value | current opening-hours setting | intermediate updates can be skipped; the latest value is retained |

A broadcast receiver sees messages sent after it subscribes, subject to the bounded retention window. Broadcasting is not a promise that every receiver processes every message. The runnable lesson focuses on `mpsc` and `oneshot`.

## mpsc: two waiters, one kitchen

This uses `waiter`, `kitchen` and `Event` from the runnable source:

```rust
use tokio::sync::mpsc;

let (sender, receiver) = mpsc::channel(8);
let kitchen_task = tokio::spawn(kitchen(receiver));
let anna = tokio::spawn(waiter("Anna", vec![(1, "soup")], sender.clone()));
let ben = tokio::spawn(waiter("Ben", vec![(2, "salad")], sender));

anna.await.unwrap();
ben.await.unwrap();
let cooked = kitchen_task.await.unwrap();
assert_eq!(cooked.len(), 2); // order between different waiters may vary
```

Each waiter owns a strong sender. The original sender is moved into Ben's task; leaving an unused clone alive would keep the receiver waiting for more orders.

- `send(value).await` waits for capacity when the bounded queue is full, or returns an error if it is closed.
- `recv().await` waits for a message when the queue is open and empty.
- Once the channel is **closed and drained**, `recv().await` returns `None`.

Dropping all strong senders normally closes the channel. The receiver can also call `close()` while senders still exist, then drain queued messages. Outstanding reserved permits must be released or used and drained before reception ends.

Buffer capacity limits **queued messages**, not their byte sizes or the number of producer tasks. An `unbounded_channel` does not provide this capacity backpressure.

The kitchen simulation collects orders; it does not really cook or sleep per dish. Its `Closed` events announce individual waiters finishing. The receive loop itself ends through channel closure.

## oneshot: a question with its own reply address

The runnable service receives a `PriceRequest` on an `mpsc` queue. Each request includes its own `oneshot::Sender`, like an envelope with a return address.

The source's `ask_price` helper performs these steps:

1. Create a reply channel.
2. Send the item and reply sender to the service.
3. Await the one reply.

```rust
use tokio::sync::mpsc;

let (service, requests) = mpsc::channel(4);
let worker = tokio::spawn(price_service(requests));

assert_eq!(ask_price(&service, "coffee").await, Ok(Some(250)));
assert_eq!(ask_price(&service, "yacht").await, Ok(None));

drop(service); // stop accepting new requests once the queue drains
worker.await.unwrap();
```

Prices are whole **cents**: 250 means 2.50 currency units. The helper returns `Result<Option<u32>, PriceError>`:

| Result | Meaning |
|---|---|
| `Ok(Some(250))` | the service found a price |
| `Ok(None)` | the service answered: unknown item |
| `Err(PriceError::ServiceClosed)` | the request could not be sent |
| `Err(PriceError::ReplyDropped)` | no reply arrived because the reply sender was dropped |

A failed channel is different from “we do not sell that item.” The tests check both error paths.

`oneshot::Sender::send` is synchronous: it does not use `.await`. The service ignores a failed reply send because the asker may have cancelled its request. The asker awaiting the receiver gets an error if its reply sender disappears.

This service design is often called an **actor**: one task owns the data and processes messages. Its price table needs no shared-data lock because other tasks ask it questions instead of reading that table directly.

## Sharing a counter with Arc and Mutex

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

let counter = Arc::new(Mutex::new(0_u64));
{
    let mut count = counter.lock().await;
    *count += 1;
} // drop the guard, releasing the lock
assert_eq!(*counter.lock().await, 1);
```

`Arc` shares ownership of the allocation. The mutex gives one guard at a time permission to access the protected counter. `Arc` alone does not make arbitrary mutable contents safe to share.

The runnable `count_visits(10, 100)` joins every task and returns 1,000. Its `u64` counter can hold the largest total described by its two `u32` inputs. Actual demos use small task counts; large workloads need a task limit.

## std Mutex or Tokio Mutex?

| Question | `std::sync::Mutex` | `tokio::sync::Mutex` |
|---|---|---|
| Waiting for a held lock | blocks the calling thread | can suspend the task through `lock().await` |
| Guard across an await | `MutexGuard` is not `Send`; conflicts with `tokio::spawn` | supported when the relevant type bounds hold |
| Panic while holding it | may poison the mutex | does not use poisoning |

A **brief, low-contention lock with no await inside** is often a good use for a standard mutex; Tokio's mutex has additional async machinery. Performance depends on contention and workload.

Use an async mutex when waiting must not block a runtime thread or when a guard genuinely needs to cross an await. A standard guard can cross an await in some local, non-`Send` code, but that can deadlock the runtime if another task blocks trying to take the same lock.

Either mutex can cause a deadlock through a bad locking design. An async guard held during slow I/O still makes other callers wait. Keep critical sections short, or consider a service task that owns the resource.

## Try it

Predict whether leaving an extra sender clone alive would let this demo's kitchen finish, then run:

```bash
cargo run
cargo test
```

[Tokio mpsc](https://docs.rs/tokio/latest/tokio/sync/mpsc/index.html), [oneshot](https://docs.rs/tokio/latest/tokio/sync/oneshot/index.html), [broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html), [watch](https://docs.rs/tokio/latest/tokio/sync/watch/index.html), [mutex selection](https://docs.rs/tokio/latest/tokio/sync/struct.Mutex.html)

Previous: [Lesson 3: select and timeouts](../03-select-and-timeouts/) · Next: [Lesson 5: Async pitfalls](../05-async-pitfalls/)
