# Hello Assembly in Rust

This project shows how to print **“Hello, world!”** using **inline assembly** (`asm!`) inside Rust.
It includes working examples for:

- **macOS x86_64** (Darwin syscalls)
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
Hello, world!
```

---

## 3. Notes for Apple Silicon (M1/M2/M3)

The inline assembly above targets **x86_64 macOS**. On Apple Silicon you can run it via **Rosetta**:

```bash
softwareupdate --install-rosetta --agree-to-license   # once, if needed
rustup target add x86_64-apple-darwin
cargo run --target x86_64-apple-darwin
```

If you want a **native arm64** inline-asm variant, add an AArch64 block or ask for an example.

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

## 5. Troubleshooting

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
