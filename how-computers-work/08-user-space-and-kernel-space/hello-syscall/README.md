# Companion example: a message at the kernel's front desk

This is the small assembly/API experiment for [lesson 8](../). It prints a **41-byte message**, including its newline:

```text
Hello, world! (written by a system call)
```

Imagine handing one complete letter to the Memory Hotel's manager. The request needs a destination, the address of the message, and its byte count.

## What the code does

1. Requests a write to **standard output**.
2. If only part of the message was written, sends the remaining bytes. If the request was interrupted, tries that request again.
3. Requests exit status **0** after the complete message, or **1** after an error.

Usually the greeting needs one write. The retry loop matters because an OS request doesn't promise to finish every byte in one go.

On Linux and macOS, the write and exit requests use inline assembly. On Windows, the program gets the output handle and calls `WriteFile` and `ExitProcess`.

This is a **normal Rust binary with standard-library startup**. The executable can make other OS requests before `main`; the steps above describe its explicit greeting path.

## The return value is part of the agreement

The manager returns either “I sent this many bytes” or “there was a problem.” Different desks encode those answers differently:

| Interface | Success | Failure |
|---|---|---|
| Linux raw syscall | Nonnegative byte count | Negative error number |
| macOS raw syscall | Byte count, carry flag clear | Positive error number, carry flag set |
| Windows `WriteFile` | Nonzero success result and a separate byte count | Zero result; `GetLastError` provides the reason |

On macOS, **the number alone isn't enough**. The code captures the carry flag immediately, before another instruction could change it, then converts the result into Rust's `Result<usize>`.

For example, the number 41 with carry clear means 41 bytes written; 41 with carry set means error number 41. A test checks that these are treated differently. [Apple's AArch64 kernel code](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/dev/arm/systemcalls.c) and [x86-64 code](https://github.com/apple-oss-distributions/xnu/blob/main/bsd/dev/i386/systemcalls.c) show these conventions.

## Tell the compiler what changed

An `asm!` block must declare its register inputs and outputs:

- On x86-64, `syscall` changes `rcx` and `r11`.
- Darwin also changes a second return register: **`x1` on AArch64**, **`rdx` on x86-64**. These are declared with `inlateout`: they supply an argument before the request and may hold a different value afterwards.
- The result register and the register holding the captured error flag are outputs too.

If a register isn't declared as changed, the compiler may keep relying on its old value. Leaving out an output violates Rust's assembly rules, even when a particular run looks fine. [Rust's inline-assembly rules](https://doc.rust-lang.org/reference/inline-assembly.html#rules-for-inline-assembly) explain this contract.

The message pointer must also refer to readable, live bytes for the requested length. The `// SAFETY:` comments explain the memory and register requirements.

## Send the rest of the letter

Suppose the manager sends only **10 of 41 bytes**. The next request starts at byte 10 and asks for the remaining **31**—it doesn't resend the first ten.

The loop:

- advances by the number of bytes successfully written;
- retries `Interrupted`;
- returns `WriteZero` if a nonempty request makes no progress;
- returns other errors to the caller.

Windows' length argument is a `u32`. The wrapper limits each request to that range; a larger slice would be sent in several chunks instead of having its length truncated.

Linux's raw `exit` ends the **calling thread**. This demonstration creates no extra threads, so that ends its process. The macOS and Windows exit interfaces here end the process.

For ordinary application output, use `std::io` and handle its errors. The direct macOS interface remains a platform experiment: normal applications use Apple's system library. [Linux's write manual](https://man7.org/linux/man-pages/man2/write.2.html) and [Microsoft's API documentation](https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-writefile) describe write results.

## A bit of history

System calls give ordinary programs a controlled way to request protected services. Multics's protection-ring work helped develop this separation; [the original designers' paper](https://multicians.org/protection.html) describes it.

**Fun fact:** the greeting's `!` is an ordinary byte. The CPU's special request instruction is what rings the manager's bell!

## Run it

From this directory, or any directory in this repository's Cargo workspace:

```bash
cargo run -p hello-syscall
cargo test -p hello-syscall
```

Tests cover partial writes, interruptions, zero progress, error decoding and, on supported Unix targets, real requests with an invalid file descriptor.

Supported targets: Linux and macOS on x86-64 or AArch64, plus Windows. Other targets produce an explicit compilation error.

Back to [Lesson 8: User space and kernel space](../)
