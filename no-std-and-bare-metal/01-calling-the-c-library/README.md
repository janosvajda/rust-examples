<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: `no_std` Rust calling the C library

A Rust program that runs **without Rust's standard library** (`#![no_std]`) and uses the **C standard library** (libc) for everything the operating system provides: printing, memory, sorting, environment variables. Along the way it shows the most important pattern in Rust's foreign function interface (FFI): wrapping unsafe C calls in small, **safe** Rust functions.

## What is `no_std`?

Rust's standard library comes in three layers:

```text
  ┌───────────────────────────────────────────────────────────────────┐
  │ std     files, networking, threads, println!, the heap, time…     │  needs an operating system
  │  ┌─────────────────────────────────────────────────────────────┐  │
  │  │ alloc   Vec, String, Box, Rc: needs a heap allocator         │  │  needs some heap
  │  │  ┌───────────────────────────────────────────────────────┐  │  │
  │  │  │ core   Option, Result, iterators, slices, &str,        │  │  │  needs nothing at all
  │  │  │        traits, math, formatting machinery               │  │  │
  │  │  └───────────────────────────────────────────────────────┘  │  │
  │  └─────────────────────────────────────────────────────────────┘  │
  └───────────────────────────────────────────────────────────────────┘
```

- **`core`** works anywhere, even on a chip with no operating system and no heap.
- **`alloc`** adds heap-allocated types, once you provide an allocator.
- **`std`** adds everything that needs an operating system.

`#![no_std]` means: **link only `core`**. You keep the language and its zero-cost tools (`Option`, iterators, slices, pattern matching, traits), but lose everything that assumes an operating system underneath.

| Without `std` you lose… | So in this example we use… |
|---|---|
| `println!` | C's `printf` and `snprintf` |
| `Vec`, `String`, `Box` | C's `malloc` / `free`, or fixed-size buffers on the stack |
| `std::env` | C's `getenv` |
| `slice::sort` *(that one is actually in `core`)* | C's `qsort`, to show a Rust callback called from C |
| the normal `fn main` | `#![no_main]` and our own C-style `main` |
| panic messages and unwinding | our own `#[panic_handler]`, which calls `abort()` |

## Why and where is it used?

`no_std` is how Rust runs in places with **no operating system**, or where every byte matters:

| Where | Why `no_std` |
|---|---|
| **Microcontrollers and embedded devices**: Arduino-class boards, sensors, drones, cars | there's no OS, no files, no threads; often only kilobytes of memory |
| **Operating system kernels and bootloaders** (Linux kernel modules in Rust, Redox OS) | they *are* the OS, so they can't rely on one |
| **Firmware and drivers** | they run before or beneath any OS |
| **WebAssembly modules** that must be tiny | the browser provides no OS functions anyway |
| **Libraries that should work everywhere** | crates like `serde`, `heapless` and many parsers support `no_std`, so they also run on embedded devices |
| **Very small binaries** | see the measurement below |

### This example is a special case

On a real microcontroller there's **no C library either**: you talk to the hardware directly, usually through a hardware abstraction crate. This example runs on a normal desktop OS, where the C library *is* available. That makes it easy to run and experiment with, and it teaches the two skills that carry over to every `no_std` and FFI project:
1. living with only `core`;
2. calling C code safely.

## Measured: how much smaller?

Both built with the same release settings (`opt-level = 3`, LTO, `panic = "abort"`, symbols stripped) on an Apple M2:

| Program | Size |
|---|---|
| `println!("Hello World!")` with `std` | **302,432 bytes** |
| this `no_std` program, doing seven different things | **50,336 bytes** |

About **6× smaller**, while doing more. Both use the operating system's C library as a shared library (`libSystem` on macOS), so the difference is Rust's standard library itself. On a microcontroller with 32 KB of flash memory, that difference decides whether the program fits at all.

## The examples

Each C function is wrapped in a **safe** Rust function. The rest of the program never touches raw pointers.

| # | C function | Safe Rust wrapper | Shows |
|---|---|---|---|
| 1 | `strstr` | `find(haystack: &CStr, needle: &CStr) -> Option<&CStr>` | C strings, null pointers → `Option` |
| 2 | `strlen` | `c_length(&CStr) -> usize` | how C finds the end of a string |
| 3 | `qsort` | `sort_with_qsort(&mut [i32])` | a **Rust function called by C** (`extern "C"` callback) |
| 4 | `malloc` / `free` | `sum_of_malloc_buffer(count) -> Option<i64>` | manual heap memory, as in C |
| 5 | `snprintf` | `format_temperature(&mut [u8; 32], c_int) -> &CStr` | formatting into a stack buffer, without `format!` |
| 6 | `getenv`, `getpid` | `env_var_length(&CStr) -> Option<usize>` | the environment and the process |
| 7 | *(written in Rust)* | `unsafe extern "C" fn sum_ints(count: c_int, ...) -> c_int` | a **variadic** function defined in Rust, callable from C |

```text
no_std Rust, talking directly to the C library

1. strstr: found "World!"
2. strlen("Hello World!") = 12
3. qsort: 3 7 19 25 42
4. malloc/free: 1 + 2 + ... + 1000 = 500500
5. snprintf: "21 degrees"
6. getenv: HOME is set (15 characters)
   getpid: this process is 90091, started with 1 argument(s)
7. variadic Rust function: sum_ints(4, 10, 20, 30, 40) = 100
```

## Key ideas, explained

### 1. Wrap every `unsafe` call in a safe function

The compiler can't check what C code does, so every call into C is `unsafe`. The rule professionals follow: keep each `unsafe` block **tiny**, check everything C requires *before* calling it, write a `// SAFETY:` comment explaining why the call is correct, and expose a **safe** function.

