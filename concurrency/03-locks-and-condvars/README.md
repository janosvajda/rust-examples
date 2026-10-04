<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Locks and condition variables

## The idea in one sentence

When threads must share data instead of sending it, a **lock** makes them take turns, and a **condition variable** lets them sleep until there's something to do.

## `Arc<Mutex<T>>`: the standard way to share and change

```rust
let counter = Arc::new(Mutex::new(0));

let counter2 = Arc::clone(&counter);
thread::spawn(move || {
    *counter2.lock().unwrap() += 1;
});
```

Each part has one job:
- **`Arc`** gives **shared ownership**: every thread gets its own handle to the same data. Moving a plain `Mutex` into several threads doesn't work, because the first thread takes it:

  ```text
  error[E0382]: use of moved value: `counter`
  ```

- **`Mutex`** gives **one-at-a-time access**. `lock()` waits for its turn and returns a **guard**. The data can only be reached through the guard, and the lock is **released when the guard is dropped**, so you can't forget to unlock.

## Poisoning: when a thread panics while holding the lock

If a thread panics halfway through changing the data, the data may be left inconsistent. Rust marks the `Mutex` as **poisoned**, and every later `lock()` returns an error. Calling `.unwrap()` on it panics:

```text
called `Result::unwrap()` on an `Err` value: PoisonError { .. }
```

The demo withdraws 30 from a balance of 100, then crashes before finishing the update. The mutex is poisoned and the balance shows 70. If you decide the data is still usable, `PoisonError::into_inner()` gives it to you anyway.

## `RwLock`: many readers or one writer

```rust
let config = RwLock::new(String::from("theme=dark"));
config.read().unwrap()      // any number of readers at the same time
config.write().unwrap()     // one writer, and no readers meanwhile
```

This is the borrowing rule ("many `&` or one `&mut`"), checked at runtime across threads. Use it for data that's **read often and changed rarely**, such as configuration or caches.

## `Condvar`: sleep until something changes

A consumer waiting for jobs shouldn't loop asking "anything yet? anything yet?", wasting a CPU core. A **condition variable** lets it sleep until a producer wakes it:

```rust
// consumer
jobs = not_empty.wait_while(jobs, |jobs| jobs.is_empty()).unwrap();

// producer
jobs.lock().unwrap().push_back(job);
not_empty.notify_one();
```

- `wait_while` **releases the lock while sleeping**, so the producer can add jobs, and takes it back when woken.
- Always wait **in a loop that re-checks the condition**. A thread can occasionally wake up even though nothing changed (a *spurious wakeup*). `wait_while` does the loop for you.

## `Barrier`: wait until everyone is ready

```rust
let barrier = Arc::new(Barrier::new(3));
barrier.wait();     // blocks until 3 threads have called wait()
```

Useful when work happens in phases: every thread must finish phase 1 before any starts phase 2. The demo's test checks that all three "phase 1" lines come before any "phase 2" line.

## File locks: coordinating with other programs

A `Mutex` only protects data inside one program. Sometimes several **programs** share a file: two copies of a tool writing the same log, a background job and a command-line tool updating the same data file. For that, the operating system offers **file locks**, and `File` has them built in:

```rust
fn append_line(path: &Path, line: &str) -> io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.lock()?;                    // wait until no one else holds a lock
    writeln!(file, "{line}")?;
    Ok(())                           // the lock is released when the file is closed
}
```

| Method | Does |
|---|---|
| `lock()` | exclusive lock: waits until nobody else holds any lock on the file |
| `lock_shared()` | shared lock: many readers at once, but no exclusive lock meanwhile, like `RwLock` |
| `try_lock()` / `try_lock_shared()` | the same, but returns an error right away instead of waiting |
| `unlock()` | release early; closing the file releases it too |

The demo has four threads that each open the file **separately**, exactly as separate programs would, and append 25 lines each. All 100 lines arrive complete. While one handle holds the lock, another handle's `try_lock()` reports `Err("WouldBlock")`.

Two things to know:
- File locks are **advisory** on most systems: they only coordinate programs that also take the lock. A program that ignores locking can still write to the file.
- The lock belongs to the open file, not to a thread, so the usual way to release it is simply to let the `File` go out of scope.

## Deadlocks

Rust prevents data races, but **not deadlocks**. If thread A holds lock 1 and waits for lock 2, while thread B holds lock 2 and waits for lock 1, both wait forever. The standard defences:
- **always take locks in the same order** in every thread;
- **hold locks briefly**: do the slow work after releasing them;
- **prefer channels** (lesson 2) when the design allows it.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: Message passing](../02-message-passing/) · Next: [Lesson 4: Atomics](../04-atomics/)
