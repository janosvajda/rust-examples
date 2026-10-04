<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Real machine code

## The idea in one sentence

A real processor works just like [lesson 4](../04-a-tiny-cpu/)'s tiny CPU: a compiled Rust function is a list of machine-code instructions in memory, and you can look at them, as assembly, and even as the raw bytes.

## Imagine…

Imagine that every brand of kitchen uses its own card language. One kitchen's cards say "pour, stir, wait". Another's say "mix, heat, pause". The dish is the same, but a cook from one kitchen can't read the other kitchen's cards.

Processors are like that. Intel and AMD chips speak **x86-64**. Phones and Apple's Macs speak **AArch64** (ARM). A Rust program is written once, and the compiler, the translator, writes a separate set of cards for each kind of processor.

## Precisely

### Two instruction sets

| | x86-64 | AArch64 (ARM, 64-bit) |
|---|---|---|
| found in | most PCs and servers (Intel, AMD) | phones, tablets, Apple Silicon Macs, many servers |
| instruction length | **1 to 15 bytes**, varying | always **4 bytes** |
| general-purpose registers | 16 (64-bit) | 31 (64-bit) |
| style | many complex instructions (*CISC*) | fewer, simpler instructions (*RISC*) |

### A Rust function, compiled

These are the real compiler outputs (`--release`) for both processors. First, adding two numbers:

```rust
pub fn add(a: u64, b: u64) -> u64 { a + b }
```

```text
AArch64:                          x86-64 (macOS):
    add   x0, x1, x0                  push  rbp
    ret                               mov   rbp, rsp
                                      lea   rax, [rdi + rsi]
                                      pop   rbp
                                      ret
```

The arguments arrive in agreed registers: `x0` and `x1` on ARM, `rdi` and `rsi` on x86-64. The result goes into an agreed register too: `x0` on ARM, `rax` on x86-64. This agreement is the **calling convention**, and it's how separately compiled code fits together. (`push rbp` … `pop rbp` set up a *frame pointer*, which macOS requires on Intel so that debuggers can find their way through the stack.)

An `if`, without any jump:

```rust
pub fn max(a: i64, b: i64) -> i64 { if a > b { a } else { b } }
```

```text
AArch64:                          x86-64 (macOS):
    cmp   x0, x1                      push   rbp
    csel  x0, x0, x1, gt              mov    rbp, rsp
    ret                               mov    rax, rsi
                                      cmp    rdi, rsi
                                      cmovg  rax, rdi
                                      pop    rbp
                                      ret
```

There's no jump here. `csel` ("conditional select") and `cmovg` ("conditional move if greater") pick one of two values based on the comparison. Avoiding a jump keeps the processor's pipeline flowing, so the compiler prefers this when it can.

A loop: add up bytes until a zero byte.

```rust
pub fn sum_until_zero(bytes: &[u8]) -> u32 {
    let mut total = 0;
    for &b in bytes {
        if b == 0 { break; }
        total += b as u32;
    }
    total
}
```

```text
AArch64:
    cbz   x1, LBB0_5             ; the slice is empty? jump to the end
    mov   x8, x0                 ; x8 = the address of the first byte
    mov   w0, #0                 ; total = 0
LBB0_2:                          ; ← the loop
    ldrb  w9, [x8], #1           ; load one byte, and move the address on by 1
    cbz   w9, LBB0_4             ; the byte is zero? leave the loop
    add   w0, w0, w9             ; total += byte
    subs  x1, x1, #1             ; one byte fewer to go
    b.ne  LBB0_2                 ; not done? back to the loop
LBB0_4:
    ret
LBB0_5:
    mov   w0, #0
    ret
```

This is lesson 4's loop again, with real instructions: `ldrb` is a READ, `add` is an ADD, and `b.ne` ("branch if not equal") is a `JNZ`. Every `for`, `while` and `if` turns into these few building blocks.

### Seeing it for yourself

Ask the compiler for the assembly of your own code:

