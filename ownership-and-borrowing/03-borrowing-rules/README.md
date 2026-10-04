<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: The borrowing rules

## The rule in one sentence

For ordinary data, allow **many readers or one exclusive writer** while those accesses are needed.

Picture a shared drawing: several friends may look at it together. If one friend is changing it with a pencil, everyone else must wait for that exclusive access to finish.

For an ordinary region of data, the basic access choices are:

- any number of shared references (`&T`), **or**
- exactly one mutable reference (`&mut T`),

While a shared borrow is active, its owner cannot directly mutate, move or drop the borrowed value. While an exclusive mutable borrow is active, the owner cannot access that region through a conflicting path.

Two separate fields or disjoint slices can each have their own mutable reference (lesson 6). A reborrow temporarily limits the original reference's access. Interior-mutability types control changes through shared references using their own safe APIs (lesson 7).

## How long does a borrow last?

**For as long as any use requires it.** In the simple example below, that ends at the reference's last use, before the block ends. This more flexible analysis is called *non-lexical lifetimes*.

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

Copies and derived references also count. If `let another = first;` is used later, the borrow stays active for that use too. A destructor can use a stored reference when its holder is dropped, even after the holder's last visible use. Lifetime annotations and function signatures can also require a longer borrow.

**A little history:** Rust 1.31 introduced non-lexical lifetimes for Rust 2018 programs, allowing many borrows to finish before the closing brace. [Release announcement](https://blog.rust-lang.org/2018/12/06/Rust-1.31-and-rust-2018/)

## Why the rule exists

The rule isn't arbitrary. It prevents a real class of bugs. Here's what could happen without it:

```text
  let first = &list[0];          first ──► [1, 2, 3]          (buffer full)
  list.push(4);                  the Vec needs more room: it may move everything
                                 to a new, bigger buffer and free the old one
                                 first ──► ░░░░░░░             (freed memory!)
                                 list ──► [1, 2, 3, 4]
  println!("{first}");           dereferences freed memory: undefined behaviour
```

Whether the buffer moves depends on the allocator: sometimes it can grow in place. Dereferencing a pointer into the freed buffer would be **undefined behaviour**; a crash or incorrect value is only one possible outcome.

Rust checks the borrowing contract of `push(&mut self, ...)`, rather than predicting what this allocator will do. The conflicting borrow is rejected even when you believe there is enough spare capacity. The demo fills the vector's actual reported capacity before pushing and prints whether its buffer address changed.

This is why ordinary shared and exclusive access must not conflict. Special APIs can provide disjoint mutable slices, and interior-mutability types support controlled shared mutation.

## Common cases, with compiler errors

| Situation | Allowed? | Error if not |
|---|---|---|
| Several `&` at the same time | ✓ | |
| Two `&mut` at the same time | ✗ | **E0499**: cannot borrow `score` as mutable more than once at a time |
| `&` and `&mut` at the same time | ✗ | **E0502**: cannot borrow `list` as mutable because it is also borrowed as immutable |
| Pushing to a `Vec` while iterating over `&v` | ✗ | **E0502** (the shared iterator still needs its borrow) |
| Moving a value while it's borrowed | ✗ | **E0505**: cannot move out of `text` because it is borrowed |
| A new borrow after the old one's last use | ✓ | |

Here, "at the same time" means the borrows' required lifetimes overlap. Two borrows written one after another are fine when the first is finished; stored references and destructors can extend the required lifetime.

## Two cases that look wrong but compile

**Passing a `&mut` to a function (a reborrow).** Mutable references can't be copied, so you might expect `handle` to be moved into the first call:

```rust
let handle = &mut count;
add_one(handle);   // lends `*handle` to the function for the duration of the call
add_one(handle);   // ✓ `handle` is usable again
```

Because `add_one` specifically expects `&mut i32`, the compiler lends a fresh, shorter borrow (`&mut *handle`) for each call. This is a **reborrow**. Passing the same reference to a generic by-value parameter can move it instead; automatic reborrowing is not a rule for every function call. An unannotated `let b = a;` also moves the mutable reference.

**`v.push(v.len())`.** This seems to use `v` mutably (`push`) and immutably (`len`) at the same time. The compiler first **reserves** the implicit mutable borrow for the method receiver. During that phase, a shared read such as `len()` is allowed; exclusive access is **activated** when `push` is called. This **two-phase borrow** supports the common pattern. An explicit `&mut v` does not automatically get the same treatment.

## Run it

```bash
cargo run
cargo test
```

Each forbidden case is in `src/main.rs` as a comment, with the real error underneath and the allowed version right after it.

Library guarantees: [Vec capacity and growth](https://doc.rust-lang.org/std/vec/struct.Vec.html).

Previous: [Lesson 2: References](../02-references/) · Next: [Lesson 4: Slices](../04-slices/)
