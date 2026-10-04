<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 9: Why Rust

## The idea in one sentence

Rust lets you work close to the machine while **safe code prevents invalid memory access**, using a mixture of compile-time rules and run-time checks.

## Imagine… a workshop with a plan inspector

Ferris wants a fast workshop. Before a job begins, an inspector checks the plan: “Who owns this tool? Will it still exist when Ada uses it? Could two workers change it in conflicting ways?”

That inspector is like Rust's **compiler**, and especially its **borrow checker**: the part that checks who owns each value and who is borrowing it ([lesson 3](../03-pointers-are-addresses/)). It can reject a bad plan before the program ever runs.

Some facts aren't known until the job runs. How long is the customer's list? Is item 10 present? Those need **run-time checks**, like a guard at the machine.

C and C++ give programmers the same low-level power. Their programmers can add guards and use safer libraries, but with raw pointers, making sure every access is valid is left to the programmer, every single time. Rust builds most of those checks into the language itself.

The analogy doesn't promise a perfect workshop: a checked plan can still make the wrong chair!

## The memory mistakes we're avoiding

| Mistake | Example | Safe Rust's response |
|---|---|---|
| Read beyond a buffer | A two-byte message claims to contain 64,000 bytes | Checked access returns an error or panics. |
| Use a value after its owner is gone | Use a vector after `drop` | Compiler rejects the use. |
| Return a reference to a local value | Return a reference to a local `String` | Compiler rejects the lifetime. |
| Conflicting access during a borrow | Grow a vector while iterating over its borrowed elements | Compiler rejects the conflict. |
| A **data race** | Different threads make conflicting accesses to the same memory, at least one writes, and the accesses aren't properly synchronised | Safe Rust's rules for sharing between threads reject it. |

