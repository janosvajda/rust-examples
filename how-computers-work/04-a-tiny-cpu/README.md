<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: A tiny CPU

## The idea in one sentence

A processor (CPU) does one very simple thing, billions of times per second: **fetch** the next instruction from memory, **decode** what it means, **execute** it, and move on. This lesson builds a complete, tiny CPU in Rust, so you can watch every step.

## Imagine…

Imagine a cook in a kitchen with a long row of numbered recipe cards. The cook has only **four small bowls** on the counter, and one finger pointing at the current card.

The cook reads the card under the finger ("pour bowl 0 into bowl 2"), does exactly that, and moves the finger to the next card. Some cards say "go back to card 12": that's how the cook repeats things. Some say "go back to card 12, **but only if** bowl 1 isn't empty": that's how the cook decides things.

The cook isn't clever at all. They only follow cards, but extremely fast. A real CPU is that cook: the bowls are **registers**, the finger is the **program counter**, and the cards are **machine code**.

## The machine

This lesson's CPU is invented, but it works like a real one:

| Part | This CPU | A real one, like Apple's M2 |
|---|---|---|
| memory | 256 bytes | gigabytes |
| registers | 4 (R0–R3), 8 bits each | 31 general-purpose registers, 64 bits each |
| instructions | 9 | hundreds |
| speed | as fast as the Rust program runs it | several billion instructions per second, per core |

| Instruction | Bytes | Does |
|---|---|---|
| `HALT` | `00` | stop |
| `LOAD Rr, v` | `01 r v` | put the number `v` into register `r` |
| `ADD Rr, Rs` | `02 r s` | `R[r] = R[r] + R[s]` |
| `SUB Rr, Rs` | `03 r s` | `R[r] = R[r] - R[s]` |
| `READ Rr, [a]` | `04 r a` | copy the byte at memory address `a` into register `r` |
| `WRITE Rr, [a]` | `05 r a` | copy register `r` into memory address `a` |
| `JUMP a` | `06 a` | continue at address `a` |
| `JNZ Rr, a` | `07 r a` | continue at address `a` **if** register `r` isn't zero |
| `PRINT Rr` | `08 r` | show register `r` |

## Machine code and assembly

The program multiplies 6 × 7. This CPU has no multiply instruction, so it adds 6, seven times:

```text
  0: 04 00 64  READ  R0, [100]
  3: 04 01 65  READ  R1, [101]
  6: 01 02 00  LOAD  R2, 0
  9: 01 03 01  LOAD  R3, 1
 12: 02 02 00  ADD   R2, R0
 15: 03 01 03  SUB   R1, R3
 18: 07 01 0c  JNZ   R1, 12
 21: 05 02 66  WRITE R2, [102]
 24: 08 02     PRINT R2
 26: 00        HALT
```

The middle column is **machine code**: the only thing the CPU understands, numbers in memory. The right column is **assembly language**: the same instructions, written so people can read them. Each line in assembly is exactly one instruction in machine code. `04 00 64` and `READ R0, [100]` are the same thing: `04` means READ, `00` means register 0, and `64` is 100 in hexadecimal.

## Fetch, decode, execute

```rust
fn run(&mut self, …) {
    loop {
        let instruction = self.fetch_and_decode()?;   // 1 + 2: read the bytes at PC, work out what they mean
        if !self.execute(instruction) { break; }      // 3: do it, and move PC on
    }
}
```

Each line below is one turn of that loop. The registers are shown **before** the instruction runs:

```text
 PC  instruction       R0  R1  R2  R3
  0  READ  R0, [100]    0   0   0   0
  3  READ  R1, [101]    6   0   0   0
  6  LOAD  R2, 0        6   7   0   0
  9  LOAD  R3, 1        6   7   0   0
 12  ADD   R2, R0       6   7   0   1
 15  SUB   R1, R3       6   7   6   1
 18  JNZ   R1, 12       6   6   6   1     ← R1 is 6, not 0: jump back to 12
 12  ADD   R2, R0       6   6   6   1
 …
 18  JNZ   R1, 12       6   0  42   1     ← R1 is 0: don't jump
 21  WRITE R2, [102]    6   0  42   1
 24  PRINT R2           6   0  42   1
 26  HALT               6   0  42   1

halted after 28 instructions; printed [42]; memory[102] = 42
```

Watch `R2` grow by 6 each time round, while `R1` counts down from 7. The loop exists only because of one instruction: **`JNZ`, a conditional jump**. Every `if`, `while` and `for` in every programming language becomes conditional jumps like this one.

## Program and data share one memory

The program sits at addresses 0–26, and the numbers 6 and 7 at addresses 100 and 101, in the **same** memory. To the CPU, they're all just bytes. Whether a byte is an instruction or a number depends only on whether the program counter ever points at it.

That's called the **stored-program** design, and almost every computer works this way. It has a surprising consequence, which a test demonstrates: a program can **change its own instructions**. `a_program_can_overwrite_itself` writes the byte `08` over a `HALT`, which turns it into a `PRINT` just before the CPU reaches it. Attackers have used exactly this, writing their own machine code into memory and tricking a program into running it. Modern systems defend against it by marking memory as *either* writable *or* executable, but not both ([lesson 7](../07-virtual-memory/)).

## What this tiny CPU leaves out

A real processor follows the same fetch-decode-execute idea, but adds a great deal to go faster:
- **Pipelining:** while one instruction executes, the next ones are already being fetched and decoded.
- **Several instructions at once,** whenever they don't depend on each other, sometimes even out of order.
- **Prediction:** a real CPU guesses which way a conditional jump will go before it knows, and starts working on that path.
- **Several cores:** several complete CPUs on one chip, each running its own instructions.

The result must always be **as if** the instructions ran one at a time, in order. So the simple model in this lesson is the right way to think about any program.

## A bit of history

| When | What happened |
|---|---|
| **1945** | **ENIAC**, one of the first electronic computers, is programmed by plugging cables and setting switches: changing the program could take days |
| **1945** | John von Neumann's report on the planned **EDVAC** computer describes keeping the **program in memory, together with the data**: the stored-program design, still called the *von Neumann architecture* |
| **1948** | On 21 June, the **Manchester Baby** in England runs the first program stored in a computer's electronic memory |
| **1971** | The **Intel 4004**, the first commercial microprocessor: a complete CPU on one chip, with 2,300 transistors |
| **today** | Apple's M2 has about 20 billion transistors, and still runs fetch, decode, execute |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 3: Pointers are addresses](../03-pointers-are-addresses/) · Next: [Lesson 5: Real machine code](../05-real-machine-code/)
