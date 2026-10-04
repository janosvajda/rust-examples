<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Hello Assembly in Rust

This project shows how to print **“Hello, world!”** using **inline assembly** (`asm!`) inside Rust.
It includes working examples for:

- **macOS x86_64** (Darwin syscalls)
- **macOS aarch64 / Apple Silicon** (Darwin syscalls via `svc #0x80`)
- **Windows x86_64** (WinAPI calls from asm)

---

## 1. Prerequisites

### macOS
1. Install Xcode command line tools:
   ```bash
   xcode-select --install
   ```
2. Install Rust (via rustup):
   ```bash
   curl https://sh.rustup.rs -sSf | sh
   source "$HOME/.cargo/env"
   ```

### Windows
1. Install **MSVC build tools** (Visual Studio “Desktop development with C++” or Build Tools).
2. Install Rust (MSVC toolchain recommended):
   - Download installer: https://rustup.rs
   - Or after installation:
     ```powershell
     rustup default stable-x86_64-pc-windows-msvc
     ```

### Verify install
```bash
rustc --version
cargo --version
```

## 2. Build & Run

### macOS (Apple Silicon, native)
```bash
cargo run
```

### macOS (Intel x64)
```bash
rustup target add x86_64-apple-darwin
cargo run --target x86_64-apple-darwin
```

### Windows (x64, MSVC)
```powershell
rustup target add x86_64-pc-windows-msvc
cargo run --target x86_64-pc-windows-msvc
```

Expected output:
```
A naked function says: 40 + 2 = 42
Hello, world!
```

---

## 3. Notes for Apple Silicon (M1/M2/M3)

`cargo run` uses the native **aarch64** variant. To try the **x86_64** variant on Apple Silicon, run it via **Rosetta**:

```bash
softwareupdate --install-rosetta --agree-to-license   # once, if needed
rustup target add x86_64-apple-darwin
cargo run --target x86_64-apple-darwin
```

---

## 4. Inspect Compiler-Generated Assembly

If you also want to see what Rust/LLVM emits for this project:

```bash
cargo rustc --release -- --emit=asm
```
Assembly files will be under:
```
target/release/deps/
```

---

## 5. A whole function in assembly: naked functions

`asm!` puts a few instructions **inside** a normal Rust function. The compiler still adds its usual code around them: a **prologue** at the start (saving registers, making room on the stack) and an **epilogue** at the end. A **naked function** has none of that. Its body is exactly the assembly you write, nothing more:

```rust
#[unsafe(naked)]
extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
    core::arch::naked_asm!(
        "add x0, x0, x1",    // AArch64: a + b, result in x0
        "ret",               // return to the caller: you must write this yourself
    )
}
```

Because the compiler adds nothing, the function must follow the **calling convention** by hand: the rules for where arguments arrive and where the result goes. `extern "C"` picks the convention, and the registers differ per platform:

| Platform | `a` | `b` | result |
|---|---|---|---|
| AArch64 (Apple Silicon, ARM Linux) | `x0` | `x1` | `x0` |
| x86_64 macOS and Linux (System V) | `rdi` | `rsi` | `rax` |
| x86_64 Windows | `rcx` | `rdx` | `rax` |

From the outside, `add_numbers(40, 2)` is an ordinary, safe function call. Naked functions are used where the exact instructions matter: interrupt handlers, the first code that runs when a system boots, switching between threads in an operating system kernel, and small hand-tuned routines.

The attribute is written `#[unsafe(naked)]` because the compiler can't check the assembly. If it breaks the calling convention, for example by forgetting `ret` or changing a register the caller relies on, the program misbehaves.

## 6. Choosing code per platform: `cfg_select!`

Each platform needs different assembly. `cfg_select!` chooses between versions like a `match` on the target you're compiling for:

```rust
cfg_select! {
    target_arch = "aarch64" => {
        // the AArch64 version of add_numbers
    }
    all(target_arch = "x86_64", target_os = "windows") => {
        // the Windows x64 version
    }
    target_arch = "x86_64" => {
        // the System V version, for macOS and Linux
    }
    _ => {
        compile_error!("add_numbers needs assembly for this processor");
    }
}
```

The **first** branch whose condition is true is compiled; the others are dropped entirely. Order matters, just as in `match`: the Windows branch must come before the general `x86_64` one, or Windows would get the System V version. The `_` branch catches every other platform with a clear error message.

It does the same job as a separate `#[cfg(…)]` on each version, as the three `main` functions in this file use, but you don't have to repeat and negate the conditions yourself. `cfg_select!` also works as an expression: `let os = cfg_select! { windows => "Windows", _ => "something else" };`.

---

## 7. Troubleshooting

- **Windows linker errors (kernel32 / LNK2019/2001):**
  Ensure you installed the **MSVC** toolchain and C++ Build Tools, then:
  ```powershell
  rustup default stable-x86_64-pc-windows-msvc
  ```

- **macOS “command line tools not found”:**
  ```bash
  xcode-select --install
  ```

- **Permission denied on macOS binary:**
  ```bash
  chmod +x target/.../hello_asm
  ```
