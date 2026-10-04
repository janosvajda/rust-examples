<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Pointers are addresses

## The idea in one sentence

A **pointer** tells code where to find a value; Rust's **references** add rules that let safe code use that value without accessing invalid memory.

## Imagine…

Ada puts the number **1204** on a slip of paper: “the treasure is in mailbox 1204.”

The slip isn't the treasure. **Following** it and looking inside the mailbox is called **dereferencing**.

But a number alone isn't enough. Does the mailbox exist? Is the treasure still there? May Ada read it, or change it? A pointer needs those answers too.

A Rust reference is a slip with promises attached: the mailbox exists, the treasure stays there for as long as the slip is in use, and nobody swaps the treasure while Ada is looking at it. In safe Rust, the compiler enforces the borrowing rules before the program runs; libraries using `unsafe` must uphold the reference's promises too. A **raw pointer** is a bare slip, with no promises at all.

## Following one reference

```rust
let score = 42u32;
let pointer = &score;
assert_eq!(*pointer, 42);
```

`&score` **borrows** `score`: it makes a reference, without copying or moving the value. `*pointer` follows that reference to the value.

The program prints the address of `score`, and the address held inside the reference:

```text
1. A reference holds an address
    score lives at       0x16fa45cdc
    the reference holds  0x16fa45cdc
    following it gives   42
    the reference itself is 8 bytes: just the address
```

