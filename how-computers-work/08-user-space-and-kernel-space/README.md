<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 8: User space and kernel space

## The idea in one sentence

Ordinary applications run in **user mode**; a **system call** asks the operating system's **kernel**, its core, to perform a protected service, such as writing to a file.

## Imagine… the hotel's front desk

Back at the [Memory Hotel](../07-virtual-memory/), Ada can read her books and rearrange furniture where her room key permits it.

But she doesn't get the keys to the delivery van or the boiler. She asks the **manager**: “Please send this letter.”

The manager checks the request and returns an answer. Ada can keep doing ordinary work in her room between requests.

If Ada visits the desk once for every letter of the alphabet, that's a lot of trips! She can instead collect the whole message and make fewer requests. That is **buffering**.

| Hotel picture | Computer idea |
|---|---|
| Guest working in a room | Application running in user mode |
| Manager's protected workspace | Kernel space |
| Request at the desk | System call |
| Collecting a whole letter first | Buffering small writes |

The analogy has a limit: the kernel isn't a separate program that Ada's request is handed to. A system call starts kernel code on behalf of the calling **thread** ([lesson 7](../07-virtual-memory/)), with the processor in kernel mode. If the request must wait, the OS may pause that thread and run others. When the call returns, the thread continues its application instructions in user mode.

## Mode means permission

The CPU has different **privilege levels**: modes in which different instructions and memory are allowed. It always knows which mode it's in. For the ordinary application–kernel boundary, we focus on two: **user mode** for applications and **kernel mode** for the kernel. Some processors also have modes for jobs such as running a hypervisor, which manages virtual machines. The memory the kernel uses is called **kernel space**; the memory of ordinary applications is **user space**.

| | User mode | Kernel mode |
|---|---|---|
| Ordinary code | Application instructions | Kernel instructions |
| Memory access | Pages permitted by the current mappings | Access to privileged OS mappings, subject to hardware rules |
| Privileged operations | Cannot freely change page tables or control devices | Can perform privileged OS operations |
| File request | Ask through an OS interface | Check access and arrange the work |

There are exceptions, always arranged by the OS: processes can share pages, and some programs that control devices (**drivers**) run in user mode with access the OS has granted them. So "user mode" doesn't mean "no hardware can ever be reached"; it means "only what the OS allows".

**Fun fact:** even a program run by an administrator normally executes in **user mode**. Account permissions and CPU privilege levels are different things.

A system call isn't the only way into the kernel. **Hardware interrupts** notify the CPU about events such as a timer tick. **Exceptions** arise while executing instructions, for example the page faults of [lesson 7](../07-virtual-memory/). The OS handles these events too. A Unix **signal**, such as SIGSEGV, is a separate way the OS notifies a process. [Linux's entry documentation](https://docs.kernel.org/arch/x86/entry_64.html) describes these entry paths on x86-64.

## A request written in assembly

The companion [hello-syscall example](hello-syscall/) prints:

```text
Hello, world! (written by a system call)
```

Including its newline, the message occupies **41 bytes**.

On Linux x86-64, its write request uses:

```rust
// Excerpt: bytes and result are declared in the surrounding function.
asm!("syscall",
     inlateout("rax") 1isize => result, // service 1: write
     in("rdi") 1,                       // file descriptor 1: standard output
     in("rsi") bytes.as_ptr(),          // address of the message
     in("rdx") bytes.len(),             // byte count
     out("rcx") _, out("r11") _);       // overwritten by syscall
```

Each line puts a value into a register before the `syscall` instruction:

- `rax` holds the **service number**: 1 means `write` on Linux x86-64. `inlateout` means the register is also an output: after the call, `rax` holds the kernel's answer, stored in `result`.
- `rdi` holds the **file descriptor**: a small number naming an open file or stream. **1** is **standard output**, where a program's normal output goes, usually the terminal window.
- `rsi` and `rdx` hold the message's address and length. The kernel reads the bytes straight from the program's memory.
- `out("rcx") _` and `out("r11") _` tell the compiler that the `syscall` instruction overwrites these two registers, so it mustn't keep anything important in them.

This is inside an `unsafe` block in the source: the compiler can't prove the assembly's safety, so we promise that the message memory is valid and that every changed register has been declared.

The CPU transfers control to a kernel entry point configured by the OS. The kernel processes the request and returns a result. At Linux's **raw syscall interface**, a successful `write` returns the **number of bytes written**; a failed one returns a negative error number. The C library's `write` wrapper instead reports failure with −1 and sets `errno`. The interface you call matters.

The companion is still a normal Rust program, using the standard library; it just doesn't use `println!` for this message. (Rust programs can also be built without the standard library, called `no_std`, but this one isn't.) So it makes more than these two system calls: Rust's start-up code, which runs before `main`, makes some too.