```rust
pub fn find<'a>(haystack: &'a CStr, needle: &CStr) -> Option<&'a CStr> {
    // SAFETY: both pointers come from valid CStrs: non-null and nul-terminated.
    let found = unsafe { libc::strstr(haystack.as_ptr(), needle.as_ptr()) };
    if found.is_null() { None } else { Some(unsafe { CStr::from_ptr(found) }) }
}
```

Callers can't misuse `find`: the types guarantee the strings are valid, a null result becomes `None`, and the lifetime `'a` says the result points *into* `haystack`.

### 2. C strings are different from Rust strings

| | Rust `&str` | C string (`&CStr`, `*const c_char`) |
|---|---|---|
| Knows its length | yes: stored next to the pointer | no: you search for the `\0` at the end |
| May contain `\0` | yes | no: a `\0` *ends* the string |
| Encoding | always valid UTF-8 | any bytes |

**`c"Hello"`** is a **C string literal**: Rust adds the terminating `\0` for you, and its type is `&CStr`. The old way, `b"Hello\0"` plus a cast, makes it easy to forget the `\0`, and then C reads past the end of the string.

### 3. Use C's types, not guesses

C's `int`, `char` and `size_t` don't have fixed sizes in the C standard. They depend on the platform. Rust provides exact matches in `core::ffi`: `c_int`, `c_char`, `c_void`. For example, **`c_char` is signed (`i8`) on x86 and macOS, but unsigned (`u8`) on ARM Linux**. Code that hard-codes `i8` doesn't compile on a Raspberry Pi. This example uses `c_char` and `.cast()` throughout. (The previous version of this example hard-coded `i8`, and also declared `main` with `isize` where C uses a 32-bit `int`.)

### 4. Callbacks: C calling Rust

```rust
extern "C" fn compare_ints(a: *const c_void, b: *const c_void) -> c_int { … }

libc::qsort(ptr, len, size_of::<i32>(), Some(compare_ints));
```

`extern "C"` makes a Rust function use C's calling convention, so C code, here `qsort`, can call it. C gives it untyped pointers (`*const c_void`), and the Rust side casts them back to the type it knows they are.

### 5. Variadic functions: `printf` can't be checked

`printf(format, ...)` takes **any number of arguments of any type**. The format string tells C what to expect, but the Rust compiler can't verify it. Each `%` code must match its argument's C type exactly: `%d` for `c_int`, `%zu` for `usize` (`size_t`), `%lld` for `i64`, `%s` for a C string. A mismatch is undefined behaviour, not a compile error. This is one of the things that makes C dangerous and Rust's `format!` safe.

Rust can also **define** a variadic function, for example to provide one that existing C code expects to call:

```rust
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sum_ints(count: c_int, mut numbers: ...) -> c_int {
    let mut total = 0;
    for _ in 0..count {
        total += unsafe { numbers.next_arg::<c_int>() };
    }
    total
}
```

`...` must be the last parameter. Inside the function it's a `core::ffi::VaList`, C's `va_list`, and each `next_arg::<T>()` reads the next argument **as type `T`**. Nothing records how many arguments were passed or what their types are, so the function needs another way to know, here the `count` parameter, just like `printf` uses its format string. That's why `next_arg` is `unsafe`, and why calling `sum_ints` is `unsafe` too: the caller promises that `count` matches the arguments.

One C rule matters here: in a variadic call, small integer types are **promoted** to `int`, and `float` to `double`. So read a `c_int` even if the caller passed a `char` or `short`, and a `c_double` for a `float`.

### 6. What a `no_std` binary must provide itself

```rust
#![no_std]      // only core
#![no_main]     // no Rust startup code

#[unsafe(no_mangle)]
pub extern "C" fn main(argc: c_int, argv: *const *const c_char) -> c_int { … }   // C's entry point

#[panic_handler]
fn on_panic(_info: &core::panic::PanicInfo) -> ! { unsafe { libc::abort() } }

#[unsafe(no_mangle)]
pub extern "C" fn rust_eh_personality() {}
```

- **An entry point.** `#![no_main]` drops Rust's normal startup, so we export a function literally named `main`, with C's signature. `#[unsafe(no_mangle)]` stops Rust from renaming it, so the C runtime can find it.
- **A panic handler.** `core` doesn't know how to report a panic without an operating system. On a desktop we abort the process. On a microcontroller you might blink an LED or reset the chip.
- **`panic = "abort"`** in `Cargo.toml`. Unwinding a panic needs support from `std`.
- **`rust_eh_personality`.** Rust's prebuilt `core` still refers to this unwinding helper, so the linker demands it even though, with `panic = "abort"`, it's never called. Without it, linking fails with `Undefined symbols … "_rust_eh_personality"`. An empty function is the usual fix on stable Rust.

### 7. Tests run with `std`

```rust
#![cfg_attr(not(test), no_std)]
```

`no_std` only applies to the real program. `cargo test` builds with `std`, so the safe wrappers are tested like any normal Rust code.

## Run it

```bash
cargo run              # the no_std program
cargo run --release    # the small, optimised build
cargo test             # tests run with std
```

This crate is kept **outside** the repository's Cargo workspace, because it needs its own `panic = "abort"` profile, and Cargo ignores profiles set in workspace members. Run it from this folder.

Next: [Lesson 2: A core-only library](../02-core-only-library/) · Back to the [course overview](../)

## Learn more

- [The Embedded Rust Book](https://docs.rust-embedded.org/book/): `no_std` on real microcontrollers.
- [The Rustonomicon, FFI chapter](https://doc.rust-lang.org/nomicon/ffi.html): calling C safely.
- The repository's [`raw_pointers`](../../raw_pointers/) example, and the ownership course on [lifetimes](../../ownership-and-borrowing/05-lifetimes/), which explains the `'a` in `find`.
