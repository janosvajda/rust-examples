<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 7: Interior mutability

## The idea in one sentence

**Interior mutability** lets a type provide controlled changes through shared `&` references. `Cell` moves or copies whole values; `RefCell` tracks borrows at runtime.

Imagine a library document with a view counter. Everyone may read the document, while its own `read` method updates the counter behind the scenes.

## Why it's needed

The borrowing rules are checked by the compiler, and the compiler is cautious: it rejects some programs that would actually be fine. Two common cases:

- **A method that's a "read" from the outside, but updates something inside.** Reading a document should only need `&self`, but we also want to count how many times it was read.
- **Several owners that all need to change the same value**, like two parts of a program sharing one bank account. When several `Rc` handles share an allocation, ordinary dereferencing provides shared access. `Rc` also has APIs for exclusive access when ownership is unique.

The outer shared references remain ordinary Rust references. The inner value changes through a carefully designed API: `Cell` avoids borrowed access to its contents through `&Cell`; `RefCell` checks the inner borrows dynamically.

## `Cell<T>`: move or copy whole values

```rust
use std::cell::Cell;

struct Document { text: String, views: Cell<u32> }

impl Document {
    fn read(&self) -> &str {
        self.views.set(self.views.get() + 1); // changes the counter through &self
        &self.text
    }
}
```

This simple counter assumes fewer than `u32::MAX` previous reads. Integer arithmetic has finite limits; a longer-lived counter needs an explicit overflow policy.

`Cell::get()` copies out a value and requires `T: Copy`, making it handy for counters and flags. But `Cell` can also hold non-`Copy` values:

```rust
use std::cell::Cell;

let label = Cell::new(String::from("Ferris"));
let old = label.replace(String::from("Ana")); // moves the old String out
assert_eq!(old, "Ferris");
assert_eq!(label.into_inner(), "Ana");        // consumes the Cell
```

Through `&Cell<T>`, these APIs move, copy or replace whole values rather than lending an ordinary reference to the contents. `take()` leaves a default value. With exclusive `&mut Cell<T>` access, `get_mut()` can return `&mut T`.

`Cell` needs **no runtime borrow counter**. That removes the counter checks used by `RefCell`; it does not mean copying or replacing arbitrary contents takes no work.

## `RefCell<T>`: for any value

`RefCell` hands out borrows, and **keeps count of them** while the program runs:

| Compile-time version | `RefCell` version | Returns |
|---|---|---|
| `&value` | `cell.borrow()` | a `Ref<T>` guard that gives shared access |
| `&mut value` | `cell.borrow_mut()` | a `RefMut<T>` guard that gives exclusive access |

A borrow lasts until its guard is dropped, usually at the end of the statement or block. If you break the rule, for example by calling `borrow_mut()` while a `borrow()` is still active, the program **panics**:

```text
RefCell already borrowed
```

To check without panicking, use `try_borrow()` and `try_borrow_mut()`. They return an error instead. The demo and tests show both.

> A `RefCell` guard's borrow lasts **until the guard is dropped**, even after its last visible use. Use a small block or `drop(guard)` to end it early. A conflicting `borrow_mut()` panics; a panic may unwind and can sometimes be caught, so it is not necessarily a process crash. Prefer `try_borrow_mut()` when a conflict should be handled as an ordinary error.

## `Rc<RefCell<T>>`: shared and changeable

```rust
use std::cell::RefCell;
use std::rc::Rc;

struct Account { balance: i64 }
let account = Rc::new(RefCell::new(Account { balance: 100 }));
let alice = Rc::clone(&account);
let bob = Rc::clone(&account);

alice.borrow_mut().balance -= 30;
bob.borrow_mut().balance += 50;     // balance is now 120
```

Each part does one job:

- **`Rc`** gives **shared ownership**: several strong handles own the allocation together, and the value is dropped when the last strong handle is dropped;
- **`RefCell`** gives **shared mutation**: any owner can change it, with the rules checked at runtime.

This combination is common in graph-like data, user interfaces and the Observer pattern (see `design-patterns/behavioral/observer-pattern`).

## Across threads

**Moving ownership** and **sharing references** are different. `Cell<T>` and `RefCell<T>` are `Send` when `T` is `Send`, so you can move ownership to another thread. Neither is `Sync`, so you cannot share ordinary references to one of them between threads. `Rc<T>` is neither `Send` nor `Sync`.

For example, moving this `Cell<String>` is allowed:

```rust
use std::cell::Cell;
use std::thread;

let label = Cell::new(String::from("Ferris"));
let handle = thread::spawn(move || label.into_inner());
assert_eq!(handle.join().unwrap(), "Ferris");
```

For **shared access across threads**, use tools with the appropriate `Send` and `Sync` bounds:

| Single thread | Multiple threads |
|---|---|
| `Rc<T>` | `Arc<T>` |
| `Cell<u32>`, `Cell<bool>`, … | `AtomicU32`, `AtomicBool`, … |
| `RefCell<T>` | `Mutex<T>`, or `RwLock<T>` for many readers |

`Mutex::lock()` waits when another thread holds the lock; `try_lock()` can return an error immediately. Locks can deadlock if used incorrectly. `Arc` makes ownership counting thread-safe; it does not make arbitrary contents thread-safe. For example, `Arc<RefCell<T>>` cannot be shared between threads.

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

Precise APIs: [Cell](https://doc.rust-lang.org/std/cell/struct.Cell.html), [RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html) and [interior mutability](https://doc.rust-lang.org/std/cell/index.html).

Previous: [Lesson 6: Borrowing in practice](../06-borrowing-in-practice/) · Next: [Lesson 8: Threads and borrowing](../08-threads-and-borrowing/)
