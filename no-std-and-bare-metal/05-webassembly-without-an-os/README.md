<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: WebAssembly without an OS

## The idea in one sentence

`wasm32-unknown-unknown` is a **truly bare target**, with no operating system, no C library, no files and no clock, and `no_std` Rust compiles to it as a tiny module that only computes and talks to its host through functions and shared memory.

## Why this is "bare metal" you can run

Lessons 1–4 ran on a full operating system. This one doesn't. The target's name says it all: `wasm32` is the instruction set, the first `unknown` means *no known vendor*, and the second `unknown` means **no known operating system**. Inside the module there's nothing to call: no `printf`, no `malloc`, no syscalls. That's the same situation as a bare microcontroller, but you can run it right now with Node.js or any browser.

```text
bare_wasm.wasm: 801 bytes
imports (what it needs from the host): []
exports: memory, add, buffer_address, buffer_size, count_primes, count_vowels, fibonacci, to_upper_in_place
```

- **801 bytes** for the whole program, with size-optimised release settings.
- **Zero imports**: the module asks its host for nothing at all. Everything it can do is pure computation.

## How the host and the module talk

**1. Exported functions, with numbers only.**

```rust
#[unsafe(no_mangle)]
pub extern "C" fn fibonacci(n: u32) -> u64 { … }
```

```js
exports.fibonacci(90)    // 2880067194370816120n (a u64 arrives in JavaScript as a BigInt)
```

Only numbers cross the boundary: integers and floats. `#[unsafe(no_mangle)]` keeps the function's name so JavaScript can find it.

**2. Text and data: through shared memory.** A WebAssembly module has one block of memory, which the host can also read and write. To pass a string:

```text
  JavaScript                                 WebAssembly memory
  ───────────                                ─────────────────────────────
  1. address = buffer_address()  ─────────►  BUFFER (1024 bytes)
  2. write the text's bytes there  ───────►  "Hello from JavaScript…"
  3. to_upper_in_place(length)               Rust changes the bytes in place
  4. read the bytes back  ◄────────────────  "HELLO FROM JAVASCRIPT…"
```

This is exactly what tools like `wasm-bindgen` automate. Doing it by hand once shows what they generate.

**3. Panics become traps.** With nobody to print a message to, the panic handler executes WebAssembly's `unreachable` instruction. The module stops, and JavaScript sees an exception.

## Building and running

```bash
cargo build --release      # .cargo/config.toml makes this build for wasm32-unknown-unknown
node run.mjs               # loads the module, calls everything, checks every result
```

`run.mjs` doubles as this lesson's test: it checks each answer, including counting the vowels independently in JavaScript and comparing with Rust's result, and exits with an error if anything is wrong.

If the target isn't installed yet: `rustup target add wasm32-unknown-unknown`.

## Where to go from here

| You want to… | Use |
|---|---|
| call Rust from a web page with strings, objects and classes | `wasm-bindgen` and `wasm-pack` (already installed on this machine) |
| run WebAssembly outside the browser, with files and a clock | the `wasm32-wasip1` target and a runtime like `wasmtime` |
| program a real microcontroller | the [Embedded Rust Book](https://docs.rust-embedded.org/book/): a `thumbv*` target, `cortex-m-rt`, and a HAL crate for your chip |

This crate is built only for WebAssembly, so it's kept outside the repository's Cargo workspace.

Previous: [Lesson 4: Memory-mapped registers](../04-memory-mapped-registers/) · Back to the [course overview](../)
