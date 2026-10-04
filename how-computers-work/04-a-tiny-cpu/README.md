<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: A tiny CPU

## The idea in one sentence

A **CPU**, or processor, follows machine instructions; this lesson models **fetch, decode and execute** in Rust so you can watch a whole little program run.

## Imagine…

Ferris is a cook with numbered recipe cards, **four small bowls** and a finger marking the current card.

A card says “put 6 in bowl 0.” Another says “add bowl 0 to bowl 2.” Another says “go back to card 12 if bowl 1 isn't empty.”

The bowls are **registers**, small places holding values inside the CPU. The finger is the **program counter**, or **PC**, which identifies the next instruction. The cards are **machine code**, stored as bytes.

Ferris follows the instructions exactly. A loop is a recipe that sends the finger backwards!

## Meet our invented machine

This is an **emulator**: a Rust program modelling an invented CPU. It doesn't build a physical processor or emulate an existing chip.

| Part | Our CPU |
|---|---|
| Memory | 256 bytes, at addresses 0–255 |
| Registers | Four: R0, R1, R2, R3; each holds a `u8` |
| Program counter | An eight-bit address |
| Arithmetic | Wraps around, like `wrapping_add` in [lesson 1](../01-bits-and-bytes/): 255 + 1 becomes 0 |
| Output | `PRINT` adds a number to the emulator's output list |
| Execution | One instruction at a time; it gives up after 1,000 instructions, in case a program loops forever |

Real processors are much bigger. For example, **AArch64**, the 64-bit Arm design used in many phones and in Apple Silicon Macs, has 31 general-purpose registers of 64 bits each, plus others for special jobs, and hundreds of different instructions. But the basic mechanism is the same, and our small model makes it visible.

## Nine instructions

Every instruction starts with one byte called the **opcode** (short for "operation code"): a number saying *which* instruction it is. `01` means `LOAD`, `02` means `ADD`, and so on. The bytes after the opcode are the **operands**: what the instruction works on, such as a register number, a value or an address. So an instruction is one, two or three bytes long.

In the table, `r` and `s` are register numbers (0–3), `v` is a value and `a` is a memory address. Numbers in the byte column are hexadecimal; register numbers and addresses in the assembly column are decimal.

| Assembly | Machine-code bytes | Meaning |
|---|---|---|
| `HALT` | `00` | Stop this emulated program. |
| `LOAD Rr, v` | `01 r v` | Put byte value `v` into register `r`. |
| `ADD Rr, Rs` | `02 r s` | Add register `s` to register `r`, wrapping. |
| `SUB Rr, Rs` | `03 r s` | Subtract register `s` from register `r`, wrapping. |
| `READ Rr, [a]` | `04 r a` | Copy memory byte `a` into register `r`. |
| `WRITE Rr, [a]` | `05 r a` | Copy register `r` into memory byte `a`. |
| `JUMP a` | `06 a` | Continue at address `a`. |
| `JNZ Rr, a` | `07 r a` | **J**ump to `a` if register `r` is **N**ot **Z**ero; otherwise, continue with the next instruction. |
| `PRINT Rr` | `08 r` | Append register `r` to the output list. |

`PRINT` is a convenience in our invented instruction set. Ordinary application code on a real CPU uses operating-system services to print, as [lesson 8](../08-user-space-and-kernel-space/) shows.

## Our recipe: 6 × 7

There is no multiply instruction, so we add **6 seven times**. Before the program starts, the emulator puts the numbers 6 and 7 into memory, at addresses 100 and 101. The program prints this listing. On each line: the address where the instruction starts, its bytes in hex, and its assembly. (The comments after `;` were added here.)

```text
1. The program, as people write it and as the CPU reads it
      0: 04 00 64  READ  R0, [100]    ; R0 = 6
      3: 04 01 65  READ  R1, [101]    ; R1 = 7: the counter
      6: 01 02 00  LOAD  R2, 0        ; R2 = 0: the total
      9: 01 03 01  LOAD  R3, 1        ; R3 = 1: what we subtract from the counter
     12: 02 02 00  ADD   R2, R0       ; total = total + 6   ← the loop starts here
     15: 03 01 03  SUB   R1, R3       ; counter = counter - 1
     18: 07 01 0c  JNZ   R1, 12       ; if the counter isn't zero, go back to 12
     21: 05 02 66  WRITE R2, [102]    ; save the answer at address 102
     24: 08 02     PRINT R2           ; show the answer
     26: 00        HALT               ; stop
    the data at 100 and 101: 6 and 7
```

