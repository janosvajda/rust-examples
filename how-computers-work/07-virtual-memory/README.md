<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 7: Virtual memory

## The idea in one sentence

Every running program gets its own private, **pretend** memory, starting from address 0. The processor translates every address it uses to a real place in memory, piece by piece, using a map that only the operating system may change.

## Imagine…

Imagine a big apartment building. In every flat, the rooms are numbered the same way: room 1, room 2, room 3. When you say "room 2" in your flat, you mean *your* kitchen, not your neighbour's. Nobody in your flat can even name a room in someone else's flat.

Only the **building manager** has the full plan: which "room 2" is which real room in the building. The manager also decides which rooms each flat gets, and which doors are locked. If you try to open a locked door, the manager stops you.

The building is the computer's real memory. Each flat is a running program. The room numbers are **virtual addresses**. The manager is the **operating system**, together with a part of the processor called the **MMU** (memory management unit), which looks up the plan for every single memory access.

## Precisely

### Pages and the page table

Memory is managed in blocks called **pages**: 16 KB on Apple Silicon Macs like the one these numbers come from, and 4 KB on most PCs. (Check yours with `pagesize` on macOS, or `getconf PAGESIZE` on Linux.)

For each program, the operating system keeps a **page table**: for each page of the program's virtual addresses, where it really is, and what the program may do with it:

```text
virtual page            real memory        allowed
0x0000_0000 (page 0)    —  not mapped      nothing
0x1026_f8000…           page 31,207        read + execute  (the code)
0x1027_34000…           page 9,402         read            (constants)
0x16d7_04000…           page 52,118        read + write    (the stack)
```

*(An illustration: a real page table has many thousands of entries, and the real page numbers are hidden from programs.)*

For every memory access, the MMU looks up the page table, computes the real address and checks the permission, in hardware, so fast that programs don't notice. A small cache of recent lookups, the **TLB**, makes this nearly free.

This gives three things.

### 1. Every program has its own addresses, randomised

```text
run 1: code 0x104b1d0cc  static 0x104b60728  stack 0x16b2e5fbf  heap 0x1052f5ba0
run 2: code 0x1029c90cc  static 0x102a0c728  stack 0x16d439fbf  heap 0x102fb1ba0
run 3: code 0x10076d0cc  static 0x1007b0728  stack 0x16f695fbf  heap 0x100fe9ba0
```

The same program, started three times, has its code, data, stack and heap at different addresses every time. This is **address space layout randomisation (ASLR)**. An attacker who finds a bug can't know in advance where anything is.

Look at the **last digits**: `0cc`, `728`, `fbf`, `ba0` are the same in every run. Randomisation moves whole **pages**, so the position *inside* a page never changes. With 16 KB pages, the last 14 bits always stay the same.

Because each program has its own address space, the same virtual address in two programs means two different places. One program can't read or overwrite another's memory, by accident or on purpose.

### 2. Touching memory you don't own: the operating system stops you

```text
reading address 16:            stopped by the operating system: signal 11, SIGSEGV (segmentation fault)
writing to a string constant:  stopped by the operating system: signal 10, SIGBUS (bus error)
```

Both are done by a **child process**, a copy of the program, so the lesson itself keeps running:
- **Address 16** is in page 0, which is never mapped, so that a null pointer can never accidentally work. The MMU finds no entry, and the operating system ends the program with a **segmentation fault**.
- **String constants** are in pages marked *read-only*. Writing there breaks the page's permission, and the program is stopped again. macOS reports this one as `SIGBUS`, and Linux as `SIGSEGV`.

This is the safety net under every program: a bug can crash its own program, but not the computer or other programs. It's also why C code with pointer bugs "segfaults". In safe Rust, these accesses can't be written at all: the child process needs `unsafe` and raw pointers to make them ([lesson 3](../03-pointers-are-addresses/)).

Remember the self-modifying program in [lesson 4](../04-a-tiny-cpu/)? Real systems guard against that too. Pages holding data are marked **not executable**, so the processor refuses to run them as code. Some systems, including macOS on Apple Silicon and OpenBSD, go further: a page may be writable or executable, but never both at once.

### 3. Memory is handed out on first use

```text
asking for 1 GB of zeroed memory: 7.83µs
then touching every page once:    156.59ms
```

Asking for 1 GB takes a few microseconds, because nothing really happens yet: the operating system only notes that those virtual pages may be used. Only when the program **first touches** a page does the processor stop, a **page fault**, and the operating system finds a real, zeroed page and adds it to the page table. That's about 65,000 page faults for 1 GB with 16 KB pages, taking a few microseconds each.

So a program can reserve more memory than it will ever use, at almost no cost. And when real memory runs out, the operating system can move rarely used pages to disk, to **swap**, and bring them back on the next touch. The program never notices, except that it gets slower.

## A bit of history

| When | What happened |
|---|---|
| **1962** | The **Atlas** computer at the University of Manchester introduces virtual memory: pages, and automatic transfer between fast memory and a slower drum store |
| **1960s–70s** | Mainframe and minicomputer systems adopt virtual memory and per-process protection |
| **1985** | Intel's **80386** brings paging to personal computers. Operating systems like Windows NT and Linux build on it |
| **2001** | The **PaX** project for Linux introduces ASLR. Windows, macOS and mainstream Linux adopt it in the following years |
| **2003** | AMD's 64-bit processors add the **NX bit** ("no execute"), so the processor itself refuses to run code from data pages |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 6: The cache](../06-the-cache/) · Next: [Lesson 8: User space and kernel space](../08-user-space-and-kernel-space/)
