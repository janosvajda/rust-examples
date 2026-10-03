<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 7: Interior mutability

## The idea in one sentence

**Interior mutability** lets you change a value through a shared `&` reference, safely, by checking the borrowing rules **while the program runs** instead of at compile time.

## Why it's needed

The borrowing rules are checked by the compiler, and the compiler is cautious: it rejects some programs that would actually be fine. Two common cases:

- **A method that's a "read" from the outside, but updates something inside.** Reading a document should only need `&self`, but we also want to count how many times it was read.
- **Several owners that all need to change the same value**, like two parts of a program sharing one bank account. Shared ownership (`Rc`) only gives out `&` references.

The rule itself doesn't go away. One writer **or** many readers still applies. Only the *checking* moves to runtime.

## `Cell<T>`: for small, copyable values

```rust
struct Document { text: String, views: Cell<u32> }

fn read(&self) -> &str {
    self.views.set(self.views.get() + 1);   // ✓ changes through &self
    &self.text
}
```

`Cell` never gives out a reference to its contents. You can only copy the value out (`get`) or replace it (`set`). If nobody can hold a reference to the inside, nobody can be looking at it while it changes, so no runtime check is needed at all and it costs nothing. Use it for counters, flags and other small `Copy` values.

## `RefCell<T>`: for any value

`RefCell` hands out borrows, and **keeps count of them** while the program runs:

| Compile-time version | `RefCell` version | Returns |
|---|---|---|
| `&value` | `cell.borrow()` | a guard that acts like `&T` |
| `&mut value` | `cell.borrow_mut()` | a guard that acts like `&mut T` |

A borrow lasts until its guard is dropped, usually at the end of the statement or block. If you break the rule, for example by calling `borrow_mut()` while a `borrow()` is still active, the program **panics**:

```text
RefCell already borrowed
```

To check without panicking, use `try_borrow()` and `try_borrow_mut()`. They return an error instead. The demo and tests show both.

> The rule is the same as in lesson 3; only the timing changes. A mistake that used to be a compile error becomes a crash at runtime. So use `RefCell` only when the compile-time rules really can't express what you need, and keep each borrow as short as possible.

## `Rc<RefCell<T>>`: shared and changeable

```rust
let account = Rc::new(RefCell::new(Account { balance: 100 }));
let alice = Rc::clone(&account);
let bob = Rc::clone(&account);

alice.borrow_mut().balance -= 30;
bob.borrow_mut().balance += 50;     // balance is now 120
```

Each part does one job:
- **`Rc`** gives **shared ownership**: several owners, and the value is dropped when the last one goes away;
- **`RefCell`** gives **shared mutation**: any owner can change it, with the rules checked at runtime.

This combination is common in graph-like data, user interfaces and the Observer pattern (see `design-patterns/behavioral/observer-pattern`).

## Across threads

`Cell`, `RefCell` and `Rc` work in a **single thread only**, and the compiler stops you from sending them to another thread. The thread-safe equivalents:

| Single thread | Multiple threads |
|---|---|
| `Rc<T>` | `Arc<T>` |
| `Cell<T>` (numbers, flags) | `AtomicU32`, `AtomicBool`, … |
| `RefCell<T>` | `Mutex<T>`, or `RwLock<T>` for many readers |

A `Mutex` *waits* when the value is in use, where a `RefCell` would panic.

## The whole course in one table

| You want to… | Use |
|---|---|
| give a value away | move it (lesson 1) |
| let someone read it | `&T` (lesson 2) |
| let someone change it | `&mut T` (lesson 2) |
| lend part of a collection | `&str`, `&[T]` (lesson 4) |
| return or store a reference | lifetimes (lesson 5) |
| change through `&` (one thread) | `Cell` or `RefCell` (this lesson) |
| share ownership | `Rc` (one thread) or `Arc` (many) |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 6: Borrowing in practice](../06-borrowing-in-practice/) · Next: [Lesson 8: Threads and borrowing](../08-threads-and-borrowing/)
