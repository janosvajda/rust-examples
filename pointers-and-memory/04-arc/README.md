<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: `Arc<T>`

## The idea in one sentence

`Arc` is `Rc` for threads: it counts its owners in the same way, but changes the count with **atomic** instructions, so several threads can share one value and the last one to finish frees it.

## Why `Rc` isn't enough

```rust
let shared = Rc::new(5);
std::thread::spawn(move || println!("{shared}"));
```

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
```

`Rc` changes its counter with ordinary instructions. If two threads cloned or dropped the same `Rc` at the same moment, both could read the old count, and one update would be lost. The value could then be freed while still in use, or never freed at all. The compiler doesn't allow the risk: `Rc` doesn't implement `Send`, so it can't move to another thread ([ownership lesson 8](../../ownership-and-borrowing/08-threads-and-borrowing/) explains `Send`).

`Arc` uses **atomic** operations for its counter: special CPU instructions that read, change and write a number as one indivisible step, even when several cores do it at once. That costs a little more than `Rc`'s plain counter, which is why both exist. Use `Rc` in single-threaded code and `Arc` across threads.

## Counting owners

```text
at start 1, after a clone 2, after dropping it 1
```

Exactly like `Rc`: `Arc::clone` adds an owner without copying the value.

## Read-only data, shared without copying

```rust
let numbers = Arc::new((1..=100).collect::<Vec<i32>>());
for i in 0..thread_count {
    let numbers = Arc::clone(&numbers);           // one more owner, for this thread
    thread::spawn(move || numbers[start..end].iter().sum::<i32>());
}
```

Four threads each sum a quarter of **one** vector. Each thread owns an `Arc`, so the vector lives as long as any thread still needs it, and is freed when the last one finishes. A test checks that once all threads are done, the count is back to 1.

## Changing shared data: `Arc<Mutex<T>>` and `Arc<RwLock<T>>`

Like `Rc`, an `Arc` only gives shared, read-only access. To change the value, put a lock inside:

```rust
let counter = Arc::new(Mutex::new(0));
// in each thread:
*counter.lock().expect("no thread panicked while holding it") += 1;
```

```text
8 × 1000 increments = 8000
```

| You need | Use |
|---|---|
| to share read-only data | `Arc<T>` |
| to share data that changes | `Arc<Mutex<T>>`: one thread at a time |
| to share data that's read often and written rarely | `Arc<RwLock<T>>`: many readers **or** one writer |
| to share a single number or flag | `Arc<AtomicUsize>`, `Arc<AtomicBool>`: no lock at all |

`Arc` handles **ownership**: who keeps the value alive. The lock handles **access**: who may change it right now. The two always come as a pair. The [concurrency course](../../concurrency/) goes deeper: [locks](../../concurrency/03-locks-and-condvars/), [atomics](../../concurrency/04-atomics/), and channels as an alternative to sharing at all.

## The family so far

| | One owner | Several owners, one thread | Several owners, several threads |
|---|---|---|---|
| read-only | `Box<T>` | `Rc<T>` | `Arc<T>` |
| changeable | `Box<T>` (via `&mut`) | `Rc<RefCell<T>>` | `Arc<Mutex<T>>` / `Arc<RwLock<T>>` |
| link back, without owning | | `rc::Weak<T>` | `sync::Weak<T>` |

`Arc` also has a `Weak`, `std::sync::Weak`, and it solves the same cycle problem as in [lesson 3](../03-rc-and-weak/).

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: Rc and Weak](../03-rc-and-weak/) · Next: [Lesson 5: Your own smart pointer](../05-your-own-smart-pointer/)