**Assembly language** gives names to machine instructions, so people can read them: `ADD R2, R0` instead of `02 02 00`. An **assembler** is the program that turns assembly into bytes. In our toy, one line of assembly is exactly one machine instruction. Real assemblers add conveniences, such as names for addresses (writing `loop` instead of `12`), so the match isn't always one to one.

Why does the second instruction start at address 3 rather than 1? `READ` occupies **three bytes**: addresses 0, 1 and 2. Why is address 100 written as `64` in the byte column? **100 in decimal is 0x64 in hexadecimal** (6 × 16 + 4). In the same way, the jump target 12 is written `0c`.

## Fetch, decode, execute

For every step, the emulator:

1. **Fetches**: reads the instruction's bytes from memory, starting at the address in the PC.
2. **Decodes**: looks at the opcode to see which instruction it is, and reads its operands.
3. **Executes**: does the work, then moves the PC to the next instruction. That's normally the instruction right after this one; a jump sets the PC to a different address instead.

The program prints a **trace**: one line per instruction, showing the registers **before** that instruction runs. (The repetitions in the middle are left out here, where you see `…`.)

```text
2. Running it, one instruction at a time
     PC  instruction       R0  R1  R2  R3
      0  READ  R0, [100]    0   0   0   0
      3  READ  R1, [101]    6   0   0   0
      6  LOAD  R2, 0        6   7   0   0
      9  LOAD  R3, 1        6   7   0   0
     12  ADD   R2, R0       6   7   0   1
     15  SUB   R1, R3       6   7   6   1
     18  JNZ   R1, 12       6   6   6   1
     12  ADD   R2, R0       6   6   6   1
     …
     18  JNZ   R1, 12       6   0  42   1
     21  WRITE R2, [102]    6   0  42   1
     24  PRINT R2           6   0  42   1
     26  HALT               6   0  42   1
```

Each time round the loop, R2 grows by 6 and R1 counts down by 1. While R1 isn't zero, `JNZ` sends the PC back to 12. When R1 reaches zero, `JNZ` doesn't jump, and the PC moves on to the `WRITE` at address 21.

The final result is **42**, stored at memory address 102 and collected in the output list:

```text
3. The result
    halted after 28 instructions; printed [42]; memory[102] = 42
```

The emulator counts **28 instructions**, including `HALT`: four setup instructions, seven repetitions of three loop instructions, and three finishing instructions.

## Program bytes and data bytes

Our instructions occupy addresses 0–26. Our input values occupy 100 and 101. Both live in the same memory array.

A byte doesn't know whether it's an instruction or a number. In this model, if the PC reaches it and the CPU fetches it as an opcode, it's an instruction; if `READ` copies it into a register, it's a number. (Lesson 1 again: the meaning comes from how the bits are used.) The test `a_program_can_overwrite_itself` even uses `WRITE` to change a `HALT` byte (`00`) into a `PRINT` byte (`08`) before the PC reaches it.

**Fun experiment:** our cook can rewrite a recipe card while cooking.

Real computers also keep programs in memory, but they guard them. The operating system marks each region of memory with what may happen there ([lesson 7](../07-virtual-memory/) shows these **permissions**): data normally can't be run as instructions, and instructions normally can't be changed. That stops many attacks in which bad input is smuggled in and then run as code. Programs that really must create machine code while running, such as the JavaScript engine in a web browser (a **just-in-time compiler**), ask the operating system to change the permissions first. Our emulator can change its recipe cards because they are ordinary data in a Rust array. Changing native instructions in memory needs special platform facilities, which a library may expose through a safe API while using `unsafe` internally.

## Where the little model stops

Our CPU finishes one instruction before starting the next. Real processors are cleverer. They must preserve the instruction set's rules for the program's results, even when they overlap work; timing and cache effects can still reveal that extra work:

