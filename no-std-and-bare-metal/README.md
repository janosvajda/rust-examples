<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# `no_std` and Bare-Metal Rust

Most Rust programs rely on the standard library, which relies on an operating system. This course is about Rust **without** them: the code that runs on microcontrollers, in kernels and firmware, and in tiny WebAssembly modules. Every lesson builds and runs on an ordinary computer, and each one teaches a skill that carries over directly to real embedded work.

## Rust's three layers

```text
  std     files, networking, threads, println!   ← needs an operating system
  alloc   Vec, String, Box, BTreeMap              ← needs a heap allocator
  core    Option, Result, iterators, slices, fmt  ← needs nothing at all
```

`#![no_std]` means "only `core`". You keep the whole language, but everything that assumes an operating system or a heap must come from somewhere else, or be done without.

## The lessons

| # | Lesson | What it shows | Runs on |
|---|---|---|---|
| 1 | [Calling the C library](01-calling-the-c-library/) | `no_std` + `no_main`, safe wrappers around C functions, C strings, callbacks, a variadic function written in Rust; a binary 6× smaller than `std`'s hello world | macOS / Linux, via libc |
| 2 | [A core-only library](02-core-only-library/) | const-generic collections, a ring buffer, CRC-32 computed at compile time, parsing binary packets, formatting without a heap | anywhere |
| 3 | [A custom allocator](03-custom-allocator/) | `no_std` + `alloc`: writing a bump allocator, then using `Vec`, `String` and `format!` again | macOS / Linux |
| 4 | [Memory-mapped registers](04-memory-mapped-registers/) | how firmware controls hardware: volatile access, `#[repr(C)]` layouts, bit fields, a UART driver | anywhere (simulated hardware) |
| 5 | [WebAssembly without an OS](05-webassembly-without-an-os/) | a truly bare target: an 801-byte module with no imports, run from Node.js | `wasm32-unknown-unknown` |

## Where this is used

| Where | Why |
|---|---|
| microcontrollers: sensors, drones, cars, appliances | no operating system; kilobytes of memory |
| operating system kernels and bootloaders | they *are* the operating system |
| firmware and device drivers | they run before or beneath any OS |
| WebAssembly | small, fast modules with no OS underneath |
| portable libraries | a `no_std` crate works on all of the above, and in normal programs too |

## Building the lessons

Lessons 2 and 4 are ordinary workspace members: `cargo run -p no-std-core-library`, `cargo run -p memory-mapped-registers`.

Lessons 1, 3 and 5 need their own build settings (`panic = "abort"`, or the WebAssembly target), so they're kept outside the workspace. Run them from their own folders:

```bash
cd no-std-and-bare-metal/01-calling-the-c-library && cargo run
cd no-std-and-bare-metal/03-custom-allocator && cargo run
cd no-std-and-bare-metal/05-webassembly-without-an-os && cargo build --release && node run.mjs
```

## Next step: a real microcontroller

Everything here carries over to real hardware. To run Rust on a board such as a Raspberry Pi Pico, an STM32 or an nRF52, you need a `thumbv*` compilation target (`rustup target add thumbv6m-none-eabi`, for example), the `cortex-m-rt` crate for start-up code, a HAL crate for your chip, and a flashing tool like `probe-rs`. The [Embedded Rust Book](https://docs.rust-embedded.org/book/) walks through it step by step.
