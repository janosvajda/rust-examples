# 🧠 Mini — A Tiny LLVM-Based Programming Language (in Rust)

**Mini** is a small, educational compiler and programming language written in **Rust**.  
It compiles your source code into **native executables** using **LLVM** via the Inkwell bindings.

---

## ✨ Features

- Integer and string variables (`let`)
- `print` for variables
- Integer **expressions with precedence** (`* /` over `+ -`), parentheses, and unary `-`
- Cross-platform native binaries (macOS, Linux, Windows)
- Clean modular code: `ast`, `parser`, `codegen`, `link`, `main`

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

If you’re building the compiler itself on macOS and use Inkwell/LLVM 16 from Homebrew:
```
brew install llvm
export LLVM_SYS_160_PREFIX="/opt/homebrew/opt/llvm"
```

---

## 🧱 Architecture

| Module       | Purpose                                      |
|--------------|----------------------------------------------|
| `ast.rs`     | Abstract syntax tree (statements, expressions) |
| `parser.rs`  | Line parser + Pratt expression parser         |
| `codegen.rs` | LLVM IR generation via Inkwell                |
| `link.rs`    | OS-specific linking to produce executables    |
| `main.rs`    | CLI wiring: parse → codegen → link            |
| `examples/`  | Sample programs                               |

---

## 🧭 Evolution (Changelog-style)

- **v0.1** — Minimal language: `let` for int/string, `print` variables, IR → run with `lli`.
- **v0.2** — Proper native **linking** (no `lli`): object via LLVM TargetMachine, linked with system linker.
- **v0.3** — **Refactor** into modules (`ast`, `parser`, `codegen`, `link`, `main`).
- **v0.4** — Added **integer expressions**: `+ - * /`, parentheses, unary minus; variable reads in expressions.

---

## 🚧 Roadmap

- `print "literal";` (print string literals directly)
- `let y = x;` (assign from variables)
- `if / else` (conditional blocks)
- `while` loops
- Functions (`fn`, calls, parameters)
- Simple types beyond int/string (arrays/structs)

---

## 📚 Docs

See `docs/` for deeper write-ups:

- `docs/01_intro.md` — background & goals  
- `docs/02_parser.md` — parsing and expressions  
- `docs/03_codegen.md` — LLVM IR generation with Inkwell  
- `docs/04_linking.md` — producing native binaries  
- `docs/05_expressions.md` — Pratt parser details  
- `docs/06_future.md` — next steps and ideas

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
