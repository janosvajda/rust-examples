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

---

## 2. Create & Setup the Project

```bash
cargo new hello_asm
cd hello_asm
```

Replace the contents of `src/main.rs` with the inline assembly examples below.

---

## 3. Source Code (`src/main.rs`)

This file has two inline-assembly `main` functions: one for **macOS x86_64** and one for **Windows x86_64**.
Rust will compile the right one for your platform.

```rust
#![allow(non_snake_case)]
use core::arch::asm;

//
// ---------- macOS x86_64 (Darwin) ----------
// Uses syscalls: write (0x2000004) and exit (0x2000001).
//
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() {
    let msg = b"Hello, world!\n";
    let ptr = msg.as_ptr();
    let len = msg.len();

    unsafe {
        // write(1, msg, len)
        asm!(
            "mov rax, 0x2000004",     // write
            "mov rdi, 1",             // fd = stdout
            "syscall",
            in("rsi") ptr,            // buf
            in("rdx") len,            // count
            out("rax") _, out("rdi") _,
        );

        // exit(0)
        asm!(
            "mov rax, 0x2000001",     // exit
            "xor rdi, rdi",           // code = 0
            "syscall",
            options(noreturn)
        );
    }
}

//
// ---------- Windows x86_64 (MSVC/GNU) ----------
// Calls: GetStdHandle, WriteFile, ExitProcess via inline asm.
// Microsoft x64 ABI: RCX,RDX,R8,R9 + 32-byte shadow space.
//
#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
#[link(name = "kernel32")]
extern "system" {
    fn GetStdHandle(nStdHandle: i32) -> *mut core::ffi::c_void;
    fn WriteFile(
        hFile: *mut core::ffi::c_void,
        lpBuffer: *const u8,
        nNumberOfBytesToWrite: u32,
        lpNumberOfBytesWritten: *mut u32,
        lpOverlapped: *mut core::ffi::c_void,
    ) -> i32;
    fn ExitProcess(uExitCode: u32) -> !;
}

#[cfg(all(target_os = "windows", target_arch = "x86_64"))]
fn main() {
    const STD_OUTPUT_HANDLE: i32 = -11;
    let msg = b"Hello, world!\r\n";
    let mut written: u32 = 0;
    let mut h_stdout: *mut core::ffi::c_void;

    unsafe {
        // h_stdout = GetStdHandle(STD_OUTPUT_HANDLE)
        asm!(
            "mov ecx, {std}",
            "sub rsp, 32",        // shadow space
            "call {GetStdHandle}",
            "add rsp, 32",
            "mov {hout}, rax",
            std = const STD_OUTPUT_HANDLE,
            GetStdHandle = sym GetStdHandle,
            hout = lateout(reg) h_stdout,
            out("rcx") _, out("rax") _,
        );

        // WriteFile(h_stdout, msg, msg.len(), &mut written, null)
        asm!(
            "mov rcx, {h}",
            "mov rdx, {buf}",
            "mov r8,  {len}",
            "mov r9,  {pwr}",
            "sub rsp, 32",        // shadow space
            "call {WriteFile}",
            "add rsp, 32",
            h   = in(reg) h_stdout,
            buf = in(reg) msg.as_ptr(),
            len = in(reg) (msg.len() as u32),
            pwr = in(reg) (&mut written as *mut u32 as *mut core::ffi::c_void),
            WriteFile = sym WriteFile,
            out("rax") _, out("rcx") _, out("rdx") _, out("r8") _, out("r9") _,
        );

        // ExitProcess(0)
        asm!(
            "xor ecx, ecx",
            "sub rsp, 32",
            "call {ExitProcess}",
            options(noreturn),
            ExitProcess = sym ExitProcess
        );
    }
}

// Helpful compile-time message if you're on an unsupported target.
#[cfg(not(any(
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
compile_error!("This example includes inline asm for macOS x86_64 and Windows x86_64. On Apple Silicon (arm64), either build the x86_64 mac target and run via Rosetta or add a native AArch64 asm block.");
```

---

## 4. Build & Run

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

## 5. Notes for Apple Silicon (M1/M2/M3)

The inline assembly above targets **x86_64 macOS**. On Apple Silicon you can run it via **Rosetta**:

```bash
softwareupdate --install-rosetta --agree-to-license   # once, if needed
rustup target add x86_64-apple-darwin
cargo run --target x86_64-apple-darwin
```

If you want a **native arm64** inline-asm variant, add an AArch64 block or ask for an example.

---

## 6. Inspect Compiler-Generated Assembly

If you also want to see what Rust/LLVM emits for this project:

```bash
cargo rustc --release -- --emit=asm
```
Assembly files will be under:
```
target/release/deps/
```

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

---

## 8. License

MIT
