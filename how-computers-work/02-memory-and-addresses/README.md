<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Memory and addresses

## The idea in one sentence

In the memory model we're exploring, an **address** identifies a byte, and a value such as a `u32` occupies several neighbouring bytes.

## Imagine…

Ferris opens a post office with a long street of numbered mailboxes. Each mailbox holds **one byte**.

A `u8` needs one mailbox. A `u32` needs four in a row. To find the `u32`, Ada needs its **starting address** and its **type**, which tells her how many bytes to read and how to interpret them.

The addresses printed by this program belong to its **virtual address space**, the memory map it sees. They aren't physical locations on a RAM chip. [Lesson 7's magic hotel](../07-virtual-memory/) explains how that map works. Some addresses have no usable mailbox at all!

A useful counting detail: **16 GiB** means 16 × 1,073,741,824 = **17,179,869,184 bytes**. **16 GB** means 16,000,000,000 bytes. The units look similar but count differently.

## Looking at a real address

In Rust, `&a` makes a **reference** to `a`: a value that holds `a`'s address, so other code can find `a` without copying it. ([Lesson 3](../03-pointers-are-addresses/) is all about references and pointers.) When printing, `{:p}` shows the address inside a reference, in hexadecimal ([lesson 1](../01-bits-and-bytes/) explains hex).

The program takes references to three local values and prints their addresses:

```text
1. The addresses of three local values (the number of each one's first byte)
    a (u8,  1 byte)  is at 0x16ce7a013
    b (u32, 4 bytes) is at 0x16ce7a014
    c (u64, 8 bytes) is at 0x16ce7a018
```

These are example addresses. Yours may differ, and may change between runs; a change isn't guaranteed on every run.

In this run, `b` starts right after `a`, at the next address, and `c` starts four bytes after `b`, exactly where `b`'s four bytes end. The compiler doesn't promise where local variables go, so another compiler version may arrange them differently. Here, we print the locations of three addressable local values. Taking a reference doesn't always force a physical memory access: the compiler may remove a reference or keep a value in a processor *register* ([lesson 4](../04-a-tiny-cpu/)) when that preserves the program's observable behaviour. It may also remove a value entirely. And a value with no bytes at all, such as `()`, needs no mailbox.

## An array puts its elements together

For `[10u32, 20, 30, 40]`, each element takes four bytes. The **offset** of an element is how many bytes after the start of the array it begins:

```text
index       0         1         2         3
value      10        20        30        40
offset      0         4         8        12 bytes
           [----]    [----]    [----]    [----]
```

The address of element `i` is:

```text
start + i × size_of::<element_type>()
```

If the array starts at byte address 1000, element 2 starts at **1000 + 2 × 4 = 1008**. Rust guarantees this element spacing for arrays. A `Vec<u32>` also keeps its elements in one contiguous allocation.

That formula locates an element; it doesn't prove the index is valid. This array has indices **0, 1, 2, 3**, not 4. [Lesson 3](../03-pointers-are-addresses/) adds the safety check.

## Which end comes first? Endianness

The number `0x12345678u32` has four bytes. The computer must choose their order:

In the decimal number 1,234, the `4` is the **least significant** digit (it's only worth ones) and the `1` is the **most significant** (it's worth thousands). Bytes in a number work the same way: in `0x12345678`, the byte `78` is worth the least and `12` the most.

| Order, from lowest address upward | Name |
|---|---|
| `78 56 34 12` | **little-endian**: least significant byte first (the "little end") |
| `12 34 56 78` | **big-endian**: most significant byte first, the order we write numbers in |

Many desktop and phone computers use little-endian order. Big-endian machines and formats exist too; internet protocols often specify big-endian fields.

```rust
let n = 0x1234_5678u32;
assert_eq!(n.to_le_bytes(), [0x78, 0x56, 0x34, 0x12]);
assert_eq!(n.to_be_bytes(), [0x12, 0x34, 0x56, 0x78]);
```

`to_ne_bytes()`, used in the example, gives the **native** order of the target computer.

Byte order matters whenever you interpret a sequence of bytes as a multi-byte value: in files, networks, shared memory or interfaces to other code. Say which order you want instead of assuming every computer agrees.

## Alignment: where may a value begin?

Mailboxes may have placement rules. A value with alignment **4** must begin at an address divisible by 4: 1000 is suitable, 1001 isn't.

