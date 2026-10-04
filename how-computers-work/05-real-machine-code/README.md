<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Real machine code

## The idea in one sentence

A compiler translates Rust into instructions for a particular processor; this example calls a tiny assembly function and examines its actual instruction bytes.

## Imagine…

Ferris's kitchen reads one recipe-card language. Ada's kitchen reads another. Both can make the same soup, but their cooks need different cards.

Processor families also have their own **instruction sets**. Many Intel and AMD computers use **x86-64**. Apple Silicon Macs and many phones use **AArch64**, the 64-bit Arm instruction set.

Rust source is the recipe. The compiler writes the cards for the chosen **target**: the kind of computer the program is for, meaning its processor family and its operating system ([lesson 2](../02-memory-and-addresses/)). The same Rust source can usually be compiled for any target without changes. Assembly is different: it's written in one processor's own instruction language, so this lesson contains one version per processor family.

## Two real instruction languages

| | x86-64 | AArch64 |
|---|---|---|
| Instruction length | 1–15 bytes | Four bytes |
| Registers used in our addition | `rdi`, `rsi`, `rax` on Linux/macOS | `x0`, `x1` |
| Addition in our example | `lea`, an address calculator, used here as an adder (explained below) | `add` adds two register values |
| Return | `ret` | `ret` |

