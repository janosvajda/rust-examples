<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: Atomics

## The idea in one sentence

An **atomic** is a single number or flag that many threads can update at once **without a lock**, because the CPU guarantees each operation happens in one indivisible step.

## Why `count += 1` isn't safe, and `fetch_add` is

`count += 1` is really three steps: **read** the value, **add** one, **write** it back. If two threads both read `5` before either writes, both write `6`, and an increment is lost. Rust won't even let you write that to a shared plain integer.

```rust
let counter = AtomicUsize::new(0);
counter.fetch_add(1, Ordering::Relaxed);    // read + add + write as ONE step
```

The demo runs 8 threads × 100,000 increments and always gets exactly 800,000.

| Atomic type | Typical use |
|---|---|
| `AtomicUsize`, `AtomicU64`, `AtomicI32`… | counters, statistics, IDs |
| `AtomicBool` | flags: "stop", "ready", "initialised" |
| `AtomicPtr` | low-level lock-free data structures |

**Atomic or `Mutex`?** Atomics for a **single** number or flag. A `Mutex` as soon as several values must change **together** consistently: atomics can't make two separate updates appear as one.

## `compare_exchange`: change it only if nobody else did

```rust
match highest.compare_exchange(current, value, Ordering::Relaxed, Ordering::Relaxed) {
    Ok(_) => …,                    // it was still `current`, and now it's `value`
    Err(actual) => current = actual, // another thread changed it first: retry with the new value
}
```

*"Set it to the new value, but only if it still has the value I last saw."* If another thread got there first, you get the up-to-date value back and try again. This compare-and-swap loop is the building block of all lock-free algorithms. (For a maximum specifically, `fetch_max` does it in one call. The demo writes it out to show how.)

## `Ordering`: what the other threads are guaranteed to see

Modern CPUs and compilers **reorder** memory operations for speed. A thread might see another thread's writes in a different order than they happened in the code. Each atomic operation takes an `Ordering` that limits this:

| Ordering | Guarantees | Use for |
|---|---|---|
| `Relaxed` | only that this one operation is atomic | counters and statistics, where only the final value matters |
| `Release` (on a store) + `Acquire` (on a load) | everything written **before** the Release store is visible **after** the Acquire load that sees it | publishing data: "I've filled the buffer, here's the ready flag" |
| `SeqCst` | all threads see all `SeqCst` operations in one single order | the safe default when you're unsure |

The demo's publish pattern:

```rust
// writer thread
data.store(42, Ordering::Relaxed);          // 1. write the data
ready.store(true, Ordering::Release);       // 2. publish

// reader thread
while !ready.load(Ordering::Acquire) {}     // 3. wait for the flag
data.load(Ordering::Relaxed)                // 4. guaranteed to be 42
```

With `Relaxed` on the flag instead, the reader could see `ready == true` and still read the **old** data on some CPUs. Release/Acquire is what links step 2 to step 3.

**Practical advice:** use `Relaxed` for simple counters and flags that don't protect other data. Use `Release`/`Acquire` when a flag announces that *other* data is ready. If in doubt, use `SeqCst`, or a `Mutex`. Lock-free code is famously hard to get right.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: Locks and condition variables](../03-locks-and-condvars/) · Next: [Lesson 5: Data parallelism with rayon](../05-data-parallelism/)
