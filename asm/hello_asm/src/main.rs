// src/main.rs
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

// Helpful compile-time message if you're on an unsupported target for this file.
#[cfg(not(any(
    all(target_os = "macos", target_arch = "x86_64"),
    all(target_os = "windows", target_arch = "x86_64")
)))]
compile_error!("This example includes inline asm for macOS x86_64 and Windows x86_64. If you're on Apple Silicon (arm64), tell me and I'll add that variant, or build the x86_64 mac target and run via Rosetta.");
