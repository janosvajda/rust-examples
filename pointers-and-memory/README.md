<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Pointers and memory

Where do Rust values live, who owns them, and who frees them? This course goes from the stack and the heap, through the smart pointers `Box`, `Rc`, `Weak` and `Arc`, to writing your own pointer type and using raw pointers responsibly.

Every compiler error quoted in these READMEs is the real output of Rust 1.99.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [The stack and the heap](01-stack-and-heap/) | where values live, `size_of`, what a move really copies, why big data goes on the heap |
| 2 | [Box](02-box/) | one owner on the heap: recursive types, trait objects, big values |
| 3 | [Rc and Weak](03-rc-and-weak/) | several owners in one thread, reference cycles that **leak**, and `Weak` to prevent them |
| 4 | [Arc](04-arc/) | several owners across threads, `Arc<Mutex>`, `Arc<RwLock>` |
| 5 | [Your own smart pointer](05-your-own-smart-pointer/) | `Deref`, `Drop`, drop order, and RAII: resources that clean themselves up |
| 6 | [Raw pointers and unsafe](06-raw-pointers-and-unsafe/) | when raw pointers are needed, writing sound `unsafe` code, `// SAFETY:` comments, Miri, and a homemade `Rc` |

## The whole course in one table

| | One owner | Several owners, one thread | Several owners, several threads | No owner |
|---|---|---|---|---|
| read-only | `Box<T>` | `Rc<T>` | `Arc<T>` | `&T` (borrowed), `*const T` (raw) |
| changeable | `Box<T>` | `Rc<RefCell<T>>` | `Arc<Mutex<T>>`, `Arc<RwLock<T>>` | `&mut T` (borrowed), `*mut T` (raw) |
| link back, without owning | | `rc::Weak<T>` | `sync::Weak<T>` | |

## Related courses

This course builds on [Ownership and borrowing](../ownership-and-borrowing/): lesson 7 there covers `Cell` and `RefCell`, and lesson 8 covers `Send` and `Sync`. For sharing between threads in depth, see [Concurrency](../concurrency/). For raw pointers in their natural habitat, C libraries and hardware, see [no_std and bare-metal Rust](../no-std-and-bare-metal/).

## Run a lesson

```bash
cd pointers-and-memory/03-rc-and-weak
cargo run
cargo test
```
