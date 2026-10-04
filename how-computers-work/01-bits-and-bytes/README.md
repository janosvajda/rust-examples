<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: Bits and bytes

## The idea in one sentence

Everything inside a computer, numbers, letters, colours, music and programs, is stored as **bits**: tiny switches that are either off (0) or on (1), grouped in eights into **bytes**.

## Imagine…

Imagine a row of eight light switches. Each one is either off or on. With one switch you can show 2 different things (off, on). With two switches, 4 (off-off, off-on, on-off, on-on). Every switch you add **doubles** the number of patterns. With eight switches there are 2 × 2 × 2 × 2 × 2 × 2 × 2 × 2 = **256** different patterns.

A computer is billions of such switches. A **bit** is one switch. A **byte** is a row of eight. That's all the computer has. Everything else is a question of what we **agree** a pattern means.

## Precisely

### Why only 0 and 1?

A computer's switches are **transistors**, which let electricity through or not. Telling "on" from "off" is easy and reliable, even when the voltage wobbles a little. Telling apart ten different levels, for our ten digits, would be much harder to do reliably. So computers count with two digits: **binary**.

### Counting in binary

In our ordinary decimal numbers, each place is worth ten times the place to its right: ones, tens, hundreds. In binary, each place is worth **twice** the one to its right: 1, 2, 4, 8, 16, 32, 64, 128.

```text
      5 = 00000101    → 4 + 1
     10 = 00001010    → 8 + 2
    255 = 11111111    → 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1
```

### A byte holds 0 to 255, and then rolls over

A `u8` is one byte, so it holds 0 to 255. One more doesn't fit:

```text
255 + 1 doesn't fit: checked_add says None
wrapping_add gives 0: like a car's odometer rolling over
```

Rust makes you choose what should happen; [error handling lesson 1](../../error-handling/01-panic-vs-result/) shows all the options. Bigger types simply use more bytes: a `u32` is 4 bytes (0 to 4,294,967,295), and a `u64` is 8.

### Negative numbers: the same bits, read differently

Bits have no minus sign. Computers store negative numbers with a trick called **two's complement**: for a byte, −1 is stored as 255, −2 as 254, and so on, so that adding still works. (−1 + 1 = 255 + 1, which rolls over to exactly 0.)

```text
  -1 as i8 is stored as 11111111, which read as a u8 is 255
  -5 as i8 is stored as 11111011, which read as a u8 is 251
-128 as i8 is stored as 10000000, which read as a u8 is 128
```

The bits `11111111` mean 255 as a `u8` and −1 as an `i8`. **The bits don't know what they are.** The type tells the computer how to read them. That's one reason types matter so much in Rust.

### Letters are numbers: UTF-8

Text is stored as numbers too. The letter `A` is 65, `a` is 97. Letters from other languages, and emoji, need more than one byte:

```text
A  → 1 byte(s): 01000001
é  → 2 byte(s): 11000011 10101001
€  → 3 byte(s): 11100010 10000010 10101100
🦀  → 4 byte(s): 11110000 10011111 10100110 10000000
```

This is **UTF-8**, the way almost all text is stored today, and the way Rust stores every `String`. That's why `"🦀".len()` is 4 (bytes), while `"🦀".chars().count()` is 1 (character).

### Colours are numbers

A colour on screen is three amounts, red, green and blue, from 0 to 255 each. Three bytes:

```text
0x00FF8800 → red 255, green 136, blue 0      (orange)
```

`0x` means the number is written in **hexadecimal**, base 16, where each digit stands for exactly 4 bits. Programmers use it because two hex digits are exactly one byte. The [bouncing face](../../games-and-graphics/bouncing-face/) example draws whole pictures this way.

### Some numbers can't be stored exactly

```text
0.1 as 64 bits: 0011111110111001100110011001100110011001100110011001100110011010
0.1 + 0.2 = 0.30000000000000004
```

In binary, 0.1 goes on forever (like 1/3 = 0.333… in decimal), so a 64-bit float stores the closest value it can. Usually that's close enough. For money, it isn't: the [Mini language](../../mini/docs/syntax.md) and the [AI course](../../software-engineering-with-ai/06-clippy-and-tests-as-guardrails/) use whole numbers of cents or millionths instead.

## A bit of history

| When | What happened |
|---|---|
| **1703** | Gottfried Wilhelm Leibniz publishes a description of binary arithmetic: counting with only 0 and 1 |
| **1948** | Claude Shannon's paper founding information theory uses the word **bit**, short for "binary digit". He credits the word to his colleague John Tukey |
| **1956** | Werner Buchholz coins **byte** while designing IBM's Stretch computer. Back then a byte's size varied from machine to machine |
| **1963** | **ASCII**: one agreed table of 128 characters for English text, so different machines could exchange text |
| **1964** | IBM's System/360 makes the **8-bit byte** the standard. Almost every computer since has used it |
| **1985** | The **IEEE 754** standard fixes how floating-point numbers are stored, so `0.1 + 0.2` gives the same answer on every computer |
| **1992** | Ken Thompson and Rob Pike design **UTF-8**: every character of every language, while plain ASCII text stays exactly as it was |

## Run it

```bash
cargo run
cargo test
```

Back to the [course overview](../) · Next: [Lesson 2: Memory and addresses](../02-memory-and-addresses/)
