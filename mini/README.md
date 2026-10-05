<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# 🧠 Mini — A Tiny LLVM-Based Programming Language (in Rust)

**Mini** is a small, educational compiler and programming language written in **Rust**.  
It compiles your source code into **native executables** using **LLVM**: Mini writes LLVM IR as plain text and the LLVM installed on your machine turns it into machine code.

---

## ✨ Features

- Four types: whole numbers, **fixed-point decimals** (`0.1 + 0.2` is `0.3`), booleans and text
- Variables (`let`)
- `print` for variables
- **Expressions with precedence** (`* /` over `+ -`), parentheses, and unary `-`
- **Comparisons** (`== != < <= > >=`) and **logic** (`and`, `or`, `not`)
- Cross-platform native binaries (macOS, Linux, Windows; on Windows, decimal `*` and `/` need the `__divti3` helper from LLVM's compiler-rt, which MSVC's C library doesn't have)
- Clean modular code: `ast`, `parser`, `codegen`, `llvm`, `link`, `main`
- The full language is described in [docs/syntax.md](docs/syntax.md)

---

## 💡 Quick Example

**examples/hello.mini**

```
let name = "Mini";
let year = 2025;
print name;
print year;
```

Build & run:

```
mini examples/hello.mini ./hello
./hello
```

Output:

```
Mini
2025
```

---

## 🧩 Expressions Example

**examples/expr.mini**

```
let a = 2 + 3 * 5;
let b = (2 + 3) * 5 - 4 / 2;
let c = -a + 10;
let msg = "result:";
print a;
print b;
print c;
print msg;
```

Run:

```
mini examples/expr.mini ./expr
./expr
```

---

## 🏗️ Build

```
cargo build --release
# run locally
./target/release/mini examples/hello.mini ./hello
# or install globally
cargo install --path . --force
mini examples/hello.mini ./hello
```

Building Mini needs Rust. **Running** Mini needs LLVM 16 or newer with a backend for the host machine, and its `llc` program on your `PATH`:
```
llc --version             # should say "LLVM version 16" or higher
```
Install it with `brew install llvm` (macOS), `sudo apt install llvm` (Ubuntu) or the installer from llvm.org (Windows). If your `llc` has another name, such as `llc-18` on some Linux systems, tell Mini with `MINI_LLC=llc-18`.

Linking also needs a C toolchain. On macOS, install Xcode Command Line Tools; on Linux, install GCC or Clang with the C runtime development files, and ensure `cc` is on `PATH`. Mini invokes `cc` so the compiler driver supplies the correct startup code, runtime libraries and linker options. On Windows, run from a Visual Studio developer shell with `link.exe` and the MSVC runtime libraries available. See [Clang's explanation of its driver and linking stages](https://clang.llvm.org/docs/CommandGuide/clang.html).

### How Mini uses LLVM

```
hello.mini → parser → codegen ──► hello.ll   LLVM IR, plain text you can read
                                     │
                                    llc       the LLVM installed on your machine
                                     ▼
                                  hello.o    real machine code
                                     │
                              cc → linker (Unix), link.exe (Windows)
                                     ▼
                                   hello     the executable
```

Mini writes LLVM IR text using syntax supported by LLVM 16 and later. It checks the installed major version and invokes `llc`; backend targets, runtime helpers and linker availability must also match the platform. The `.ll` file stays next to the executable, so open it and see what your program became.

---

## 🧱 Architecture

| Module       | Purpose                                      |
|--------------|----------------------------------------------|
| `ast.rs`     | Abstract syntax tree (statements, expressions) |
| `parser.rs`  | Line parser + Pratt expression parser         |
| `codegen.rs` | Checks types and writes LLVM IR as text       |
| `runtime.ll` | Decimal `*`, `/` and printing, in LLVM IR     |
| `llvm.rs`    | Checks for LLVM 16+ and runs its `llc`        |
| `link.rs`    | OS-specific linking to produce executables    |
| `main.rs`    | CLI wiring: parse → codegen → llc → link      |
| `examples/`  | Sample programs                               |

---

## 🧭 Evolution (Changelog-style)

- **v0.1** — Minimal language: `let` for int/string, `print` variables, IR → run with `lli`.
- **v0.2** — Proper native **linking** (no `lli`): object via LLVM TargetMachine, linked with system linker.
- **v0.3** — **Refactor** into modules (`ast`, `parser`, `codegen`, `link`, `main`).
- **v0.4** — Added **integer expressions**: `+ - * /`, parentheses, unary minus; variable reads in expressions.
- **v0.5** — Works with **any installed LLVM 16+**: LLVM IR is written as text and compiled by the installed `llc`, so Mini no longer depends on an LLVM crate.
- **v0.6** — New types: **fixed-point decimals** (6 places, stored as millionths) and **booleans**, with comparisons and `and` / `or` / `not`. Unknown characters in expressions are now errors.

---

## 🚧 Roadmap

- `print "literal";` (print string literals directly)
- `if / else` (conditional blocks)
- `while` loops
- Functions (`fn`, calls, parameters)
- Compound types (arrays, structs)

---

## 📚 Docs

- [How Mini turns your program into a real program](docs/how-mini-works.md): what a compiler and a linker are, and why we need them, explained simply enough for children.
- [The Mini language](docs/syntax.md): the complete syntax, with examples, error messages and the grammar.

---

## 🧠 Why Mini?

To learn how **real compilers** work end-to-end: parse → IR → codegen → link → run.  
The codebase stays small and hackable, but the output is **real machine code**.

---

## 🧑‍💻 Author

Built with ❤️ and Rust by **Janos Vajda**

---

## 📜 License

Free

## Numeric behavior and checked failures

Whole-number values use signed 32-bit storage. The literal `-2147483648` is accepted; positive `2147483648` is rejected. Integer addition, subtraction and multiplication wrap modulo 2³². Integer division truncates toward zero and traps for zero divisors or `-2147483648 / -1`, rather than emitting LLVM's undefined `sdiv` cases.

Decimals store signed 64-bit millionths. Representable literals and addition/subtraction are exact within their range. Multiplication and division use wider intermediates and truncate toward zero to six decimal places; they do not provide unlimited precision. For example, `1.0 / 3.0` becomes `0.333333`. Decimal arithmetic wraps if the final stored integer exceeds its range, and decimal division by zero traps.

Expressions are limited to 256 tokens to bound parser/AST recursion. This is a Mini language limit, not a Rust or LLVM limit. Malformed quotes and unsupported tokens return parser errors. Output paths are passed to linkers as native platform paths, without assuming UTF-8.

`integer-runtime.ll` holds the guarded integer-division helper; `runtime.ll` holds decimal helpers. A trap terminates the generated program; Mini does not currently provide catchable runtime arithmetic errors. [LLVM division semantics](https://llvm.org/docs/LangRef.html#sdiv-instruction)