```bash
cargo rustc --release -- --emit asm      # the .s files are in target/release/deps/
```

Or use [Compiler Explorer](https://godbolt.org), a website that shows the assembly as you type, for many languages and processors.

### A function written in assembly, and its bytes

This lesson's program contains a **naked function**: a whole function written in assembly, with nothing added by the compiler. `cfg_select!` picks the right version for the processor it's built for:

```rust
#[unsafe(naked)]
extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
    core::arch::naked_asm!("add x0, x0, x1", "ret")      // the AArch64 version
}
```

Then the program reads the function's machine code back out of memory, as bytes:

```text
AArch64:  at 0x1026fb57c: 00 00 01 8b c0 03 5f d6
          the processor manual's encoding: matches exactly
x86-64:   at 0x10207d620: 48 8d 04 37 c3
          the processor manual's encoding: matches exactly
```

| Bytes | Instruction | Notes |
|---|---|---|
| `00 00 01 8b` | `add x0, x0, x1` | the 32-bit number `0x8b010000`, stored little-endian ([lesson 2](../02-memory-and-addresses/)) |
| `c0 03 5f d6` | `ret` | `0xd65f03c0`: every AArch64 instruction is exactly 4 bytes |
| `48 8d 04 37` | `lea rax, [rdi + rsi]` | x86-64: 4 bytes for this one… |
| `c3` | `ret` | …and just 1 byte for this one |

A test checks these bytes on every run. **A function really is just bytes in memory**, exactly like lesson 4's program. Reading them uses a raw pointer, which is why that part of the code is `unsafe` ([raw pointers](../../pointers-and-memory/06-raw-pointers-and-unsafe/) explains what that means).

A naked function has no setup code from the compiler, so it must follow the calling convention by hand. That's why it's marked `#[unsafe(naked)]`: the compiler can't check the assembly. Normal Rust code never needs this. It's for special cases such as interrupt handlers, operating-system code and the first instructions that run when a computer starts.

### Where things live

```text
code   (add_numbers)   0x1026fb57c
static (GREETING text) 0x102734a0d
stack  (a local)       0x16d705f40
heap   (a Box)         0x102fc9ad0
```

A running program's memory is divided into regions: the **code**, **constants** (like the text `"hello"`), the **stack** ([pointers lesson 1](../../pointers-and-memory/01-stack-and-heap/)) and the **heap**. Each region has its own permissions: code may be run but not changed, and the stack may be changed but not run. [Lesson 7](../07-virtual-memory/) shows who enforces this.

## A bit of history

| When | What happened |
|---|---|
| **1940s** | Programs are written directly as numbers: machine code |
| **early 1950s** | **Assembly languages** let programmers write `ADD` instead of a number. An *assembler* program translates them |
| **1952** | Grace Hopper's **A-0** system, one of the first **compilers**: a program that translates a higher-level description into machine code |
| **1957** | **Fortran**, the first widely used high-level language: its compiler produced code nearly as fast as hand-written assembly |
| **1978** | Intel's **8086**: the start of the **x86** family. Today's Intel and AMD processors can still run its instructions |
| **1985** | Acorn's **ARM1**, the first ARM processor, designed to be simple and efficient |
| **2003** | AMD's Opteron brings **x86-64**: 64-bit x86. Intel adopted the same instruction set |
| **2011** | Arm announces **AArch64** (ARMv8), the 64-bit ARM instruction set |
| **2020** | Apple's **M1** moves the Mac from x86-64 to ARM. Its **Rosetta 2** translates x86-64 programs to ARM; that's how this lesson's x86-64 version ran on an ARM Mac |

## Run it

```bash
cargo run
cargo test
```

On an Apple Silicon Mac, `cargo run --target x86_64-apple-darwin` runs the Intel version through Rosetta. The target is added with `rustup target add x86_64-apple-darwin`.

Previous: [Lesson 4: A tiny CPU](../04-a-tiny-cpu/) · Next: [Lesson 6: The cache](../06-the-cache/)
