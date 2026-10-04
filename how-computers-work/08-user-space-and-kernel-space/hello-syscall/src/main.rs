// "Hello, world!" by asking the kernel directly, with system calls written in
// assembly, without the standard library's println!.
//
// A system call is an instruction that switches the processor from user mode
// into the kernel: `syscall` on x86-64, `svc` on ARM. The program puts a
// number into an agreed register to say WHICH service it wants (write, exit…),
// and the arguments into others.
//
// Each operating system has its own numbers and rules:
//   Linux    the system call numbers are a stable, documented interface
//   macOS    they work, but Apple only supports calling the kernel through its
//            system library (libSystem); the numbers may change
//   Windows  the numbers change between Windows versions, so programs must go
//            through the system's API (kernel32); see the Windows part below

const MESSAGE: &[u8] = b"Hello, world! (written by a system call)\n";

// ---- Linux, x86-64: write = 1, exit = 60 --------------------------------------------------
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod kernel {
    use core::arch::asm;

    pub fn write_stdout(bytes: &[u8]) -> isize {
        let result: isize;
        // SAFETY: write(1, bytes, len) only reads `bytes`, which is valid for `len` bytes.
        // The `syscall` instruction overwrites rcx and r11, so they're declared as clobbered.
        unsafe {
            asm!("syscall", inlateout("rax") 1isize => result, in("rdi") 1, in("rsi") bytes.as_ptr(),
                 in("rdx") bytes.len(), out("rcx") _, out("r11") _, options(nostack));
        }
        result
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: exit ends the process; it never returns.
        unsafe { asm!("syscall", in("rax") 60, in("rdi") code, options(noreturn, nostack)) }
    }
}

// ---- Linux, AArch64: write = 64, exit = 93; the number goes in x8 -------------------------
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod kernel {
    use core::arch::asm;

    pub fn write_stdout(bytes: &[u8]) -> isize {
        let result: isize;
        // SAFETY: write(1, bytes, len) only reads `bytes`.
        unsafe {
            asm!("svc #0", in("x8") 64, inlateout("x0") 1isize => result, in("x1") bytes.as_ptr(),
                 in("x2") bytes.len(), options(nostack));
        }
        result
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: exit ends the process.
        unsafe { asm!("svc #0", in("x8") 93, in("x0") code, options(noreturn, nostack)) }
    }
}

// ---- macOS, AArch64: write = 4, exit = 1; the number goes in x16 --------------------------
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod kernel {
    use core::arch::asm;

    pub fn write_stdout(bytes: &[u8]) -> isize {
        let result: isize;
        // SAFETY: write(1, bytes, len) only reads `bytes`.
        unsafe {
            asm!("svc #0x80", in("x16") 4, inlateout("x0") 1isize => result, in("x1") bytes.as_ptr(),
                 in("x2") bytes.len(), options(nostack));
        }
        result
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: exit ends the process.
        unsafe { asm!("svc #0x80", in("x16") 1, in("x0") code, options(noreturn, nostack)) }
    }
}

// ---- macOS, x86-64: write = 0x2000004, exit = 0x2000001 -----------------------------------
// (0x2000000 marks the BSD part of the macOS kernel.)
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod kernel {
    use core::arch::asm;

    pub fn write_stdout(bytes: &[u8]) -> isize {
        let result: isize;
        // SAFETY: write(1, bytes, len) only reads `bytes`. `syscall` overwrites rcx and r11.
        unsafe {
            asm!("syscall", inlateout("rax") 0x2000004isize => result, in("rdi") 1, in("rsi") bytes.as_ptr(),
                 in("rdx") bytes.len(), out("rcx") _, out("r11") _, options(nostack));
        }
        result
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: exit ends the process.
        unsafe { asm!("syscall", in("rax") 0x2000001, in("rdi") code, options(noreturn, nostack)) }
    }
}

// ---- Windows: no public system call numbers, so ask kernel32 --------------------------------
#[cfg(windows)]
mod kernel {
    use core::ffi::c_void;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(which: i32) -> *mut c_void;
        fn WriteFile(file: *mut c_void, buffer: *const u8, length: u32, written: *mut u32, overlapped: *mut c_void) -> i32;
        fn ExitProcess(code: u32) -> !;
    }

    pub fn write_stdout(bytes: &[u8]) -> isize {
        const STD_OUTPUT_HANDLE: i32 = -11;
        let mut written = 0u32;
        // SAFETY: GetStdHandle has no preconditions; WriteFile only reads `bytes`
        // and writes the count into `written`, which lives until it returns.
        let ok = unsafe {
            WriteFile(GetStdHandle(STD_OUTPUT_HANDLE), bytes.as_ptr(), bytes.len() as u32, &mut written, core::ptr::null_mut())
        };
        if ok != 0 { written as isize } else { -1 }
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: ExitProcess ends the process.
        unsafe { ExitProcess(code as u32) }
    }
}

#[cfg(not(any(
    all(any(target_os = "linux", target_os = "macos"), any(target_arch = "x86_64", target_arch = "aarch64")),
    windows
)))]
compile_error!("hello-syscall supports Linux and macOS on x86-64 and AArch64, and Windows");

fn main() {
    let written = kernel::write_stdout(MESSAGE);
    // A system call's answer is just a number: here, how many bytes were written.
    kernel::exit(if written == MESSAGE.len() as isize { 0 } else { 1 });
}
