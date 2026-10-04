<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 8: User space and kernel space

## The idea in one sentence

Your program runs in a restricted **user mode** and can't touch the disk, the network or the screen by itself; for all of that it must ask the **kernel**, the core of the operating system, through a **system call**, and every such request has a cost.

## Imagine…

Back in the apartment building from [lesson 7](../07-virtual-memory/): in your own flat you can do whatever you like. You can cook, tidy and rearrange the furniture. But you can't open the front door of the building, use the post room or switch the lights in the corridor. Only the **building manager** has those keys.

So you go to the manager's office, ring the bell and say exactly what you want: "please post this letter". The manager checks that you're allowed, does it, and tells you how it went. Going to the office takes a little while, so you don't go once for every single word of your letter: you write the whole letter first, and then go once.

Your flat is **user space**. The manager's office is **kernel space**. Ringing the bell is a **system call**. Writing the whole letter first is **buffering**.

## Precisely

### Two modes

The processor itself has (at least) two modes:

| | User mode | Kernel mode |
|---|---|---|
| who runs in it | every normal program | the operating system's kernel |
| memory | only the program's own pages ([lesson 7](../07-virtual-memory/)) | everything |
| hardware (disk, network, screen, timers) | no direct access | full access |
| special instructions (changing page tables, stopping the processor…) | refused by the processor | allowed |

A user-mode program that tries anything forbidden is stopped by the processor, exactly as lesson 7's child processes were. The only door into the kernel is a special instruction: **`syscall`** on x86-64, **`svc`** on ARM.

### A system call, by hand

The `hello-syscall` program next to this lesson prints its message **without** the standard library: it makes the system calls itself, in inline assembly. On Linux x86-64:

```rust
asm!("syscall",
     inlateout("rax") 1isize => result,   // which service: 1 = write
     in("rdi") 1,                         // where: file 1, standard output
     in("rsi") bytes.as_ptr(),            // what: the address of the bytes
     in("rdx") bytes.len(),               // how many
     out("rcx") _, out("r11") _);         // `syscall` overwrites rcx and r11
```

And this is the complete compiled program for Linux, just two system calls:

```text
mov  eax, 0x1      ; system call 1: write
mov  edx, 0x29     ; 41 bytes: the message's length
mov  edi, 0x1      ; to file 1: standard output
syscall            ; → into the kernel, and back with the answer in rax
xor  edi, edi
cmp  rax, 0x29     ; were all 41 bytes written?
setne dil          ; exit code: 0 if yes, 1 if not
mov  eax, 0x3c     ; system call 60: exit
syscall
```

The program puts a **number** in an agreed register to say *which* service it wants, the arguments in others, and executes `syscall`. The processor switches into kernel mode and jumps to the kernel's entry point, which the kernel set up in advance; a program can't choose where to land. The kernel checks the request, does the work, puts the answer in `rax` and switches back.

Each operating system has its own numbers and rules:

| | write | exit | instruction | stable for programs to use directly? |
|---|---|---|---|---|
| Linux x86-64 | 1 | 60 | `syscall` | **yes**: the numbers are a documented, stable interface |
| Linux AArch64 | 64 | 93 | `svc #0` (number in `x8`) | yes |
| macOS AArch64 | 4 | 1 | `svc #0x80` (number in `x16`) | **no**: they work, but Apple only supports calls through its system library, `libSystem` |
| macOS x86-64 | 0x2000004 | 0x2000001 | `syscall` | no (as above) |
| Windows | — | — | — | **no**: the numbers change between Windows versions, so `hello-syscall` calls the official API (`WriteFile` in `kernel32`) instead |

That's why normal programs never do this by hand. Rust's standard library calls the system's C library (on macOS and Linux), which makes the system calls. You get the same result, and it keeps working when the operating system is updated.

When writing inline assembly, you must tell the compiler about **every** register the assembly changes. The first version of this example, the repository's old `asm/hello_asm`, forgot that `syscall` overwrites `rcx` and `r11`, and its Windows code called functions without declaring the registers they may change. Such bugs can corrupt values silently, so this version declares them all.

### What a system call costs: measured

Writing one million bytes, one byte at a time, measured with `cargo run --release` on an Apple M2 Mac:

```text
straight to the kernel:     1.86s   1000000 system calls
through a BufWriter:       8.31ms   about 123 system calls
into a Vec in memory:    412.04µs   no system calls
one `write` system call to a file took about 1847 ns on this machine
```

| | Time | Why |
|---|---|---|
| a million system calls | 1.86 s | every byte: into the kernel, through the file system, and back |
| `BufWriter`: 123 system calls | 8 ms, **over 200× faster** | bytes are collected in 8 KB of the program's own memory, then handed over 8 KB at a time |
| no system calls | 0.4 ms | only user-space work |

A `write` to a file took about 1.8 µs, most of it spent in the kernel and the file system, not in the mode switch itself. In that time, the processor could have done thousands of steps of ordinary work. That's why the [data processing](../../data-processing/02-streaming-big-files/) course wraps every file in a `BufReader` or `BufWriter`: they turn many small requests into a few big ones.

### Everyday system calls

Almost everything a program does with the outside world is a system call: opening, reading and writing files; sockets and network traffic; starting processes and threads; getting memory pages for the heap ([lesson 7](../07-virtual-memory/)'s page faults are the kernel at work too); sleeping; and asking the process id (`getpid`). On Linux, `strace` shows every system call a program makes; on macOS, `dtruss` does, with some restrictions.

## A bit of history

| When | What happened |
|---|---|
| **1950s** | Early computers have no protection at all: every program can do everything, including breaking the machine for everyone |
| **1964** | IBM's **System/360** separates a **supervisor state** for the operating system from a **problem state** for programs: user and kernel mode |
| **1960s** | **Multics** builds a whole system around protection **rings**: several levels of privilege, with the kernel in the innermost ring |
| **1970s** | **Unix** makes the system call interface small and simple: `open`, `read`, `write`, `close`, `fork`. Linux, macOS and Android still follow it today |
| **1982** | Intel's **80286** brings protection rings to personal computers; the 80386 (1985) adds paging. Most systems use only ring 0 (kernel) and ring 3 (user) |
| **2018** | The **Meltdown** attack shows that user programs could read kernel memory through the processor's speculative execution. The fix, keeping the kernel's pages hidden while user code runs, made system calls noticeably more expensive on affected processors |

## Run it

```bash
cargo run --release           # the measurements
cargo test

cargo run -p hello-syscall    # "Hello, world!" by system calls in assembly
```

Previous: [Lesson 7: Virtual memory](../07-virtual-memory/) · Next: [Lesson 9: Why Rust](../09-why-rust/)
