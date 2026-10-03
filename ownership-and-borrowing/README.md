<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Ownership and Borrowing

<p align="center">
  <a href="ownership-and-borrowing-comic.png">
    <img src="ownership-and-borrowing-comic.png" alt="Comic poster summarising all twelve ownership and borrowing lessons" width="100%">
  </a>
</p>

Ownership and borrowing are how Rust manages memory **without a garbage collector and without manual `free`**, and how it rules out whole classes of bugs before the program ever runs: use-after-free, double free, dangling pointers and data races.

This folder is a course in twelve lessons. Each one is a small runnable program with a README that explains one idea simply and precisely. Every compiler error quoted in these READMEs is the real message from Rust 1.99, and the code that triggers it is in the lesson's `src/main.rs` as a comment.

## All the rules on one page

**Ownership**
1. Every value has exactly one owner.
2. Ownership can be moved; the old owner can't be used after that.
3. When the owner goes out of scope, the value is dropped.

**Borrowing**
1. A reference lets you use a value without owning it. `&T` reads, `&mut T` reads and changes.
2. At any moment a value can have **either** any number of `&T` **or** exactly one `&mut T`.
3. A borrow lasts until the reference's last use.
4. A reference can never outlive the value it points to.

Everything else in this course is these rules applied to real code.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [Ownership and moves](01-ownership-and-moves/) | move vs copy vs clone, functions taking and returning ownership, when values are dropped |
| 2 | [References](02-references/) | `&` and `&mut`, dereferencing with `*`, methods taking `&self`, `&mut self` or `self` |
| 3 | [The borrowing rules](03-borrowing-rules/) | "many readers or one writer", how long a borrow lasts, *why* the rule exists, reborrows |
| 4 | [Slices](04-slices/) | `&str` and `&[T]`, why functions should take them, byte positions in UTF-8 text |
| 5 | [Lifetimes](05-lifetimes/) | dangling references, `'a` annotations, the elision rules, structs holding references, `'static` |
| 6 | [Borrowing in practice](06-borrowing-in-practice/) | struct fields, `split_at_mut`, the three kinds of loop, closures, `mem::take` |
| 7 | [Interior mutability](07-interior-mutability/) | `Cell`, `RefCell` and `Rc<RefCell<T>>`: borrow checking at runtime |
| 8 | [Threads and borrowing](08-threads-and-borrowing/) | why Rust has no data races, `Send` and `Sync`, scoped threads, `Arc`, `Mutex`, `RwLock` |
| 9 | [Borrowing in patterns](09-borrowing-in-patterns/) | `match` and `if let` on references, `ref`, partial moves, `as_ref` / `as_mut` / `as_deref` |
| 10 | [Advanced lifetimes](10-advanced-lifetimes/) | method results tied to data instead of `self`, several lifetimes, `dyn Trait + 'a`, `impl Trait + use<>`, `for<'a>` |
| 11 | [Cow and the borrowing traits](11-cow-and-borrowing-traits/) | `Cow`, why `HashMap<String, _>` accepts `&str`, `AsRef`, `Deref` and deref coercion |
| 12 | [Where the borrow checker is too strict](12-borrow-checker-limits/) | correct code that's still rejected, self-referential structs, and the standard workarounds |

Read them in order. Each lesson builds on the one before. Lessons 1–7 are the essentials; 8–12 go deeper.

## The compiler errors you'll meet, and what they mean

| Error | In plain words | Lesson |
|---|---|---|
| **E0382** use / borrow of moved value | you gave this value away earlier | 1 |
| **E0596** cannot borrow as mutable | the variable isn't `mut`, or you only have a `&` | 2 |
| **E0499** cannot borrow as mutable more than once at a time | two writers at once | 3 |
| **E0502** cannot borrow as mutable because it is also borrowed as immutable | a writer while someone is reading (or the reverse) | 3, 4, 6 |
| **E0505** cannot move out because it is borrowed | giving a value away while it's lent out | 3 |
| **E0597** does not live long enough | the reference would outlive the value | 5 |
| **E0106** missing lifetime specifier | say which input a returned reference comes from | 5 |
| **E0515** cannot return reference to local variable | the value dies when the function returns; return it owned | 5 |
| **E0507** cannot move out of … which is behind a reference / of index of `Vec` | you only borrowed it; borrow, clone, or use `mem::take` | 6, 9 |
| **E0506** cannot assign to … because it is borrowed | changing a value something still points at | 10 |
| **E0373** closure may outlive the current function | a thread borrows local data; use `move` or `thread::scope` | 8 |
| **E0277** … cannot be sent / shared between threads safely | `Rc` / `RefCell` across threads; use `Arc` / `Mutex` | 8 |
| lifetime may not live long enough | a `Box<dyn Trait>` captures a borrow; add `+ 'a` | 10 |

`rustc --explain E0502` (or any other code) prints a longer explanation with examples.

## Run a lesson

```bash
cd ownership-and-borrowing/03-borrowing-rules
cargo run
cargo test
```

Or from the repository root: `cargo run -p borrowing-rules`. The package names are the folder names without the number.

To see an error for yourself, uncomment one of the marked lines in a lesson's `src/main.rs` and run `cargo build`.
