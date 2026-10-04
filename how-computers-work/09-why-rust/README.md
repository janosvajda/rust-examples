<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 9: Why Rust

## The idea in one sentence

For fifty years, programs that needed full control of memory and top speed were written in C and C++, which leave memory safety to the programmer; Rust gives the same control and speed, while the compiler rejects most memory bugs before the program ever runs.

## Imagine…

Imagine two workshops with the same powerful machines.

In the first workshop, there are no guards on the saws and no rules. Skilled workers do amazing work, quickly. But even the best workers sometimes get tired, and when something goes wrong, it goes very wrong.

The second workshop has the same machines, just as fast, but the saws have guards that **check every cut before it starts**. Some cuts are refused: "your hand would be in the way". Annoying at first. But the work comes out the same, at the same speed, and nobody gets hurt. And for the rare special job, there's a key that removes a guard, `unsafe`, used carefully and on purpose.

C is the first workshop. Rust is the second.

## Precisely

### What C gave us, and what it cost

C (1972) let programmers work directly with addresses, pointers and memory, everything in lessons 2–8 of this course, and compiled to fast machine code. Operating systems, databases, browsers and games were written in it, and in C++, which kept the same model.

But the language **trusts the programmer** with every memory access. Nothing stops a program from:

| Bug | What happens | A famous case |
|---|---|---|
| reading or writing **past the end** of a buffer | other data is read, or overwritten | the **Morris worm** (1988) used a buffer overflow in a network service to spread to thousands of computers; **Heartbleed** (2014) read past a buffer in OpenSSL and leaked servers' secrets |
| using memory **after it was freed** | the memory now belongs to something else | a common source of browser vulnerabilities |
| following a **dangling** or **null** pointer | crashes, or worse | Tony Hoare's "billion-dollar mistake" ([lesson 3](../03-pointers-are-addresses/)) |
| two threads changing the same data **at once** | random corruption that's hard to reproduce | |

These aren't the mistakes of bad programmers. Experienced teams make them at scale:
- **Microsoft** reported in 2019 that about **70%** of the security vulnerabilities it fixed each year were memory-safety bugs.
- **Google's Chromium** team reported the same figure, about 70%, for serious security bugs in 2020.

### What Rust checks

The same bugs, written in Rust. These are the real compiler errors, from Rust 1.99:

```rust
let names = vec![String::from("Ada")];
drop(names);                           // the memory is freed here
println!("{}", names.len());           // use after free
```

```text
error[E0382]: borrow of moved value: `names`
```

```rust
for n in &numbers {
    numbers.push(*n);                  // changing a list while reading it
}
```

```text
error[E0502]: cannot borrow `numbers` as mutable because it is also borrowed as immutable
```

In C++, that `push` may move the list to new memory while the loop is still reading the old, freed memory.

```rust
fn get() -> &'static str {
    let text = String::from("hi");
    &text                              // a pointer to something about to disappear
}
```

```text
error[E0515]: cannot return reference to local variable `text`
```

Reading past the end is caught when the program runs, because the length isn't known before then. Here's the heartbeat bug from Heartbleed, in Rust:

```rust
fn heartbeat(payload: &[u8], claimed_length: usize) -> Result<&[u8], String> {
    payload.get(..claimed_length).ok_or(format!("refused: …"))
}
```

```text
honest client (2 bytes, claims 2):    Ok("hi")
attacker (2 bytes, claims 64000):     Err("refused: claimed 64000 bytes, but only 2 were sent")
```

OpenSSL's C code copied `claimed_length` bytes without checking, and sent back up to 64 KB of whatever lay beyond: passwords, private keys. In Rust, the attacker gets an error. Even the shorter `&payload[..claimed_length]` would stop the program rather than leak memory.

Data races between threads are compile errors too ([Why Rust fits AI-assisted development](../../software-engineering-with-ai/05-why-rust-fits-ai/) shows the real error).

### …at the same speed

The checks are designed to cost nothing at run time, or as close to nothing as possible. Adding up 50 million numbers, three ways (`cargo run --release`, Apple M2, the best of five runs):

```text
iterator (safe, idiomatic):        6.31 ms
indexing (safe, checked):          6.34 ms
get_unchecked (unsafe, like C):    6.34 ms
```