## Different systems, different front desks

| Target | Write service | Exit service | Interface used here |
|---|---:|---:|---|
| Linux x86-64 | 1 | 60 | `syscall` |
| Linux AArch64 | 64 | 93 | `svc #0`, number in `x8` |
| macOS AArch64 | 4 | 1 | `svc #0x80`, number in `x16` |
| macOS x86-64 | `0x2000004` | `0x2000001` | `syscall` |
| Windows | API rather than a hard-coded number | API | `WriteFile` and `ExitProcess` |

Linux documents syscall conventions as part of its userspace interface. See [Linux's syscall manual](https://man7.org/linux/man-pages/man2/syscall.2.html). On Linux, this raw `exit` service ends only the **calling thread**. Our program has only one thread, so that ends the whole process. (Ending all the threads of a process at once is a different service, `exit_group`.)

On macOS, the numbers work, but Apple doesn't promise to keep them: normal programs call the kernel through Apple's system library, `libSystem`, which Apple keeps up to date. Windows code uses its documented API, including [`WriteFile`](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-writefile), rather than assuming a stable direct syscall number.

Rust's standard library hides these differences. On each system it calls the kernel in the supported way, so applications can use the same `std::io` API across supported platforms and handle errors without choosing syscall numbers.

The companion normally writes the greeting in one request, but handles **partial writes** by sending the remaining bytes and retries an **interrupted** request. It reports failure if a request makes no progress or returns another error. On macOS it reads the **carry flag** to distinguish a positive error number from a byte count, and declares the extra return register (`x1` on AArch64, `rdx` on x86-64) as changed. Every changed register must be declared: a successful print alone doesn't prove an assembly block correct. In ordinary Rust, use `std::io` and handle its errors. The [companion's README](hello-syscall/) explains the details.

## Why batching helps: a measurement

The main program writes **1,000,000 bytes**, one byte at a time, three ways:

| Destination | What happens |
|---|---|
| `File` directly | normally one system call per one-byte `write_all`; retries can add calls |
| `BufWriter<File>` | bytes are collected in a **buffer** in the program's memory, and written to the file in batches |
| `Vec<u8>` | bytes are only appended to memory; nothing goes to a file |

Here are example timings from an M2 Mac run, shown with the current output labels (µs is a microsecond, a millionth of a second):

```text
Writing 1000000 bytes, one byte at a time:
    straight to the kernel:     1.84s   about 1000000 write calls (estimate)
    through a BufWriter:      12.45ms   about 123 write calls (estimate)
    actual buffer capacity: 8192 bytes
    into a Vec in memory:    417.92µs   no system calls for the writes
    so each unbuffered write cost about 1827 ns extra on this machine (an estimate)
```

The buffered version was about 150 times faster **in that run**: the same million bytes, but about 123 trips to the front desk instead of a million. Your numbers will differ.

