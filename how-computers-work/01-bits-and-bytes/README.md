<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: Bits and bytes

## The idea in one sentence

An ordinary digital computer represents its information using **bits**, each with a value of **0 or 1**; eight bits make a **byte**.

## Imagine…

Ferris builds a secret-message board with eight light switches. Off means 0; on means 1.

One switch gives two patterns. Two switches give four: `00`, `01`, `10`, `11`. Each extra switch doubles the possibilities. Eight switches give 2 × 2 × 2 × 2 × 2 × 2 × 2 × 2 = **256 different patterns**.

Ferris and Ada must agree what the patterns mean. Does `01000001` mean the number 65, the letter A, or part of a picture? **The bits alone don't tell us.** The encoding, and the program using it, supply the meaning.

The switches are a picture of two distinguishable states. Real computers use electrical circuits, and storage can use charge or magnetism. A bit is a unit of information, not necessarily one physical switch or transistor.

## Counting with two digits

In the numbers we use every day (**decimal**), each place is worth ten times the place to its right: 1, 10, 100, 1000. In **binary**, each place is worth **two** times the place to its right: 1, 2, 4, 8, 16, 32, 64, 128. To read a binary number, add up the place values where the bit is 1:

```text
place: 128 64 32 16  8  4  2  1
bit:     0  0  0  0  0  1  0  1   → 4 + 1 = 5

     10 = 00001010   → 8 + 2
    255 = 11111111   → 128 + 64 + 32 + 16 + 8 + 4 + 2 + 1
```

Leading zeroes don't change the number. They help us see all eight bits. The program prints a few numbers this way:

```text
  0 = 00000000
  1 = 00000001
  2 = 00000010
  3 = 00000011
  4 = 00000100
  5 = 00000101
 10 = 00001010
255 = 11111111
```

## A shorter way to write bits: hexadecimal

Long rows of 0s and 1s are hard to read, so programmers often write bytes in **hexadecimal** ("hex", base 16). Hex needs sixteen digits, so after 0–9 it continues with letters:

| Hex digit | 0–9 | a | b | c | d | e | f |
|---|---|---|---|---|---|---|---|
| value | 0–9 | 10 | 11 | 12 | 13 | 14 | 15 |

One hex digit stands for exactly **four bits** (16 = 2 × 2 × 2 × 2), so we can write any byte with **two** hex digits, adding a leading zero when needed (`0a` means 10). Rust also accepts the shorter spelling `0xA`:

```text
hex 41  →  4 = 0100, 1 = 0001   →  01000001  =  65
hex c3  →  c = 1100, 3 = 0011   →  11000011  = 195
hex ff  →  f = 1111, f = 1111   →  11111111  = 255
```

In Rust, `0x` in front marks a hex number (`0x41` is 65), and `0b` marks a binary number (`0b0100_0001` is also 65; the `_` is only for readability).

## What happens when the number doesn't fit?

Rust's `u8` is an **unsigned 8-bit integer**: "unsigned" means no negative numbers, so it holds whole numbers from **0 to 255**. That's 256 values, including zero.

What should 255 + 1 be, in eight bits? Rust lets you choose:

```text
255.checked_add(1)  = None
255.wrapping_add(1) = 0
```

- `checked_add` returns `None` when the answer doesn't fit. `None` is Rust's way of saying "there is no answer": the result type is `Option<u8>`, which is either `Some(number)` or `None`.
- `wrapping_add` keeps only the lowest eight bits. 256 in binary is `1_00000000`, nine bits; drop the ninth and you're left with `00000000`, which is 0. It's like a car's odometer going from 999 to 000.

Ordinary `+` follows a third rule, which depends on how the program was built:

- `cargo run` makes a **debug build**: quick to compile, with extra checks. There, integer overflow **panics**. An unhandled panic in `main` ends this run; [lesson 3](../03-pointers-are-addresses/) explains panics.
- `cargo run --release` makes a **release build**: optimised for speed. There, overflow **wraps around**, like `wrapping_add`.

That's the default; it can be configured. When the behaviour matters, say what you mean with `checked_add` or `wrapping_add`. See [Rust's overflow rules](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow).

A `u32` has 32 bits, or four bytes, and holds 0 to 4,294,967,295. More bits give more possible values.

## Negative numbers: the same pattern, a different meaning

Rust's `i8` ("signed 8-bit integer") holds **−128 to 127**. It uses **two's complement**, a way to store negative numbers in the same eight bits. The program reads the same patterns both ways:

```text
00000001  as u8:   1   as i8:    1
01111111  as u8: 127   as i8:  127
10000000  as u8: 128   as i8: -128
11111011  as u8: 251   as i8:   -5
11111111  as u8: 255   as i8:   -1
```

The rule: for an `i8`, a pattern whose leftmost bit is 1 means its unsigned value **minus 256**. For example, 251 − 256 = −5. This isn't a separate minus-sign bit: all eight bits take part. The clever part is that adding still works with the same circuits: −1 + 1 is `11111111` + `00000001`, which wraps to `00000000`, exactly 0.

**Try it:** what does `11111110` mean?

<details>
<summary>Show the answer</summary>

It means **254** as a `u8`, or **−2** as an `i8` (254 − 256 = −2).

</details>

## Letters and emoji are bytes too

Text is stored in two steps:

1. **Unicode** assigns numbers to text elements such as letters, symbols and combining marks. These numbers are **code points**, written `U+` and the number in hex. One visible character can involve several code points. `A` is U+0041 (65), `é` is U+00E9, `€` is U+20AC, and 🦀 is U+1F980. Unicode has room for more than a million code points.
2. **UTF-8** says how to store those numbers as bytes. Small numbers take one byte; bigger ones take two, three or four.