The same speed, within measuring noise. Looking at the machine code ([lesson 5](../05-real-machine-code/)) shows why:
- **The borrow checker costs nothing at run time.** Ownership and borrowing are checked while compiling, and leave no trace in the machine code.
- **Most bounds checks disappear.** In `for i in 0..numbers.len() { numbers[i] }`, the compiler proves every index is valid, and the compiled function contains **no** bounds check at all. Change the loop to `for i in 0..n` with an `n` it can't verify, and exactly **one** check appears, precisely where an out-of-bounds read could happen.
- **Safe abstractions cost no space.** `Option<&T>` is the size of a plain pointer ([lesson 3](../03-pointers-are-addresses/)), and a `Box` is one pointer ([pointers course](../../pointers-and-memory/02-box/)).

This is what Rust calls **zero-cost abstractions**: you don't pay for what you don't use, and what you do use couldn't be written any faster by hand.

### `unsafe`: the guard you can remove

Some jobs need what the compiler can't check: talking to hardware, calling C libraries, or writing your own memory-management code. For those, `unsafe` lets you work exactly as in C. This course used it for reading machine code ([lesson 5](../05-real-machine-code/)), crashing on purpose ([lesson 7](../07-virtual-memory/)) and system calls in assembly ([lesson 8](../08-user-space-and-kernel-space/)).

The difference from C: `unsafe` is **marked, small and searchable**. A Rust program has a few `unsafe` blocks, each with a `// SAFETY:` comment explaining why it's correct, wrapped in safe functions. When something goes wrong, you know where to look. In C, every line is "unsafe".

### What Rust doesn't solve

To stay precise:
- **Logic bugs compile fine.** A wrong formula, a wrong condition: the compiler checks memory, not meaning. That's what tests are for.
- **Memory leaks are possible** in safe Rust, for example through `Rc` cycles ([Rc and Weak](../../pointers-and-memory/03-rc-and-weak/)). They waste memory, but don't corrupt it.
- **`unsafe` code can still have every C bug.** It just has much less room to hide.

## A bit of history

| When | What happened |
|---|---|
| **1972** | Dennis Ritchie creates **C** at Bell Labs, to rewrite the Unix operating system in a language more portable than assembly |
| **1985** | Bjarne Stroustrup's **C++** adds classes and abstractions to C, with the same memory model |
| **1988** | The **Morris worm** spreads across the early internet through a buffer overflow |
| **2006** | Graydon Hoare starts **Rust** as a personal project. Mozilla begins sponsoring it in 2009, and announces it in 2010 |
| **2014** | **Heartbleed**, an out-of-bounds read in OpenSSL, affects a large part of the web's secure servers |
| **2015** | **Rust 1.0**, on 15 May: the language promises stability from now on |
| **2019–20** | Microsoft and Google's Chromium team each report that about **70%** of their serious security bugs are memory-safety bugs |
| **2022** | **Linux 6.1** accepts the first Rust code into the kernel. The US NSA recommends moving to memory-safe languages |
| **2024** | Google reports that as Android's new code moved to memory-safe languages such as Rust, memory-safety bugs fell from **76%** of Android's vulnerabilities in 2019 to **24%** in 2024 |

## The course in one table

| Lesson | In one sentence |
|---|---|
| [1. Bits and bytes](../01-bits-and-bytes/) | everything is switches, grouped in eights |
| [2. Memory and addresses](../02-memory-and-addresses/) | memory is numbered mailboxes |
| [3. Pointers are addresses](../03-pointers-are-addresses/) | a pointer is a mailbox number written down |
| [4. A tiny CPU](../04-a-tiny-cpu/) | fetch, decode, execute, billions of times a second |
| [5. Real machine code](../05-real-machine-code/) | a function is bytes in memory |
| [6. The cache](../06-the-cache/) | the order of reading memory can make code ten times faster |
| [7. Virtual memory](../07-virtual-memory/) | every program gets its own pretend memory, guarded by the processor |
| [8. User space and kernel space](../08-user-space-and-kernel-space/) | programs ask the kernel for everything outside themselves |
| 9. Why Rust | the control and speed of C, with the compiler checking memory |

## Run it

```bash
cargo run --release      # the timings: only meaningful in a release build
cargo test
```

Previous: [Lesson 8: User space and kernel space](../08-user-space-and-kernel-space/) · Back to the [course overview](../)