Some words in the table: a value's **owner** is responsible for its lifetime. Ownership can move to another variable or into a container. When a value is **dropped**, its cleanup runs; for a `Vec`, that drops its elements and releases its allocation. `drop(names)` does this early by taking ownership of the vector. But `drop(42)` only drops a copied integer; not every value has heap memory to free! [Rust's `drop` documentation](https://doc.rust-lang.org/std/mem/fn.drop.html) explains the distinction.

A **thread** ([lesson 7](../07-virtual-memory/)) is one execution path in a process. Threads share the process's memory and can take turns on one core or run in parallel on several.

A **data race** is one specific kind of memory error. It isn't every timing problem: safe Rust programs can still have a **race condition** in the wider sense, such as two tasks both checking "is a seat free?" and then both booking it.

Invalid C/C++ accesses cause **undefined behaviour** ([lesson 3](../03-pointers-are-addresses/)): there's no guaranteed result. A crash, leaked secrets or an apparently normal run may occur. Hardware page protection also doesn't catch every such bug, as [lesson 7](../07-virtual-memory/) explains.

## Watch the compiler reject a bad plan

These snippets intentionally fail to compile. The errors below were checked with Rust **1.99.0**. They aren't code to add to a working `main` unchanged.

### A vector after its owner is dropped

```rust
fn main() {
    let names = vec![String::from("Ada")];
    drop(names);
    println!("{}", names.len());
}
```

```text
error[E0382]: borrow of moved value: `names`
```

`drop(names)` takes ownership of the vector and frees it. "Moved" means ownership went somewhere else; `names` no longer owns anything, so the compiler refuses to let us use it. A similar C program can compile, but accessing an allocation after freeing it is **undefined behaviour**. It doesn't promise any particular read or result.

### Grow a list while borrowing it

```rust
fn main() {
    let mut numbers = vec![1, 2, 3];
    for n in &numbers {
        numbers.push(*n);
    }
}
```

```text
error[E0502]: cannot borrow `numbers` as mutable because it is also borrowed as immutable
```

When a vector runs out of capacity, `push` may move its elements to a new allocation, leaving references into the old allocation dangling. An allocator may sometimes grow the allocation in place, but Rust can't let safety depend on that. The loop borrows the vector's elements while `push` needs a mutable borrow of the vector. Those borrows conflict, so the compiler rejects the code—even if you believe there is spare capacity. See [the `Vec` guarantees](https://doc.rust-lang.org/std/vec/struct.Vec.html#guarantees).

### Return a reference to a disappearing value

```rust
fn get() -> &'static str {
    let text = String::from("hi");
    &text
}

fn main() {}
```

```text
error[E0515]: cannot return reference to local variable `text`
```

`text` is a local variable: it's freed when `get` returns. A reference to it would point at freed memory, a **dangling** reference. The return type says `&'static str`: a reference valid for the whole run of the program (a **lifetime** of `'static`), but writing that doesn't make `text` live forever!

[The Rust Book's borrowing chapter](https://doc.rust-lang.org/book/ch04-02-references-and-borrowing.html) explains these rules.

## Heartbleed: when a length became a leak

**OpenSSL** is a C library that protects a large part of the internet's traffic (the "s" in "https"). It has a **heartbeat** feature: one computer sends a few bytes, says how many it sent, and the other sends the same bytes back, to show the connection is still alive.

In **2014**, the **Heartbleed** bug was found: OpenSSL trusted the length it was *told* instead of checking the bytes it actually *received*. An attacker could send 2 bytes, claim to have sent 64,000, and get back their 2 bytes followed by whatever happened to lie next in the server's memory. OpenSSL's advisory reported that **up to about 64 KiB of memory** could leak per request, which could contain passwords and secret keys. It was a missing **bounds check** ([lesson 3](../03-pointers-are-addresses/)). [The original advisory](https://openssl-library.org/news/secadv/20140407.txt)

Our Rust example models the length check, not the entire network protocol:

```rust
fn heartbeat(payload: &[u8], claimed_length: usize) -> Result<&[u8], String> {
    payload.get(..claimed_length).ok_or_else(|| format!(
        "refused: claimed {claimed_length} bytes, but only {} were sent",
        payload.len()
    ))
}
```

```text
1. A heartbeat, in Rust
    honest client (2 bytes, claims 2):    Ok("hi")
    attacker (2 bytes, claims 64000):     Err("refused: claimed 64000 bytes, but only 2 were sent")
```

`payload.get(..claimed_length)` asks for the bytes from the start up to `claimed_length`. If that range doesn't fit inside `payload`, it returns `None` instead of reading further ([lesson 3](../03-pointers-are-addresses/)). [`ok_or_else`](https://doc.rust-lang.org/std/option/enum.Option.html#method.ok_or_else) turns `None` into `Err(message)` and `Some(bytes)` into `Ok(bytes)`. The `||` begins a small **closure**, a function passed to this method. It builds the error message only when the range is missing, so valid requests don't allocate an unused message. The bounds check still applies to every request.

Writing `&payload[..claimed_length]` instead would **panic** for an invalid range; the bounds check still prevents reading beyond the payload. An unhandled panic can interrupt service, and panic messages can themselves contain information. Returning an error lets the caller handle this bad request deliberately.

**Try it:** what should a zero-length request return, even if the payload is empty?

<details>
<summary>Show the answer</summary>

`Ok` with an **empty slice**. The range `..0` fits. Memory safety doesn't mean every unusual request is invalid.

</details>

## Do the checks slow everything down?

This program sums **50,000,000 `u64` values**, three ways. The data occupies 400,000,000 bytes, about 381 MiB.

- **iterator**: `numbers.iter().sum()`. The iterator knows the slice's end; the program doesn't request individual elements by index.
- **indexing**: a loop reading `numbers[i]`, which Rust checks, unless the compiler can prove that `i` is always smaller than the length.
- **`get_unchecked`**: the same loop, but each read uses `unsafe` to skip the check, like C.

Each way runs five times, and the program shows the fastest time. One release run on the M2 Mac printed:

```text
3. Adding up 50 million numbers, three ways (best of 5 runs)
    iterator (safe, idiomatic):        6.44 ms
    indexing (safe, numbers[i]):       6.37 ms
    get_unchecked (unsafe, like C):    6.32 ms
    the same answer every time: 1249999975000000
```

The three reported times are close **in this run**; the minimum of five runs doesn't tell us the full variation. A loop such as `for i in 0..numbers.len()` can let the optimiser prove each index valid and remove checks. That is a possible explanation, not something timings alone establish. This experiment shows little measured benefit from manual unchecked access here; it doesn't compare with a compiled C program or prove that all safe code is equally fast. Inspect the generated assembly ([lesson 5](../05-real-machine-code/)) to find which checks remain.

The ownership and borrowing rules cost nothing while the program runs: they're checked by the compiler, before it runs. (A few special types, such as `RefCell`, enforce additional borrowing rules at run time, for example when several owners share access and the borrowing pattern depends on run-time decisions.)

An **abstraction** is a convenient high-level tool, such as an iterator, that hides the details underneath. **Zero-cost abstractions** is one of Rust's design goals: such tools should compile to machine code as good as what you'd write by hand. It's a goal, not a promise that every abstraction is free.

**Fun fact:** in this sum, adding `unsafe` didn't provide a useful measured speedup. Clear safe code can give the optimiser all the information it needs.

## What does unsafe change?

`unsafe` allows a few extra operations whose safety the compiler can't check, such as following a raw pointer ([lesson 3](../03-pointers-are-addresses/)), running assembly, or calling an `unsafe` foreign function, such as a C API with memory requirements the caller must uphold.

It **doesn't turn off the borrow checker** or any other rule. It means: "for these few operations, I, the programmer, have checked that they're valid." A good `// SAFETY:` comment says what had to be true, and why it is. [The Rust Book's unsafe chapter](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html) describes the boundary.

This course uses unsafe operations for assembly and reading known instruction bytes. Lesson 7's deliberate invalid accesses are different: they **violate** the requirements to demonstrate bugs; their outcomes aren't guaranteed.

Safe code often calls libraries that use `unsafe` inside, including the standard library. If that `unsafe` code has a mistake, the safe code calling it can break too. So Rust's guarantees depend on the compiler and the `unsafe` code underneath being correct. The benefit is that the dangerous parts are few, marked, and can be checked very carefully.

## What still needs your attention?

| Safe Rust can still… | Example |
|---|---|
| Calculate a wrong answer | Add when the task required subtraction. |
| Panic or fail | An invalid index or an I/O error. |
| Leak memory | Two values that each keep the other alive, through `Rc` (a shared-ownership pointer), are never freed. |
| Deadlock | Two threads each hold a **lock** (permission to use something alone) and wait forever for the other's. |
| Mishandle security rules | Reveal data the caller wasn't authorised to see. |

Use tests, error handling, reviews and good design alongside the language's protections.

## Words to remember

| Word | Meaning |
|---|---|
| **ownership** | responsibility for a value's lifetime; moving transfers it, dropping performs cleanup |
| **borrow checker** | the part of the compiler that checks references against the ownership and borrowing rules |
| **use after free** | using memory after it has been freed: rejected by safe Rust |
| **data race** | unsynchronised conflicting memory accesses from different threads, at least one writing |
| **bounds check** | testing that an index or range is inside the data before reading |
| **`unsafe`** | a marked block where the programmer, not the compiler, checks a few extra operations |
| **memory safety** | never reading or writing memory that a value doesn't validly own or borrow |

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1969–73** | C developed alongside Unix at Bell Labs, with much of its formative work in **1972**. It made machine-oriented programming more portable than assembly. [Dennis Ritchie's account](https://www.nokia.com/bell-labs/about/dennis-m-ritchie/chist.html) |
| **2006** | Graydon Hoare starts **Rust** as a personal project. He later recalls his frustration with a broken elevator that year—a memorable example of why reliable software matters. His own account also stresses the many people who later built the language. [Hoare's reflection](https://rustfoundation.org/media/10-years-of-stable-rust-an-infrastructure-story/) |
| **2009** | **Mozilla** begins funding Hoare's work on Rust. A hobby project becomes a supported language-development effort. [Hoare's early-project archive](https://github.com/graydon/rust-prehistory) |
| **2014** | **Heartbleed** made the consequences of a missing buffer-length check widely visible. [OpenSSL's advisory](https://openssl-library.org/news/secadv/20140407.txt) |
| **2015** | **Rust 1.0** was released on **15 May**, starting its stability commitment. [The release announcement](https://blog.rust-lang.org/2015/05/15/Rust-1.0/) |
| **2024** | Google reported that memory-safety bugs' share of Android vulnerabilities fell from **76% in 2019 to 24% in 2024**, during a move toward memory-safe new code, including Rust. These are historical Android figures, not percentages for all software or a measurement of Rust alone. [Google's report](https://security.googleblog.com/2024/09/eliminating-memory-safety-vulnerabilities-Android.html) |

These protections help experienced programmers as much as learners: nobody has to remember every dangerous detail every time.

## Run it

From this lesson's directory:

```bash
cargo run --release
cargo test
```

The program shows the heartbeat results and compares the three sums. The programs that the compiler rejects can't be part of it, of course: they're shown above, and in comments in the source. To see the errors yourself, paste one into a new project and run `cargo build`.

Next we'll explore a different kind of information: **qubits**, used by quantum computers. Rust can help simulate and control quantum computers, but ordinary Rust values are still the ordinary bits of lesson 1.

Previous: [Lesson 8: User space and kernel space](../08-user-space-and-kernel-space/) · Next: [Lesson 10: Quantum computing](../10-quantum-computing/)
