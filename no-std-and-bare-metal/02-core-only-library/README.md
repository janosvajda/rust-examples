<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: A core-only library

## The idea in one sentence

A `#![no_std]` library that uses only `core` runs **everywhere**: on a microcontroller, in a browser, in an operating system kernel, and in ordinary programs. Writing one means doing without the heap, which turns out to be very manageable.

## Why write libraries this way

Most of the Rust ecosystem's most-used crates, like `serde`, `heapless`, `nom`, `crc` and `embedded-hal`, can be compiled with `no_std`. One library, every platform. And a `no_std` library works in normal `std` programs too, because `core` is part of `std`. This lesson's demo, `src/main.rs`, is an ordinary program using the `no_std` library in `src/lib.rs`.

## No heap? Use sizes known at compile time

Without `alloc` there's no `Vec`, `String` or `Box`. The replacement is **const generics**: the capacity becomes part of the *type*, so the memory needed is fixed when the program is compiled:

```rust
let mut readings: FixedVec<i16, 3> = FixedVec::new();   // room for exactly 3, no allocation
readings.push(190)    // Err(190) when full: the item is handed back, nothing grows
```

```text
FixedVec<i16, 3>:  24 bytes
ByteRing<16>:      32 bytes
StackString<48>:   56 bytes      ← every size known before the program runs
```

On a device with a few kilobytes of RAM, that's exactly what you want: memory use can't grow at runtime, so it can't run out at runtime.

## What's in the library

| Item | What it is | Typical use on a device |
|---|---|---|
| `FixedVec<T, N>` | a `Vec` with a fixed capacity | a list of the last N sensor readings |
| `ByteRing<N>` | a ring buffer of bytes | bytes arriving from a serial port faster than they're processed |
| `crc32(&[u8])` | the standard CRC-32 checksum | detecting corrupted messages |
| `SensorReading::parse` | decoding a binary packet from bytes | reading data from a radio or a bus |
| `StackString<N>` + `write!` | formatting text into a fixed buffer | writing text to a display or a log port |

## Ideas worth noticing

**A lookup table computed at compile time.**

```rust
const CRC32_TABLE: [u32; 256] = build_crc32_table();    // a const fn, run by the compiler
```

The 1 KB table is calculated while compiling and stored in the binary. On a microcontroller it lives in flash memory, and computing it costs nothing at startup. A test checks the result against CRC-32's official check value: `crc32("123456789") = 0xCBF43926`.

**Parsing never panics.** Data from a wire can be anything: noise, half a message, a flipped bit. Every way a packet can be wrong is a variant of `PacketError` (wrong length, bad sync byte, checksum mismatch), never a crash. In the demo, a single flipped bit is caught by the checksum.

**Formatting without `format!`.** `core::fmt::Write` is the trait behind `write!`. Implement its one method, `write_str`, on a fixed-size buffer, and the full formatting machinery works, floats and `{:>8}` included. If the text doesn't fit, `write!` returns an error instead of growing.

**Bytes ↔ numbers:** `i16::from_be_bytes` / `to_be_bytes` convert between numbers and their bytes in a chosen byte order (`be` = big-endian, the usual order on networks and buses).

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: Calling the C library](../01-calling-the-c-library/) · Next: [Lesson 3: A custom allocator](../03-custom-allocator/)
