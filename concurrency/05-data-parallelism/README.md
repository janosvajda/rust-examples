<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Data parallelism with rayon

## The idea in one sentence

**rayon** turns an ordinary iterator chain into a parallel one by changing `.iter()` to `.par_iter()`, splitting the work across every CPU core, with no threads, channels or locks to manage yourself.

## One word changes

```rust
(0..limit).filter(|&n| is_prime(n)).count()                     // one core
(0..limit).into_par_iter().filter(|&n| is_prime(n)).count()     // every core
```

Measured on an 8-core Apple M2, release build, counting primes below 3,000,000:

```text
sequential     307.382 ms
parallel        60.083 ms
```

About 5× faster. Not 8×, because coordinating threads has a cost and the cores don't all run at full speed together. Speed-ups depend on the machine and the work.

| Sequential | Parallel |
|---|---|
| `.iter()` | `.par_iter()` |
| `.iter_mut()` | `.par_iter_mut()` |
| `.into_iter()` | `.into_par_iter()` |
| `.sort()` / `.sort_unstable()` | `.par_sort()` / `.par_sort_unstable()` |

Most iterator methods (`map`, `filter`, `sum`, `count`, `collect`, `any`…) work the same way on parallel iterators. `collect` even **keeps the original order**: results come back in the same positions as the inputs.

## How it works: work stealing

rayon keeps one thread per core in a pool. It splits the data in half, then in half again, and threads that run out of work **steal** pieces from busy ones. So the load balances itself, even when some items take much longer than others.

You can split work yourself with **`rayon::join`**, which runs two closures, possibly in parallel, and waits for both:

```rust
let (left_sum, right_sum) = rayon::join(|| parallel_sum(left), || parallel_sum(right));
```

That's how parallel sorts and searches work internally: split, solve both halves at once, combine.

## No data races, guaranteed

The borrowing rules apply inside parallel closures. A parallel `for_each` may call the closure from several threads at once, so the closure must be `Fn`, which means it can't change captured variables:

```rust
let mut total = 0;
data.par_iter().for_each(|x| total += x);
```

```text
error[E0596]: cannot borrow `total` as mutable, as it is a captured variable in a `Fn` closure
```

Use `.sum()`, `.reduce()` or `.collect()`, which combine results safely, or an atomic (lesson 4). (Closures course, lesson 2, explains `Fn`.)

## When parallelism doesn't help

```text
tiny seq         0.000 ms
tiny par         0.029 ms
```

For 100 trivial items, coordinating threads costs more than it saves. rayon pays off when there's **a lot of data or expensive work per item**. Measure before and after, and always time with `--release`: debug builds are many times slower and distort the comparison.

**Threads, async or rayon?**

| Work | Use |
|---|---|
| the same CPU-heavy operation on many items | **rayon** |
| a few long-running, different jobs | **threads** (lessons 1–3) |
| lots of waiting: network, files, timers | **async** (async course) |

## Run it

```bash
cargo run --release     # --release for meaningful timings
cargo test
```

Previous: [Lesson 4: Atomics](../04-atomics/) · Back to the [course overview](../)
