<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Message passing

## The idea in one sentence

Instead of sharing data, threads can **send** it to each other through a **channel**, so each piece of data has exactly one owner at a time and there's nothing to lock.

> "Do not communicate by sharing memory; instead, share memory by communicating." This line comes from the Go language community, and it applies just as well to Rust.

## A channel has two ends

```rust
let (sender, receiver) = mpsc::channel();

thread::spawn(move || sender.send(42).unwrap());   // one thread puts messages in…
let value = receiver.recv().unwrap();              // …another takes them out
```

`mpsc` stands for **multiple producer, single consumer**:
- the **`Sender`** can be **cloned**, so any number of threads can send;
- there's only one **`Receiver`**.

**Sending moves the value.** After `sender.send(reading)`, the sending thread no longer has `reading`. Ownership has travelled through the channel. That's why message passing is safe without locks: two threads can never be using the same value.

## When does the channel close?

```rust
for message in receiver.iter() { … }    // waits for each message
```

Iterating the receiver waits for each message, and **ends when every `Sender` has been dropped**. That's how the consumer knows the work is finished, with no special "stop" message needed.

The classic mistake: cloning the sender for each producer but keeping the original. The receiver then waits forever, because one sender is still alive. The demo's `drop(sender)` after creating the producers is there for exactly this reason.

## Bounded channels: backpressure

```rust
let (sender, receiver) = mpsc::sync_channel(2);    // room for 2 waiting messages
```

`mpsc::channel()` has unlimited room, so a producer faster than its consumer slowly fills memory. **`sync_channel(n)`** holds at most `n` messages, and `send` **waits** when it's full. The demo's slow consumer makes this visible:

```text
sent 1 after   0 ms
sent 2 after   0 ms
sent 3 after   0 ms     ← one taken by the consumer, two in the buffer
sent 4 after  34 ms     ← had to wait for the consumer to make room
sent 5 after  69 ms
```

## A worker pool: several consumers sharing one queue

A `Receiver` can't be shared by several threads directly:

```text
error[E0277]: `std::sync::mpsc::Receiver<u32>` cannot be shared between threads safely
```

Wrap it in `Arc<Mutex<…>>`, and workers take turns pulling the next job:

```rust
let job = job_receiver.lock().unwrap().recv();    // lock just long enough to take one job
```

The lock is released at the end of that line, **before** the job is processed, so the other workers can take jobs meanwhile. Results come back through a second channel. Which worker handles which job varies from run to run, so the demo sorts the results.

## Waiting with a timeout

`receiver.recv_timeout(Duration::from_millis(100))` gives up after the deadline:

```text
nothing arrived: timed out waiting on channel
```

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: Threads](../01-threads/) · Next: [Lesson 3: Locks and condition variables](../03-locks-and-condvars/)
