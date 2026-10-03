<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: Threads

## The idea in one sentence

A **thread** is an independent path of execution, and the operating system runs threads **at the same time** on different CPU cores, so a program can do several things at once.

## Starting a thread and waiting for it

```rust
let handle = thread::spawn(|| {
    // runs in the new thread
    "worker finished"
});

let result = handle.join().unwrap();    // wait for it, and get its result
```

- **`thread::spawn`** takes a closure and starts running it in a new thread **immediately**. Your code carries on at the same time.
- It returns a **`JoinHandle`**. **`.join()`** waits until the thread ends and gives you the closure's return value.
- When `main` returns, the whole program ends, **including threads that haven't finished**. Join the threads whose work matters.

## Why bother: real parallelism

```text
[1, 4, 9, 16]: four 200 ms jobs took 205 ms, not 800
```

Four threads, four jobs, all running at once. Threads are for doing several things **in parallel**, especially CPU-heavy work. (For work that mostly *waits*, on the network or a timer, see the async course.)

## Threads own their data: `move`

```rust
let names = vec![String::from("Ana"), String::from("Bob")];
thread::spawn(move || names.len());      // `names` now belongs to the thread
```

A spawned thread can outlive the function that started it, so it can't borrow that function's local variables. Without `move`:

```text
error[E0373]: closure may outlive the current function, but it borrows `names`
```

With `move` the thread takes ownership, and `names` can't be used afterwards (`E0382`). To use the same data from several threads, share it with `Arc` (lesson 3) or send it through a channel (lesson 2).

## Scoped threads: borrowing is allowed

```rust
thread::scope(|s| {
    s.spawn(|| low.iter().sum::<i32>());     // borrows `low`
    s.spawn(|| high.iter().sum::<i32>());    // borrows `high`
});                                          // all scoped threads end here
```

`thread::scope` guarantees every thread started inside it has finished before it returns, so the threads may borrow local data. It's the simplest way to split work across threads when the data already exists. More in the ownership course, lesson 8.

## A panic stays in its thread

If a thread panics, Rust prints the message. In Rust 1.99 the first line shows the thread's name (`'<unnamed>'` if it has none) and its numeric ID:

```text
thread 'unlucky-worker' (1614879) panicked at concurrency/01-threads/src/main.rs:14:25:
the setting must be a number: ParseIntError { kind: InvalidDigit }
```

Only **that** thread stops. Its `join()` returns `Err` instead of `Ok`, and the rest of the program carries on. Use `thread::Builder::new().name(…)` to give threads names, so panics and debuggers show which one failed.

## How many threads?

`thread::available_parallelism()` tells you how many threads the machine can really run at the same time, usually the number of CPU cores. For CPU-heavy work, more threads than that doesn't help: they just take turns. For a pool of workers, it's a sensible default size.

## Run it

```bash
cargo run
cargo test
```

Next: [Lesson 2: Message passing](../02-message-passing/)
