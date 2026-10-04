<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Concurrency with Threads

Rust's promise is **fearless concurrency**: the compiler rejects data races before your program ever runs. This course shows the tools that make it possible, from starting threads to lock-free atomics and one-word parallel iterators.

Every compiler error and panic message quoted in these READMEs is the real output of Rust 1.99, and timings come from real runs on an 8-core Apple M2.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [Threads](01-threads/) | `spawn` and `join`, `move`, scoped threads, panics in threads, how many threads to use |
| 2 | [Message passing](02-message-passing/) | `mpsc` channels, many producers, closing a channel, backpressure, a worker pool |
| 3 | [Locks and condition variables](03-locks-and-condvars/) | `Arc<Mutex<T>>`, poisoning, `RwLock`, `Condvar`, `Barrier`, file locks between programs, avoiding deadlocks |
| 4 | [Atomics](04-atomics/) | lock-free counters and flags, `compare_exchange`, memory `Ordering` explained |
| 5 | [Data parallelism with rayon](05-data-parallelism/) | `par_iter`, parallel sort, `rayon::join`, when parallelism doesn't pay |

## Choosing a tool

| You want to… | Use |
|---|---|
| run work in the background | `thread::spawn` (lesson 1) |
| split existing data across threads | `thread::scope` (lesson 1) or rayon (lesson 5) |
| hand work or results between threads | a channel (lesson 2) |
| share data several threads change | `Arc<Mutex<T>>` (lesson 3) |
| share data that's mostly read | `Arc<RwLock<T>>` (lesson 3) |
| a single shared counter or flag | an atomic (lesson 4) |
| do the same CPU-heavy work on many items | rayon (lesson 5) |
| wait on the network or timers | async (see the async course) |

## The errors you'll meet

| Error | In plain words | Lesson |
|---|---|---|
| **E0373** closure may outlive the current function | a thread borrows local data; use `move` or `thread::scope` | 1 |
| **E0382** use of moved value | data moved into one thread can't be used by another; use `Arc` | 1, 3 |
| **E0277** … cannot be shared between threads safely | `Receiver`, `Cell`, `RefCell` and `Rc` aren't thread-safe; use the thread-safe versions | 2, 3 |
| `PoisonError { .. }` (at runtime) | a thread panicked while holding the lock | 3 |
| **E0596** … captured variable in a `Fn` closure | a parallel closure can't change shared state; use `sum`, `reduce` or an atomic | 5 |

## Run a lesson

```bash
cd concurrency/02-message-passing
cargo run
cargo test
```

Or from the repository root: `cargo run -p message-passing`. For lesson 5, add `--release` for meaningful timings.
