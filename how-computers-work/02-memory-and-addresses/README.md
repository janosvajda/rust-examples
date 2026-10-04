<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Memory and addresses

## The idea in one sentence

Memory is one very long row of bytes, each with its own number, its **address**; a value bigger than one byte simply uses several bytes next to each other.

## Imagine…

Imagine a street with billions of mailboxes in one long row. Each mailbox is numbered: 0, 1, 2, 3, and so on. Each mailbox holds exactly one byte, eight switches.

To store a small number, the computer uses one mailbox. A bigger number needs four mailboxes in a row, or eight. To find the number again, you only need the number of the **first** mailbox, and to know how many belong together.

That mailbox number is an **address**. The computer you're reading this on has billions of mailboxes: this Mac has 16 GB of memory, which is 17,179,869,184 of them.

## Precisely

### Every value has an address

`{:p}` prints a value's address: the number of its first byte, written in hexadecimal.

```text
a (u8,  1 byte)  is at 0x16d13d94b
b (u32, 4 bytes) is at 0x16d13d94c
c (u64, 8 bytes) is at 0x16d13d950
```

The addresses are different on every run and every machine; lesson 7 explains why. Also, where the compiler puts local variables isn't guaranteed: it may reorder them, or keep some only in the CPU and never in memory. What *is* guaranteed is that each value's bytes lie together.

### An array: mailboxes side by side

```text
numbers[0] = 10: address 0x16d13d9e0 = start + 0 bytes
numbers[1] = 20: address 0x16d13d9e4 = start + 4 bytes
numbers[2] = 30: address 0x16d13d9e8 = start + 8 bytes
numbers[3] = 40: address 0x16d13d9ec = start + 12 bytes
```

The elements of an array are **guaranteed** to sit directly next to each other, each `size_of::<u32>()` = 4 bytes after the previous one. That's how the computer finds `numbers[i]` instantly: it's at **start + i × 4**, with no searching. This simple rule is why arrays and `Vec`s are so fast, as [lesson 6](../06-the-cache/) shows.

### Which byte comes first? Endianness

A `u32` like `0x12345678` is four bytes: `12`, `34`, `56` and `78`. In which order do they go into the four mailboxes?

```text
0x12345678 in memory: [78, 56, 34, 12]
this computer is little-endian
```

**Little-endian** computers store the **least** significant byte (`78`, the "little end") first. Almost every computer today is little-endian: all Intel and AMD processors, and Apple's and most other ARM chips as normally used. **Big-endian** puts the most significant byte first, as we write numbers. It's still used in network protocols, which is why it's also called **network byte order**.

It only matters when bytes leave the program, for example in a file or over the network. Then you must say which order you mean: Rust's `to_le_bytes()` and `to_be_bytes()` do exactly that, on every computer.

### Alignment: numbers like round addresses

```text
u8   size 1, must start at a multiple of 1
u16  size 2, must start at a multiple of 2
u32  size 4, must start at a multiple of 4
u64  size 8, must start at a multiple of 8
```

A `u32` always starts at an address divisible by 4. It's called **alignment**. The processor fetches memory in blocks, and an aligned value never straddles the border between two blocks. Reading a misaligned value is slower on some processors, and not allowed at all on others. Rust guarantees that every value is properly aligned.

### Gaps: the same fields, two layouts

Alignment can leave gaps. Here's a struct with fields of 1, 4 and 1 bytes:

```text
in C order (repr(C)): 12 bytes: small at 0, big at 4, other at 8
Rust's choice:        8 bytes: small at 4, big at 0, other at 5
```

```text
C order:   [s][ gap ×3 ][  b  i  g  ][o][ gap ×3 ]        = 12 bytes
Rust:      [  b  i  g  ][s][o][gap×2]                     =  8 bytes
```

- With `#[repr(C)]`, fields stay in the written order, as in the C language. `big` must start at a multiple of 4, so three bytes are wasted after `small`. The whole struct must also end on a multiple of 4, so three more are wasted at the end.
- Rust's default layout is **unspecified**: the compiler may arrange fields as it likes. Today it puts `big` first and saves a third of the space.

Use `#[repr(C)]` when the exact layout matters, such as when sharing data with C code or hardware ([no_std lesson 4](../../no-std-and-bare-metal/04-memory-mapped-registers/)), and let Rust choose otherwise.

## A bit of history

| When | What happened |
|---|---|
| **1950s** | **Magnetic-core memory**: tiny rings of magnetic material threaded on wires, one ring per bit, magnetised one way or the other. It kept its contents without power, and was used for about twenty years |
| **1966** | Robert Dennard at IBM invents the **DRAM** cell: one transistor and one tiny capacitor per bit. The charge leaks away, so it must be refreshed thousands of times per second, but it's small and cheap |
| **1970** | Intel's **1103** is the first DRAM chip sold commercially, holding 1,024 bits. DRAM soon replaced core memory, and every computer's main memory still uses it today |
| **1980** | Danny Cohen's essay "On Holy Wars and a Plea for Peace" names **big-endian** and **little-endian**, after the two sides in *Gulliver's Travels* (1726) who fight over which end of a boiled egg to crack |

From 1,024 bits on one chip in 1970 to 16 GB in a laptop today, the number of bits has grown more than a hundred million times. The idea, numbered boxes of bits, hasn't changed at all.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: Bits and bytes](../01-bits-and-bytes/) · Next: [Lesson 3: Pointers are addresses](../03-pointers-are-addresses/)
