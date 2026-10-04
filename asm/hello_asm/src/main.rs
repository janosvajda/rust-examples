// src/main.rs
#![allow(non_snake_case)]
use core::arch::asm;

//
// ---------- A naked function: a whole function written in assembly ----------
// A normal function gets a prologue and epilogue from the compiler (saving
// registers, setting up the stack). A naked function gets NOTHING: its body is
// exactly the assembly you write, so it must follow the calling convention
// itself, including the final `ret`. `extern "C"` says which convention:
// where the arguments arrive and where the result must go.
//
// cfg_select! picks the first branch whose condition is true for the target
// being compiled, like a `match` on the platform. Order matters: the Windows
// branch must come before the general x86_64 one.
//
cfg_select! {
    target_arch = "aarch64" => {
        /// AArch64: arguments arrive in x0 and x1, the result goes in x0.
        #[unsafe(naked)]
        extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
            core::arch::naked_asm!(
                "add x0, x0, x1",
                "ret",
            )
        }
    }
    all(target_arch = "x86_64", target_os = "windows") => {
        /// Windows x64: arguments arrive in rcx and rdx, the result goes in rax.
        #[unsafe(naked)]
        extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
            core::arch::naked_asm!(
                "lea rax, [rcx + rdx]",
                "ret",
            )
        }
    }
    target_arch = "x86_64" => {
        /// x86_64 on macOS and Linux (System V): arguments in rdi and rsi, result in rax.
        #[unsafe(naked)]
        extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
            core::arch::naked_asm!(
                "lea rax, [rdi + rsi]",
                "ret",
            )
        }
    }
    _ => {
        compile_error!("add_numbers needs assembly for this processor");
    }
}

/// Called at the start of every `main` below, before the assembly "Hello, world!".
fn naked_function_demo() {
    println!("A naked function says: 40 + 2 = {}", add_numbers(40, 2));
}

//
// ---------- macOS x86_64 (Darwin) ----------
// Uses syscalls: write (0x2000004) and exit (0x2000001).
//
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
fn main() {
    naked_function_demo();
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
// ---------- macOS aarch64 (Apple Silicon) ----------
// Uses syscalls: write (4) and exit (1).
// The syscall number goes in x16, arguments in x0..x2, and `svc #0x80` traps into the kernel.
//
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
fn main() {
    naked_function_demo();
    let msg = b"Hello, world!\n";
    let ptr = msg.as_ptr();
    let len = msg.len();

    unsafe {
        // write(1, msg, len)
        asm!(
            "mov x16, #4",            // write
            "mov x0, #1",             // fd = stdout
            "svc #0x80",
            in("x1") ptr,             // buf
            in("x2") len,             // count
            out("x0") _, out("x16") _,
        );

        // exit(0)
        asm!(
            "mov x16, #1",            // exit
            "mov x0, #0",             // code = 0
            "svc #0x80",
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
unsafe extern "system" {
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
    naked_function_demo();
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
            // nNumberOfBytesToWrite is a 32-bit DWORD: use the 32-bit register
            // names. Writing r8d also clears the upper half of r8.
            "mov r8d, {len:e}",
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

// Helpful compile-time message if you're on an unsupported target for this file.
#[cfg(not(any(
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "macos", target_arch = "aarch64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
compile_error!("This example only includes inline asm for macOS x86_64, macOS aarch64 (Apple Silicon) and Windows x86_64.");
