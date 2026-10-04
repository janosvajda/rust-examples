<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# How computers work: from bits to the kernel

<p align="center">
  <a href="rust-how-computers-work.png">
    <img src="rust-how-computers-work.png" alt="Ferris the crab reads a book called How computers work, in front of a board showing the lessons: bits and bytes, memory, pointers, the CPU, machine code, the cache, virtual memory, and user space and kernel space" width="350px">
  </a>
</p>

What really happens inside a computer when a program runs? This course starts with single bits, and works up through memory, pointers, the processor, machine code, caches and virtual memory, to the operating system's kernel, and ends with why Rust is a good tool for writing code this close to the machine.

## Made for everyone

Every lesson explains each idea twice:
- **"Imagine…"**: a picture from everyday life (mailboxes, a cook with recipe cards, an apartment building), simple enough for children;
- **"Precisely"**: the real facts, with Rust code you can run.

The picture is never allowed to contradict the facts. It's the same fact, said simply. Every lesson also has **"A bit of history"**: how this part of computing came to be, and why.

## The lessons

| # | Lesson | The picture | Precisely |
|---|---|---|---|
| 1 | [Bits and bytes](01-bits-and-bytes/) | rows of light switches | binary, bytes, negative numbers, UTF-8 text, colours, why `0.1 + 0.2` isn't `0.3` |
| 2 | [Memory and addresses](02-memory-and-addresses/) | a street of numbered mailboxes | addresses, arrays, endianness, alignment, struct layout |
| 3 | [Pointers are addresses](03-pointers-are-addresses/) | a slip of paper with a mailbox number | references, indexing as arithmetic, what goes wrong with pointers and how Rust prevents it |
| 4 | [A tiny CPU](04-a-tiny-cpu/) | a cook following recipe cards | **a working 8-bit CPU in Rust**: registers, machine code, fetch–decode–execute, loops from jumps |
| 5 | [Real machine code](05-real-machine-code/) | each kitchen's own card language | x86-64 and ARM side by side, a function written in assembly, its bytes read from memory |
| 6 | [The cache](06-the-cache/) | a shelf by the stove and a warehouse down the road | **measured**: 2 ns vs 105 ns per read; reading order making code 10× faster or slower |
| 7 | [Virtual memory](07-virtual-memory/) | every flat numbers its rooms 1, 2, 3 | pages, address randomisation, the operating system stopping bad accesses, memory given on first use |
| 8 | [User space and kernel space](08-user-space-and-kernel-space/) | asking the building manager | system calls **by hand in assembly** on Linux, macOS and Windows; **measured**: what a system call costs |
| 9 | [Why Rust](09-why-rust/) | a workshop where every saw has a guard | C's power and its memory bugs, with real history; Rust's checks, shown with real compiler errors, at the same measured speed |

## About the numbers

Every number in these lessons comes from actually running the code, on an Apple M2 Mac unless a lesson says otherwise. On your computer, sizes and timings will differ, sometimes a lot, but the patterns won't. Run the lessons yourself and compare: that's the best way to learn what your own machine does.

## Related courses

- [Pointers and memory](../pointers-and-memory/): Rust's smart pointers (`Box`, `Rc`, `Arc`) and raw pointers, in depth.
- [no_std and bare-metal Rust](../no-std-and-bare-metal/): Rust with no operating system at all, down to hardware registers.
- [Ownership and borrowing](../ownership-and-borrowing/): the rules behind the safety in lesson 9.

## Run a lesson

```bash
cd how-computers-work/04-a-tiny-cpu
cargo run
cargo test
```

Lessons 6, 8 and 9 measure speed: run them with `cargo run --release`.
