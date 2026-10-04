<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 8: Threads and borrowing

## The idea in one sentence

Borrowing, **`Send` and `Sync`**, and safe synchronisation APIs work together to prevent data races in safe Rust.

Think of two cooks: they can read the same recipe, or chop ingredients on separate boards. If both need to change one shared order list, they need a coordinated way to take turns.

## Data races, and how safe Rust prevents them

A **data race** involves conflicting accesses to the same memory from different threads: at least one writes, at least one is non-atomic, and the accesses lack the required synchronisation ordering. They need not literally execute during the same clock tick. A data race is **undefined behaviour**, so incorrect values or crashes are possible rather than predictable outcomes.

Ordinary borrowing prevents conflicting shared and exclusive access. `Send` and `Sync` extend the safety rules across threads, including for types with interior mutability. A `Mutex` or `RwLock` can coordinate access to shared data; **atomic operations** can coordinate supported values without a lock. Unsafe implementations must uphold their own safety requirements.

## `Send` and `Sync`

Two traits tell the compiler what may cross between threads. You almost never implement them yourself: the compiler works them out from what a type contains.

| Trait | Means | Not implemented by |
|---|---|---|
| **`Send`** | a value can be **moved** to another thread | `Rc<T>` |
| **`Sync`** | a value can be **shared** between threads (`&T` is `Send`) | `Rc<T>`, `Cell<T>`, `RefCell<T>` |

`Rc` has a non-atomic ownership counter; `RefCell` has a non-atomic borrow counter. `Cell` has **no borrow counter**, but it lets shared callers change its contents without thread synchronisation, so it is not `Sync`.

`Cell<T>` and `RefCell<T>` can still be **moved** to another thread when `T: Send`. Sharing one through `&` is a different operation. The following diagnostics come from moving an `Rc` or trying to share an `Arc<RefCell<i32>>` with a spawned thread:

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
```

## Problem 1: `thread::spawn` requires `'static` captures

This example **does not compile**:

```rust
use std::thread;
let names = vec![String::from("Ana")];
thread::spawn(|| println!("{names:?}"));   // ✗
```

```text
error[E0373]: closure may outlive the current function, but it borrows `names`,
              which is owned by the current function
```

A spawned thread may keep running after the function that started it returns, when `names` would be dropped. `thread::spawn` therefore requires its closure and return value to satisfy `'static` as well as `Send`. It **can** borrow suitable `'static` data, such as a shared static value.

For this local vector, two fixes are:

1. **`move`**: give the thread ownership, `thread::spawn(move || ...)`.
2. **Scoped threads**: if you want to *borrow*, use `thread::scope`.

## Scoped threads: borrowing across threads

```rust
use std::thread;
let scores = vec![70, 85, 90, 100];
thread::scope(|s| {
    s.spawn(|| scores.iter().sum::<i32>());     // ✓ borrows scores
    s.spawn(|| scores.iter().max());            // ✓ another reader at the same time
});                                             // all scoped threads finish before this returns
println!("{scores:?}");                         // still ours
```

`thread::scope` joins every **scoped thread created through that scope's `s.spawn`** before returning. This allows those threads to borrow sufficiently long-lived local data. An ordinary `thread::spawn` call made inside the scope is still unscoped and does not get this guarantee.

The rules still apply. Several threads may read shared data when its type is `Sync`. Trying to give two scoped threads direct, overlapping mutable access fails, just like two conflicting `&mut` borrows in one thread:

```text
error[E0499]: cannot borrow `total` as mutable more than once at a time
```

To let several threads write, give each one **its own part**: `data.chunks_mut(2)` splits a `Vec` into separate mutable slices, one per thread.

## Sharing across threads: the thread-safe toolbox

These tools support shared ownership or coordinated access when their type bounds are satisfied:

| One thread | Across threads | What changes |
|---|---|---|
| `Rc<T>` | **`Arc<T>`** | the reference count is updated atomically |
| `RefCell<T>` | **`Mutex<T>`** | `lock()` waits for another thread's exclusive guard |
| `RefCell<T>` with many readers | **`RwLock<T>`** | many readers **or** one writer, waiting when needed |
| `Cell<u32>` | **`AtomicU32`** (and friends) | single numbers and flags, no lock needed |

```rust
use std::sync::{Arc, Mutex};
use std::thread;

let counter = Arc::new(Mutex::new(0)); // shared ownership + coordinated mutation
let c = Arc::clone(&counter);
let handle = thread::spawn(move || *c.lock().unwrap() += 1);
handle.join().unwrap();               // wait before checking the result
assert_eq!(*counter.lock().unwrap(), 1);
```

Like `borrow_mut()`, `lock()` returns a guard giving exclusive access; its scheduling and error behaviour differ. Dropping the guard releases the lock. `lock()` can return a poisoning error after a panic while the lock was held; this demo uses `unwrap()` because such a panic is not expected.

`Arc<T>` only makes the **reference count** atomic. Sharing its data also requires `T: Send + Sync`; wrapping a `RefCell` in `Arc` does not add those permissions. The runnable demo joins 8 workers after 1,000 protected increments each; when the workers complete successfully, the result is 8,000.

**Keep locks short.** Holding a guard while doing slow work blocks every other thread waiting for the same lock. Two threads each waiting for a lock the other holds is a *deadlock*. Rust prevents data races, but not deadlocks.

## Run it

```bash
cargo run
cargo test
```

API guarantees: [spawn](https://doc.rust-lang.org/std/thread/fn.spawn.html), [scope](https://doc.rust-lang.org/std/thread/fn.scope.html) and [Arc](https://doc.rust-lang.org/std/sync/struct.Arc.html).

Previous: [Lesson 7: Interior mutability](../07-interior-mutability/) · Next: [Lesson 9: Borrowing in patterns](../09-borrowing-in-patterns/)