The current `BufWriter` default is **8 KiB**, but Rust documents that this may change. The program queries and prints the writer's **actual capacity**, then uses it to estimate the number of batches. With 8,192 bytes, 1,000,000 bytes need **123 batches**, including the last partial batch. [`BufWriter`'s documentation](https://doc.rust-lang.org/std/io/struct.BufWriter.html) explains the buffer and flushing.

**Read the output carefully:**

- The write-call estimates are **calculated**, not traced: normally one per byte for direct writes, and bytes ÷ actual buffer capacity rounded up for buffered writes. Partial or interrupted writes can add requests. A hypothetical zero-capacity buffer would send each byte directly.
- "No system calls for the writes" is about the appending only. Getting memory for the `Vec` can still involve the OS ([lesson 7](../07-virtual-memory/)).
- The "about 1827 ns extra" is (unbuffered time − buffered time) ÷ 1,000,000. It's the extra cost of each unbuffered write, which includes the kernel's file work, not only the switch into kernel mode and back.
- Writing to a file doesn't mean the bytes are on the disk yet: ordinary buffered file writes usually reach the OS's memory cache first. `flush()` empties the `BufWriter` into its underlying file; it doesn't request durable storage. `File::sync_all()` asks the OS to synchronise file contents and metadata with storage. It must succeed, and storage must honour that request, for the platform's durability guarantee to hold. See [Rust's `sync_all` documentation](https://doc.rust-lang.org/std/fs/struct.File.html#method.sync_all).

[Linux's write manual](https://man7.org/linux/man-pages/man2/write.2.html) explains partial writes and why a successful write alone isn't a durability guarantee.

## Does every library call ring the bell?

No. Arithmetic and accesses to ready, permitted memory can happen entirely in user mode. Some memory accesses still need kernel help for a page fault ([lesson 7](../07-virtual-memory/)). A buffer also needs the OS when it fills and writes its contents to a file.

The program also asks the OS for two pieces of information:

```text
These ask the operating system for information:
    this process's id:    54721  (normally the getpid system call)
    entries in the temp dir: 443  (includes subdirectories; entry errors are returned)
```

Every process has a number, its **process ID**, given by the OS. Reading a directory normally needs OS requests to open it, retrieve entries and close it. The program counts successfully retrieved **directory entries**, including subdirectories. If retrieving an entry fails, it returns the I/O error instead of counting the error as an entry. This isn't a count of regular files.

Even some OS information can be fetched without a system call. Linux's **vDSO** is a small piece of kernel-provided code mapped into many Linux processes, so that frequent questions, such as "what time is it?", can be answered in user mode. A library may also remember an answer. See [Linux's vDSO explanation](https://man7.org/linux/man-pages/man7/vdso.7.html).

To see the real system calls a program makes on Linux, run it under [`strace`](https://man7.org/linux/man-pages/man1/strace.1.html), a tool that prints each one. Reading the source isn't enough: one Rust function call can make several system calls, or none.

## Words to remember

| Word | Meaning |
|---|---|
| **kernel** | the core of the operating system, running in its privileged CPU mode |
| **user mode / kernel mode** | the processor's restricted mode for applications / its privileged mode for the kernel |
| **user space / kernel space** | the memory of ordinary applications / the memory of the kernel |
| **system call** | a request from a program to the kernel, made with a special instruction |
| **file descriptor** | a small number naming an open file or stream; 1 is standard output |
| **buffer** | memory where data is collected before being handed on in bigger pieces |
| **hardware interrupt** | a notification that makes the processor handle a hardware event |

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1962** | The **Atlas** computer at Manchester ([lesson 7](../07-virtual-memory/)) runs a **supervisor**, an early operating system, that manages memory and devices for the programs. Its full version became available in 1964. [Manchester's Atlas history](https://curation.cs.manchester.ac.uk/computer50/www.computer50.org/kgill/atlas/atlas.html) |
| **1960s–70s** | **Multics** develops hardware-enforced **protection rings**: several levels of privilege, like circles around a castle, with the most trusted code in the middle. Today's user mode and kernel mode are the simplest version: two rings. [The designers' 1972 paper](https://multicians.org/protection.html) explains the architecture. |
| **1969** | At Bell Labs, Ken Thompson and Dennis Ritchie develop **Unix**, borrowing ideas from Multics. Its small set of system calls, such as `open`, `read`, `write` and `close`, still shapes Linux and macOS. [Computer History Museum, 1969](https://www.computerhistory.org/timeline/1969/) |
| **1973** | Much of the Unix kernel is rewritten in **C**, so an operating system no longer has to be written in one machine's assembly language. [Dennis Ritchie's firsthand history](https://www.nokia.com/bell-labs/about/dennis-m-ritchie/chist.html) |
| **2018** | **Meltdown** ([lesson 7](../07-virtual-memory/)) shows that some processors could leak kernel memory to user programs. One defence, kernel page-table isolation, can add system-call overhead when enabled. The cost varies with hardware and workload. [Linux's KPTI documentation](https://docs.kernel.org/arch/x86/pti.html) |

The enduring goal is to let many programs share a machine without giving each one the keys to the whole hotel.

## Run it

From this lesson's directory:

```bash
cargo run --release          # buffering measurements
cargo test
cargo run -p hello-syscall   # the assembly/API companion
```

Each benchmark run **atomically creates its own temporary directory** and creates its two files there using `File::create_new`, which refuses to replace an existing file. It checks their lengths, then attempts to remove its directory on ordinary scope exit, including after an I/O error. Think of giving each guest a separate letter tray and clearing only that tray afterwards. Concurrent runs use separate directories; the tests also check isolation and preservation of existing files. If the process is forcibly stopped or cleanup fails, its own directory may remain. The companion supports Linux and macOS (on x86-64 or AArch64), and Windows.

**Try it:** why can `BufWriter` make one million small writes faster?

<details>
<summary>Show the answer</summary>

It collects the bytes in user-space memory and hands them to the OS in **fewer, larger batches**. The bytes and final file contents can be identical while the number of requests differs.

</details>

Previous: [Lesson 7: Virtual memory](../07-virtual-memory/) · Next: [Lesson 9: Why Rust](../09-why-rust/)