- **Pipelining**: like a kitchen with several cooks in a row, one fetches the next card while another decodes the card before it, and a third executes the one before that. Several instructions are in progress at once.
- **Branch prediction**: at a `JNZ`, the processor doesn't wait to find out whether it will jump. It guesses (usually correctly, because loops repeat), carries on, and throws the work away if the guess was wrong.
- **Several instructions at once**: instructions that don't depend on each other's results can run side by side.

A processor's parts work in step with a **clock** that ticks billions of times per second. A clock rate of **3 GHz** means three billion ticks (**clock cycles**) per second. That isn't the same as three billion instructions: an instruction may take several cycles, and thanks to the tricks above, several instructions may finish in one cycle.

The compiler can choose instructions cleverly too. An `if` in Rust may become a jump like our `JNZ`, or a **conditional select**: a single instruction that picks one of two values without jumping. If the compiler can prove the answer in advance, it may produce no instruction at all. [Lesson 5](../05-real-machine-code/) shows a real example.

Our emulator is a teaching machine. Real processors also have memory protection ([lesson 7](../07-virtual-memory/)), caches ([lesson 6](../06-the-cache/)), **interrupts** (notifications, for example from a keyboard or a timer, that make the processor handle an event), and several **cores**: complete processors on one chip, running different programs at the same time.

## Words to remember

| Word | Meaning |
|---|---|
| **CPU** (processor) | the part of the computer that follows instructions |
| **register** | a tiny, very fast storage place inside the CPU |
| **program counter** (PC) | a register holding the address of the next instruction |
| **machine code** | instructions stored as bytes, which the CPU reads directly |
| **opcode** | the byte (or bits) that says which instruction it is |
| **operand** | what an instruction works on: a register, a value or an address |
| **assembly language** | machine instructions written with names, for people |
| **fetch, decode, execute** | the three steps a CPU repeats for every instruction |
| **jump** | setting the PC to a different address, to repeat or skip instructions |
| **emulator** | a program that imitates a computer |

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1945** | John von Neumann writes the *First Draft of a Report on the EDVAC*. It describes a **stored-program computer**: the program is kept in the same electronic memory as the data, just like in our emulator. Before that, computers such as ENIAC were reprogrammed by changing cables and switches by hand. [Computer History Museum, 1945](https://www.computerhistory.org/timeline/1945/) |
| **1948** | On **21 June**, the **Manchester Baby** becomes the first computer to run a program stored electronically in its memory. Changing what a computer did became as easy as storing a new recipe. [The University of Manchester's account](https://www.manchester.ac.uk/about/news/university-celebrates-babys-65th-birthday/) |
| **1949** | **EDSAC**, at Cambridge University, becomes the first practical stored-program computer to provide a regular computing service to scientists. [Computer History Museum, 1949](https://www.computerhistory.org/timeline/1949/) |
| **1964** | IBM launches **System/360**: a whole family of computers, small to large, sharing one instruction set. Compatible models let customers move programs to a bigger machine in the family without rewriting them. This wasn't the first compatible computer family, but it was an influential one. [IBM's history](https://www.ibm.com/history/system-360) |
| **1971** | Intel's **4004** puts a whole CPU on one chip: a **microprocessor**, with about 2,250 transistors, made for a Japanese calculator company, Busicom. [Computer History Museum, 1971](https://www.computerhistory.org/timeline/1971/) |
| **1987** | Acorn launches the **Archimedes** series, the first personal computers built on the Arm architecture. They use ARM2; the ARM1 prototype had already worked in 1985. Later Arm designs, including AArch64, power many phones. [Arm's history](https://newsroom.arm.com/blog/evolution-of-arm-architecture-evolution-40-years) |

**Fun fact:** Baby's historic program found a factor of a number. Today's computers still use arithmetic, branches and memory, just on a vastly larger scale.

## Run it

From this lesson's directory:

```bash
cargo run
cargo test
```

**Try it:** after the third addition, what are R2 and R1?

<details>
<summary>Show the answer</summary>

After the third `ADD`, R2 is **18** and R1 is still **5**. The following `SUB` changes R1 to **4**. Watching the exact instruction matters!

</details>

Previous: [Lesson 3: Pointers are addresses](../03-pointers-are-addresses/) · Next: [Lesson 5: Real machine code](../05-real-machine-code/)
