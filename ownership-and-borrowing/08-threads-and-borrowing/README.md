<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 8: Threads and borrowing

## The idea in one sentence

The borrowing rules you already know are exactly what make threads safe in Rust: **"many readers or one writer" is the rule that prevents data races.**

## Data races, and why Rust doesn't have them

A **data race** happens when two threads access the same memory at the same time, at least one of them writes, and nothing coordinates them. The result depends on timing: values get lost or corrupted, differently on every run.

Look at the conditions again: *two accesses at once, one of them a write*. That's precisely what the borrowing rules forbid. A `&mut` can't exist alongside any other reference, so two threads can never write to the same value, or read it while it's being written, without some kind of lock. **In safe Rust, a data race is a compile error.**

## `Send` and `Sync`

Two traits tell the compiler what may cross between threads. You almost never implement them yourself: the compiler works them out from what a type contains.

| Trait | Means | Not implemented by |
|---|---|---|
| **`Send`** | a value can be **moved** to another thread | `Rc<T>` |
| **`Sync`** | a value can be **shared** between threads (`&T` is `Send`) | `Rc<T>`, `Cell<T>`, `RefCell<T>` |

`Rc`, `Cell` and `RefCell` keep counters that aren't updated atomically. Two threads updating them at once could corrupt them, so the compiler stops you:

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
error[E0277]: `RefCell<i32>` cannot be shared between threads safely
```

## Problem 1: `thread::spawn` can't borrow

```rust
let names = vec![String::from("Ana")];
thread::spawn(|| println!("{names:?}"));   // ✗
```

```text
error[E0373]: closure may outlive the current function, but it borrows `names`,
              which is owned by the current function
```

A spawned thread may keep running after the function that started it returns, and by then `names` would be dropped. Two fixes:

1. **`move`**: give the thread ownership, `thread::spawn(move || ...)`.
2. **Scoped threads**: if you want to *borrow*, use `thread::scope`.

## Scoped threads: borrowing across threads

```rust
let scores = vec![70, 85, 90, 100];
thread::scope(|s| {
    s.spawn(|| scores.iter().sum::<i32>());     // ✓ borrows scores
    s.spawn(|| scores.iter().max());            // ✓ another reader at the same time
});                                             // all threads are joined here
println!("{scores:?}");                         // still ours
```

`thread::scope` guarantees every thread started inside it has finished before it returns. Borrowed data is therefore guaranteed to outlive the threads, so ordinary borrowing works.

The rules still apply. Several threads may **read** the same data. Two threads **changing** the same value fail exactly like two `&mut` in one thread:

```text
error[E0499]: cannot borrow `total` as mutable more than once at a time
```

To let several threads write, give each one **its own part**: `data.chunks_mut(2)` splits a `Vec` into separate mutable slices, one per thread.

## Sharing across threads: the thread-safe toolbox

Lesson 7's single-thread tools each have a thread-safe twin:

| One thread | Across threads | What changes |
|---|---|---|
| `Rc<T>` | **`Arc<T>`** | the reference count is updated atomically |
| `RefCell<T>` | **`Mutex<T>`** | a second user **waits** for the lock instead of panicking |
| `RefCell<T>` with many readers | **`RwLock<T>`** | many readers **or** one writer, waiting when needed |
| `Cell<u32>` | **`AtomicU32`** (and friends) | single numbers and flags, no lock needed |

```rust
let counter = Arc::new(Mutex::new(0));          // shared ownership + shared mutation
let c = Arc::clone(&counter);
thread::spawn(move || *c.lock().unwrap() += 1);
```

`lock()` is the thread-safe version of `borrow_mut()`. It returns a guard, and the lock is released when the guard is dropped. The demo runs 8 threads × 1,000 increments, and the result is always exactly 8,000.

**Keep locks short.** Holding a guard while doing slow work blocks every other thread waiting for the same lock. Two threads each waiting for a lock the other holds is a *deadlock*. Rust prevents data races, but not deadlocks.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 7: Interior mutability](../07-interior-mutability/) · Next: [Lesson 9: Borrowing in patterns](../09-borrowing-in-patterns/)