Every AArch64 instruction is exactly four bytes. (Older 32-bit Arm processors also have a compact instruction set, Thumb, with two- and four-byte instructions.) See [Intel's architecture manuals](https://www.intel.com/content/www/us/en/developer/articles/technical/intel-sdm.html) and [Arm's A64 instruction documentation](https://developer.arm.com/documentation/ddi0602/latest).

## The agreement at the function's door

A **calling convention** says where arguments arrive, where the result goes, and which registers a function must preserve. It's like agreeing which pocket the shopping list goes in, and which pocket the change comes back in. Without the agreement, the caller and the function would look in different places.

Our function is declared `extern "C"`. That tells Rust to use the target's **C calling convention**: the agreement that C programs on this target use, which every language, including hand-written assembly, can follow.

For this function's two `u64` arguments:

| Target | First argument | Second argument | Result |
|---|---|---|---|
| AArch64 targets demonstrated here | `x0` | `x1` | `x0` |
| x86-64 Linux/macOS | `rdi` | `rsi` | `rax` |
| x86-64 Windows | `rcx` | `rdx` | `rax` |

The operating system matters as well as the processor! The [Arm calling-convention specification](https://github.com/ARM-software/abi-aa/blob/main/aapcs64/aapcs64.rst) and [Microsoft's x64 convention](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention) describe these agreements.

Ordinary Rust functions (without `extern "C"`) use Rust's own convention. The compiler is free to change it between versions, so hand-written assembly can't rely on it. That's why our function asks for the C one. (The full set of such agreements, covering calls, data layout and more, is called an **ABI**: *application binary interface*.)

## A whole function in two instructions

The AArch64 version is:

```rust
#[unsafe(naked)]
extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
    core::arch::naked_asm!("add x0, x0, x1", "ret")
}
```

An ordinary function may need **setup and cleanup instructions**, for example to make room on the stack. A tiny function may need neither. A **naked** function gets no compiler-generated setup or cleanup: we supply the whole body, including `ret`. The compiler checks assembly syntax and constraints, but it can't prove that our instructions obey the calling convention and safely implement the function. The `unsafe` attribute marks that responsibility.

- `add x0, x0, x1` means x0 = x0 + x1. The first argument is in x0, the second in x1, and the result must end up in x0.
- `ret` returns to the code that called the function.

On x86-64 Linux/macOS the body is:

```text
lea rax, [rdi + rsi]
ret
```

`lea` stands for *load effective address*. It was made for calculating addresses such as "rdi + rsi", which is why it uses brackets, the usual notation for a memory address. But **this `lea` doesn't read memory**. It only calculates the number rdi + rsi and puts it in `rax`, so compilers often use it as a handy adder.

Calling `add_numbers(40, 2)` returns **42**. This hand-written function also deliberately **wraps**: `add_numbers(u64::MAX, 1)` returns **0**. That is the hardware operation we chose, like Rust's `wrapping_add`; it isn't the debug-overflow behaviour of ordinary Rust `+`.

The program calls it:

```text
1. Calling a function written in assembly
    this machine: AArch64 (ARM, 64-bit)
    add_numbers(40, 2) = 42
```

## Read the instruction bytes

A function is bytes in memory, just like lesson 4's program. Every instruction has an **encoding**: the number the processor manual assigns to it. The program reads the function's bytes straight out of memory, and compares them with the manual's encodings:

```text
2. The function is bytes in memory: here they are
    at 0x102a7757c: 00 00 01 8b c0 03 5f d6
    the processor manual's encoding: matches exactly
```

On this AArch64 Mac:

```text
00 00 01 8b   c0 03 5f d6
\_________/   \_________/
    add           ret
```

| Bytes | Instruction |
|---|---|
| `00 00 01 8b` | `add x0, x0, x1`: encoding `0x8b010000`, stored little-endian (lowest byte first, [lesson 2](../02-memory-and-addresses/)) |
| `c0 03 5f d6` | `ret`: encoding `0xd65f03c0`, stored little-endian |
| `48 8d 04 37` | x86-64 Linux/macOS: `lea rax, [rdi + rsi]` |
| `48 8d 04 11` | x86-64 Windows: `lea rax, [rcx + rdx]` |
| `c3` | x86-64: `ret` |

The test checks the bytes against these expected encodings. The body takes **eight bytes on AArch64**, **five on x86-64**.

A useful detail: **AArch64 instructions are always stored little-endian**, even on a system whose data uses big-endian order. Instruction byte order and data byte order are separate rules. [Arm's memory-model guide](https://developer.arm.com/-/media/Arm%20Developer%20Community/PDF/Learn%20the%20Architecture/Armv8-A%20memory%20model%20guide.pdf?revision=58b1dd0a-3800-4218-b21a-f95a0332034c) explains this.

**Fun fact:** a function that adds two enormous 64-bit numbers can fit in fewer bytes than the text “Hello, world!”

The byte-reading helper is an **`unsafe fn`**. Calling it requires an `unsafe` block and a promise: the target permits reading code through this pointer, the requested bytes satisfy Rust's single-allocation read requirements, and they remain unchanged while copied. Our calls use the assembly function above and its known byte length, with a `// SAFETY:` comment explaining why. A different function or length needs its own proof. Some systems allow code to run but not be read. This is a platform-specific experiment, not a general API for inspecting any function. [`from_raw_parts` documents the memory requirements](https://doc.rust-lang.org/std/slice/fn.from_raw_parts.html).

## An if doesn't always become a jump

For this Rust expression:

```rust
fn larger(a: i64, b: i64) -> i64 {
    if a > b { a } else { b }
}
```

A compiler can choose instructions such as these **body excerpts**:

```text
AArch64:                  x86-64 with arguments in rdi/rsi:
cmp  x0, x1               mov   rax, rsi
csel x0, x0, x1, gt       cmp   rdi, rsi
                          cmovg rax, rdi
```

- `cmp` compares two registers. The processor remembers the result (greater, equal, less) in special bits called **flags**.
- `csel x0, x0, x1, gt` (**c**onditional **sel**ect) means: x0 = if the comparison said **g**reater **t**han, then x0, else x1.
- On x86-64, `mov rax, rsi` first copies b into the result register. Then `cmovg rax, rdi` (**c**onditional **mov**e if **g**reater) replaces it with a, but only if a > b.

Neither version jumps. That can help: a jump makes the processor guess which way to go ([lesson 4's branch prediction](../04-a-tiny-cpu/)), and a wrong guess wastes time, while a select never needs a guess. But it isn't always better, so compilers may choose a jump instead. These are illustrations, not a promise about your compiler's output; the best choice depends on the target and the program.

Loops often become comparisons and jumps back, like the `JNZ` loop in [lesson 4](../04-a-tiny-cpu/). Optimisers may also **unroll** a loop (copy its body several times, so fewer jumps are needed), combine loops, or remove a loop whose result they can calculate in advance.

## Ask your compiler for its cards

From this lesson's directory:

```bash
cargo rustc --release -- --emit asm
```

`--emit asm` asks the compiler to save the assembly it generated. The file is named like `real_machine_code-<letters and digits>.s`, in **the repository root's `target/release/deps/`** folder, not inside this lesson's folder, because all the lessons share one build folder. (If you've set `CARGO_TARGET_DIR`, it's there instead.) Expect a long file: it contains the whole program, including the code that prints.

[Compiler Explorer](https://godbolt.org) also lets you inspect compiled snippets. Pick a compiler, target and optimisation settings (for example `-C opt-level=3`), then compare. Details vary between compilers and settings: the start and end instructions, which registers are chosen, and whether a register is kept to track the stack (the **frame pointer**).

## Where the program puts things

A running program's memory has several regions, each for a different kind of thing:

| Region | What lives there | In this program |
|---|---|---|
| **Code** | the machine instructions | `add_numbers` |
| **Static data** | data with storage for the whole run; initial bytes may come from the file or be zero-filled when loaded | the text `"hello"` of `GREETING` |
| **Stack** | storage for active function calls, often including addressable local values; reclaimed as calls return | a local `u64` whose address we print |
| **Heap** | memory the program asks for while running, kept as long as needed | the `u64` inside a `Box` |

The program prints one address from each:

```text
3. Code, constants, the stack and the heap: different parts of memory
    code   (add_numbers)   0x102a7757c
    static (GREETING text) 0x102ab0a0d
    stack  (a local)       0x16d389e90
    heap   (a Box)         0x1033d1a90
```

The code and the static text are close together: both come from the program file. The stack and the heap are elsewhere. Your addresses will differ; these are **virtual addresses** ([lesson 7](../07-virtual-memory/)), and Rust doesn't promise any particular locations or order. Operating systems normally make code runnable but not changeable, and stack and heap data changeable but not runnable. [Lesson 7](../07-virtual-memory/) explains these permission checks.

## Words to remember

| Word | Meaning |
|---|---|
| **instruction set** | the instruction language a processor family understands, such as x86-64 or AArch64 |
| **target** | the kind of computer a program is compiled for: processor family and operating system |
| **encoding** | the bytes that stand for one instruction |
| **calling convention** | the agreement about where a function's arguments and result go |
| **ABI** | the full set of such binary agreements between separately compiled pieces of code |
| **naked function** | a function whose instructions are exactly the assembly we wrote |
| **conditional select** | one instruction that picks one of two values, without jumping |
| **stack / heap** | memory for running functions' local variables / memory requested while running |

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1952** | Grace Hopper completes **A-0**, a program that turns more human-friendly instructions into a computer's instructions. It had several features of a modern compiler. [Computer History Museum, 1952](https://www.computerhistory.org/timeline/1952/) |
| **1957** | After work starting in 1954, IBM releases **Fortran** (*formula translation*). Its compiler let scientists write formulas instead of machine instructions, making scientific programs easier to express; programmers still needed to learn the language. [IBM's history](https://www.ibm.com/history/fortran) |
| **1985** | At Acorn, Sophie Wilson and Steve Furber complete the **ARM1**, a 32-bit processor with only about 25,000 transistors. Its simple, energy-efficient design began the Arm family, whose descendants power many phones. [Arm's account](https://newsroom.arm.com/blog/40-year-anniversary-of-arm-architecture) |
| **2020** | Apple's **M1** brought Arm-based Apple Silicon to the Mac. **Rosetta 2** translates many Intel Mac applications so they can run on Apple Silicon. [Apple's announcement](https://www.apple.com/newsroom/2020/11/apple-unleashes-m1/) |

Rosetta 2 is a **binary translator**: it converts an already compiled x86-64 program into AArch64 instructions. A compiler starts from source code instead. Both produce cards for a different kitchen.

## Run it

From this lesson's directory:

```bash
cargo run
cargo test
```

The source provides AArch64 and x86-64 versions. On any other processor family, compiling stops with the message "this lesson has machine code for AArch64 and x86-64 only". It was checked with Rust **1.99.0** during this documentation review.

On an Apple Silicon Mac, you can also build the x86-64 version and run it through Rosetta 2. This needs Rosetta and Rust's Intel Mac target (`rustup target add x86_64-apple-darwin`) installed:

```bash
cargo run --target x86_64-apple-darwin
```

**Try it:** why do Windows and Linux x86-64 use different argument registers?

<details>
<summary>Show the answer</summary>

The instruction set doesn't choose the whole function interface. The platforms use different **calling conventions**.

</details>

Previous: [Lesson 4: A tiny CPU](../04-a-tiny-cpu/) · Next: [Lesson 6: The cache](../06-the-cache/)