**Why?** Processors support memory accesses of different sizes. Depending on the processor and instruction, an unaligned access may be slower, need extra work when it crosses a boundary, or be unsupported. Alignment rules tell the compiler where an ordinary value may safely begin. They don't mean the CPU always reads eight bytes, or that every aligned value fits in one read. A large, aligned struct can still span many memory blocks!

The rules depend on the **target**: the kind of computer a program is compiled for, meaning its processor and operating system. The program asks Rust for the actual requirements with `align_of`. On the tested Mac:

| Type | Size in bytes | Alignment in bytes |
|---|---:|---:|
| `u8` | 1 | 1 |
| `u16` | 2 | 2 |
| `u32` | 4 | 4 |
| `u64` | 8 | 8 |

**Size and alignment are different questions.** Integer sizes are fixed for these types; alignment can depend on the target. The table's four-byte alignment for `u32` describes this target. [Rust's `align_of` documentation](https://doc.rust-lang.org/std/mem/fn.align_of.html) explains the target dependence.

The program also checks one real address using `align_of::<u32>()`. On this target, `b`'s address **modulo 4** (the remainder after dividing by 4) is `0`, so `b` starts at a multiple of 4. Another target may use a different requirement. Rust guarantees that every ordinary reference points to a properly aligned value.

## Why are there gaps in a struct?

Our structs contain these fields:

```rust
small: u8,
big: u32,
other: u8,
```

On the tested target, `big` needs alignment 4. With `#[repr(C)]`, fields stay in declaration order, with padding:

```text
offset:     0   1   2   3   4   5   6   7   8   9  10  11
C order:   [s] [ .   .   . ][    big     ] [o] [ .   .   . ]  12 bytes
Rust here: [    big     ] [s] [o] [ .   . ]                   8 bytes
```

Dots are **padding**, not extra fields. The final padding makes the next struct in an array begin at a correctly aligned address.

Rust's default representation may reorder fields. This compiler chose eight bytes; **Rust doesn't promise this field order, this size, or that its choice is always smaller than the C layout**. `repr(C)` follows the target's C layout rules, so it isn't a universal file format either. See [Rust's representation rules](https://doc.rust-lang.org/reference/type-layout.html#representations).

The layout test calculates the C field offsets and total size from **this target's alignment rules**. For default Rust layout, it checks that fields are aligned, don't overlap and fit inside the struct. It doesn't require Rust's layout to be smaller than C's. The 12-byte and eight-byte drawings above remain observations from the demonstrated build.

Use `repr(C)` when an interface requires C-compatible layout. For a file or network packet, encode the fields and byte order explicitly.

## Words to remember

| Word | Meaning |
|---|---|
| **address** | the number of a byte in memory: its mailbox number |
| **reference** (`&x`) | a value holding another value's address, so it can be found without copying it |
| **offset** | how many bytes after a starting point something begins |
| **little-endian / big-endian** | storing a number's least / most significant byte at the lowest address |
| **alignment** | the rule that a value must start at a multiple of some number, such as 4 |
| **padding** | unused bytes inserted to keep fields aligned |
| **target** | the kind of computer a program is compiled for: its processor and operating system |

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1966 / 1968** | IBM's Robert Dennard devised a compact **DRAM** memory cell in 1966; the patent followed in 1968. It stores a bit using a capacitor's charge and a transistor. The charge needs periodic refreshing. [IBM's DRAM history](https://www.ibm.com/history/dram) |
| **1970** | Intel's **1103** made commercial DRAM an important alternative to magnetic-core memory. It held **1,024 bits**, just **128 bytes**. Its cell design differed from Dennard's single-transistor design. [IBM's account](https://www.ibm.com/history/dram) |
| **1980** | Danny Cohen used **big-endian** and **little-endian** in an essay about choosing byte order. The names came from the argument over which end of an egg to crack in *Gulliver's Travels*. [Cohen's original essay](https://history.rfc-editor.org/ien/ien137.txt) |

**Fun fact:** a serious computing term came from a fictional breakfast argument. Computers don't care which end you pick, but computers exchanging data must agree!

## Run it

From this lesson's directory:

```bash
cargo run
cargo test
```

**Try it:** if an array of `u64` values starts at address 2000, where does element 3 start?

<details>
<summary>Show the answer</summary>

A `u64` takes eight bytes. Element 3 starts at **2000 + 3 × 8 = 2024**, provided the array has at least four elements.

</details>

Previous: [Lesson 1: Bits and bytes](../01-bits-and-bytes/) · Next: [Lesson 3: Pointers are addresses](../03-pointers-are-addresses/)
