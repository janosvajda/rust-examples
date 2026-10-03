<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: The borrowing rules

## The rule in one sentence

At any moment, a value can have **many readers or one writer**, never both.

Precisely: for a given value, at any point in the program there may be **either**

- any number of shared references (`&T`), **or**
- exactly one mutable reference (`&mut T`),

and while any reference is in use, the **owner** can't move, change or drop the value either.

That's the whole rule. Everything the borrow checker does follows from it.

## How long does a borrow last?

**From where the reference is created to where it's last used**, not to the end of the block. This is called *non-lexical lifetimes*.

```rust
let mut list = vec![1, 2, 3];
let first = &list[0];     // shared borrow starts
println!("{first}");      // …and ends here, at its last use
list.push(4);             // ✓ no borrow is active any more
```

Move the `println!` below the `push` and the borrows overlap:

```text
error[E0502]: cannot borrow `list` as mutable because it is also borrowed as immutable
```

## Why the rule exists

The rule isn't arbitrary. It prevents a real class of bugs. Here's what could happen without it:

```text
  let first = &list[0];          first ──► [1, 2, 3]          (buffer full)
  list.push(4);                  the Vec needs more room: it may move everything
                                 to a new, bigger buffer and free the old one
                                 first ──► ░░░░░░░             (freed memory!)
                                 list ──► [1, 2, 3, 4]
  println!("{first}");           reads freed memory: garbage, or a crash
```

Whether the buffer actually moves depends on the memory allocator: sometimes it can grow the block in place. The compiler can't know in advance, so it must assume the worst. In C or C++ this code compiles and fails at runtime, *sometimes*, which makes it one of the hardest bugs to find. In Rust it doesn't compile at all. The demo prints whether the buffer moved on your machine.

The same reasoning explains every case below. If something can change or free the data, nobody else may be looking at it. If several parts of the code are looking, nobody may change it.

## Every case, with the real compiler error

| Situation | Allowed? | Error if not |
|---|---|---|
| Several `&` at the same time | ✓ | |
| Two `&mut` at the same time | ✗ | **E0499**: cannot borrow `score` as mutable more than once at a time |
| `&` and `&mut` at the same time | ✗ | **E0502**: cannot borrow `list` as mutable because it is also borrowed as immutable |
| Changing a `Vec` while looping over it | ✗ | **E0502** (the loop holds a `&` for its whole duration) |
| Moving a value while it's borrowed | ✗ | **E0505**: cannot move out of `text` because it is borrowed |
| A new borrow after the old one's last use | ✓ | |

"At the same time" always means *the borrows are both still going to be used*. Two borrows written one after another are fine as long as the first one is finished.

## Two cases that look wrong but compile

**Passing a `&mut` to a function (a reborrow).** Mutable references can't be copied, so you might expect `handle` to be moved into the first call:

```rust
let handle = &mut count;
add_one(handle);   // lends `*handle` to the function for the duration of the call
add_one(handle);   // ✓ `handle` is usable again
```

The compiler automatically lends a fresh, shorter borrow (`&mut *handle`) for each call. It's called a **reborrow**. But `let b = a;` between two `&mut` variables *does* move, as lesson 2 showed.

**`v.push(v.len())`.** This seems to use `v` mutably (`push`) and immutably (`len`) at the same time. It compiles because the mutable borrow for `push` doesn't really start until the arguments have been evaluated. This is called a **two-phase borrow**, and it exists precisely to allow this common pattern.

## Run it

```bash
cargo run
cargo test
```

Each forbidden case is in `src/main.rs` as a comment, with the real error underneath and the allowed version right after it.

Previous: [Lesson 2: References](../02-references/) · Next: [Lesson 4: Slices](../04-slices/)