Your addresses may differ, and may change from run to run. What matters is that the two addresses are the same: the reference really is the address of `score`. An `&u32` takes eight bytes on this 64-bit machine, which is exactly the size of one address. (On this target, pointers occupy 64 bits = 8 bytes. That doesn't mean every one of those bits selects usable memory, or that the computer has 2⁶⁴ bytes of RAM.)

Not every reference is one address, though. A reference to a **slice**, a piece of an array such as `&numbers[1..3]`, also stores the slice's **length**, so it takes two words: 16 bytes here. Rust needs the length to check every index.

Printing an address as a number is fine. Going the other way, turning an arbitrary number back into a reference, isn't allowed in safe Rust: a valid reference must point into memory that really belongs to a live value. Rust calls this connection between a pointer and the memory it came from **provenance**. See [Rust's pointer documentation](https://doc.rust-lang.org/std/ptr/index.html).

## Finding an array element without searching

For `[10u32, 20, 30, 40]`, the address calculation from lesson 2 is:

```text
element 0: start + 0 × 4
element 1: start + 1 × 4
element 2: start + 2 × 4
element 3: start + 3 × 4
```

Finding element 3 doesn't mean walking through elements 0, 1 and 2. Its offset can be calculated directly.

The program computes these addresses by hand and compares them with the addresses Rust uses for `numbers[i]`:

```text
2. Indexing is arithmetic: start + index × size
    numbers[0]: start + 0 × 4 = 0x16fa45d80  (Rust used 0x16fa45d80)
    numbers[1]: start + 1 × 4 = 0x16fa45d84  (Rust used 0x16fa45d84)
    numbers[2]: start + 2 × 4 = 0x16fa45d88  (Rust used 0x16fa45d88)
    numbers[3]: start + 3 × 4 = 0x16fa45d8c  (Rust used 0x16fa45d8c)
```

They agree exactly for these valid indices. You can also calculate a number beyond the array, but that doesn't make it an element you may read. Very large calculations can overflow the integer type too. Address arithmetic alone can't keep a program safe; the next sections add the checks.

## A reference can point to another reference

```rust
let score = 42u32;
let inner = &score;
let outer = &inner;
assert_eq!(**outer, 42);
```

`outer` points to `inner`, which points to `score`. Two stars follow two slips:

```text
3. A pointer to a pointer: an address of an address
    the outer reference holds 0x16fa45ce0, where the inner one is stored
    following both gives 42
```

`inner` is itself stored in memory, at its own address, and `outer` holds that address.

**Try it:** if the treasure is 42, does copying a shared reference copy the treasure?

<details>
<summary>Show the answer</summary>

No. It copies the reference: a second slip with the same mailbox number, pointing to the same treasure. There is still only one 42. Copying the treasure itself is a separate step, for example `let copy = *pointer;`.

</details>

## A bad address isn't a predictable adventure

In C, reading outside an array, using a freed allocation or dereferencing a null pointer causes **undefined behaviour**. A crash is possible; so are apparently successful runs, corruption and security vulnerabilities. The language doesn't promise which outcome you get.

Safe Rust handles these cases differently:

| Problem | Safe Rust's rule |
|---|---|
| Index outside an array or slice | `numbers[10]` panics; `numbers.get(10)` returns `None`. |
| Reference outlives the value | The compiler rejects the invalid borrow. |
| Reference with no target | References must be non-null. Use `Option<&T>` to represent “maybe a reference.” |
| Changing a value while it's being read | The borrowing rules forbid it: while shared references to a value are in use, the value can't be changed. (A few special types, such as `Cell` and `RefCell`, allow controlled changes and check the rules in their own way; see [`std::cell`](https://doc.rust-lang.org/std/cell/index.html).) |

A **panic** reports a failure, such as `index out of bounds: the len is 4 but the index is 10`. The bounds check prevents that access from reading outside the array. Depending on the build, a panic either **unwinds** (leaves function calls, cleaning up local values) or **aborts** the process. An unwinding panic can be caught with [`catch_unwind`](https://doc.rust-lang.org/std/panic/fn.catch_unwind.html); an unhandled panic in `main` ends this run.

```rust
let numbers = [10, 20, 30, 40];
match numbers.get(10) {
    Some(value) => println!("Found {value}"),
    None => println!("That mailbox isn't in this array."),
}
```

```text
4. An index that doesn't exist
    numbers.get(10) = None: there is no mailbox 10 in this array
    (numbers[10] would panic: index out of bounds)
```

`get` returns an **`Option`**: either `Some(value)` or `None` ("there isn't one"). `Option` makes absence explicit in the type. You still have to decide what to do: calling `.unwrap()` on `None` panics, just like `numbers[10]`.

## Safety and space

A **bounds check** asks "is this index smaller than the length?" before `numbers[i]` accesses the element. If the answer is no, indexing panics. The compiler may remove a check when it proves the index valid; otherwise it generates code to enforce the rule. The exact instructions vary. A loop such as `for n in &numbers` uses an iterator instead of asking for an element by index. [Lesson 9](../09-why-rust/) measures the cost.

```text
5. "Maybe a reference" is an Option, and takes no extra space
    numbers.get(2) = Some(30), numbers.get(10) = None
    Option<&u32> is 8 bytes, the same as &u32: None is stored as address 0
```

For **`Option<&u32>`**, Rust guarantees the same size as **`&u32`**. A reference can never be **null** (address 0, which means "points nowhere"), so address 0 is free to mean `None`. This is a specific guarantee, not a rule for every `Option<T>`. See [the standard library's representation guarantees](https://doc.rust-lang.org/std/option/index.html#representation).

Rust also has `*const T` and `*mut T`, called **raw pointers**. They have none of a reference's guarantees: they can be null, or **dangling** (pointing at memory that has been freed). Following one requires an `unsafe` block, in which you, not the compiler, promise that the pointer is valid. Writing `unsafe` permits operations such as dereferencing a raw pointer, whose requirements the programmer must prove. It **doesn't switch off** the borrow checker, ordinary indexing checks or Rust's memory-access rules. An invalid dereference is still undefined behaviour. [The Rust Book's unsafe chapter](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html) explains these extra operations.

Safe Rust's guarantees rely on the `unsafe` code underneath it, in the standard library and in other libraries, being written correctly. The [Pointers and memory course](../../pointers-and-memory/) explores these types further.

## Words to remember

| Word | Meaning |
|---|---|
| **pointer** | a value that says where another value is: its address |
| **reference** (`&x`) | Rust's checked pointer: never null, always valid while in use |
| **dereferencing** (`*r`) | following a pointer to the value it points to |
| **null** | address 0, used to mean "points nowhere" |
| **dangling** | pointing at memory that no longer holds the value |
| **undefined behaviour** | anything can happen; the language makes no promise |
| **bounds check** | testing that an index is inside the array before reading |
| **raw pointer** (`*const T`, `*mut T`) | a pointer without Rust's guarantees; following one needs `unsafe` |

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1964–65** | Harold Lawson developed pointer variables for **PL/I**, making linked structures easier to express in a high-level language. Addresses and indirect addressing already existed in machine code. [IEEE Computer Society's account](https://www.computer.org/profiles/harold-lawson) |
| **2009** | Tony Hoare looked back at his earlier null-reference design and called it his **“billion-dollar mistake”** because of the resulting failures and debugging costs. [Hoare's talk](https://www.infoq.com/presentations/Null-References-The-Billion-Dollar-Mistake-Tony-Hoare/) |
| **2015** | **Rust 1.0** made a stable language available with ownership and borrowing central to its memory-safety design. [The release announcement](https://blog.rust-lang.org/2015/05/15/Rust-1.0/) |

**Fun fact:** a slip can point to a slip that points to another slip. Computers can build whole trees and graphs from this tiny idea.

## Run it

From this lesson's directory:

```bash
cargo run
cargo test
```

Watch the addresses agree, then look at the `Some` and `None` results.

Previous: [Lesson 2: Memory and addresses](../02-memory-and-addresses/) · Next: [Lesson 4: A tiny CPU](../04-a-tiny-cpu/)