```text
A  U+0041  → 1 byte(s): 41
é  U+00E9  → 2 byte(s): c3 a9
€  U+20AC  → 3 byte(s): e2 82 ac
🦀  U+1F980  → 4 byte(s): f0 9f a6 80
```

`A` keeps its old one-byte value, 0x41, the same as in the older ASCII code. That's a big reason UTF-8 was adopted so widely: existing **ASCII text** kept exactly the same bytes. Rust's `String` and `str` always hold UTF-8.

```rust
assert_eq!("🦀".len(), 4);             // bytes
assert_eq!("🦀".chars().count(), 1);   // Unicode scalar values
```

A **Unicode scalar value** is a code point other than one of the reserved **surrogate** code points (U+D800–U+DFFF). Rust's `char` holds one scalar value and occupies four bytes; UTF-8 may encode that same value in one to four bytes. It isn't always a whole visible character. For example, `"e\u{301}"` is an `e` followed by a combining accent: **three bytes, two scalar values**, usually displayed together as **é**. Some emoji also combine several scalar values. See [Rust's explanation of `char`](https://doc.rust-lang.org/std/primitive.char.html).

## A colour recipe

One common colour format uses three 8-bit amounts: **red, green and blue**, each from 0 to 255.

```text
red = 255, green = 136, blue = 0   → an orange colour
```

The program keeps all three in one 32-bit number, `0x00FF8800`, one byte each: `00` (unused here), then `FF` (red), `88` (green) and `00` (blue). To get the amounts back out, it uses **shifts** and **masks**:

```rust
let colour = 0x00FF_8800u32;
let red   = (colour >> 16) & 0xFF;
let green = (colour >> 8)  & 0xFF;
let blue  =  colour        & 0xFF;
```

- **`>> 16` shifts** all the bits 16 places to the right. Red's byte was third from the right; after the shift, it's the rightmost byte. (The bits that move off the right edge are dropped.)
- **`& 0xFF` masks**: `&` keeps a bit only where *both* numbers have a 1, and `0xFF` is `11111111` in the lowest byte and 0 everywhere else. So it keeps the lowest byte, and clears all the others.

```text
0x00FF8800 → red 255, green 136, blue 0
```

It's like mixing three pots of paint, though screen light mixes differently from physical paint. Other image formats may add transparency or use more bits per colour.

## Why does 0.1 + 0.2 look odd?

```text
0.1 + 0.2 = 0.30000000000000004
```

Rust's `f64` stores numbers in the IEEE 754 **binary64** format: 64 bits, used like scientific notation in base 2. (In decimal, scientific notation writes 1500 as 1.5 × 10³.) The program shows how 0.1 is stored:

```text
0.1 is stored as 0.100000000000000005551115123126
its 64 bits: sign 0  exponent 01111111011  fraction 1001100110011001100110011001100110011001100110011010
```

- the **sign** bit: 0 means positive;
- the **11 exponent bits**: for a normal number such as 0.1, subtract **1023** from the stored number. Here 1019 − 1023 = −4, so we multiply by 2⁻⁴, or 1/16;
- the **52 fraction bits**: the binary digits after an implied leading `1.`. Here `1.100110011…` is approximately 1.6 in decimal. Multiplying it by 1/16 gives approximately 0.1.

In binary, 1/10 repeats forever, just as 1/3 = 0.333… repeats forever in decimal. Binary64 keeps a finite number of digits, so it **rounds** to a nearby representable value. That's why 0.1 is stored as 0.1000000000000000055…, and why adding the stored approximations of 0.1 and 0.2 gives 0.30000000000000004. Zero, extremely tiny numbers, infinity and NaN use special rules. [Rust's `f64` documentation](https://doc.rust-lang.org/std/primitive.f64.html) describes the format.

For a toy shop with prices in whole cents, storing 129 cents as the integer `129` avoids approximating 1.29 in the price calculation. You still need rules for division and rounding. Different jobs need different number types.

## A bit of history

| When | What happened, and why it mattered |
|---|---|
| **1948** | Claude Shannon publishes *A Mathematical Theory of Communication*, the start of information theory. It measures information in **bits**, a word he credits to his colleague John Tukey. [The original paper](https://archive.org/details/bstj27-3-379) |
| **1964** | IBM introduced **System/360**, which helped popularise the eight-bit byte across a family of compatible computers. Earlier machines didn't all use the same byte size. [IBM's history](https://www.ibm.com/history/system-360) |
| **1987–1991** | **Unicode** begins as a project in 1987, to build a shared character encoding for the world's writing systems. The Unicode Consortium is founded in January 1991. [Unicode's history](https://www.unicode.org/history/summary.html) |
| **1992** | Ken Thompson and Rob Pike developed **UTF-8**, preserving ordinary ASCII bytes while allowing a much larger set of text symbols. [Pike's firsthand account](https://www.cl.cam.ac.uk/~mgk25/ucs/utf-8-history.txt) |

**Fun fact:** Pike remembers the UTF-8 design taking shape on a placemat in a New Jersey diner. A scribble over dinner helped computers share text around the world!

## Run it

From this lesson's directory:

```bash
cargo run
cargo test
```

The program shows binary patterns, overflow choices, signed numbers, UTF-8, RGB and floating-point rounding. Predict a result first, then compare it with the output.

Back to the [course overview](../) · Next: [Lesson 2: Memory and addresses](../02-memory-and-addresses/)
