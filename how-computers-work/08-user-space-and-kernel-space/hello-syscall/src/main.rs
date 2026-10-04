// "Hello, world!" through an explicit OS write request.
// Linux and macOS use inline assembly; Windows uses its documented API.
// This is still a normal Rust binary with standard-library startup.
//
// Linux's syscall numbers are a documented userspace interface.
// macOS normally uses libSystem; its raw interface is an experiment here.
// Windows programs use the system API rather than hard-coded syscall numbers.

use std::io;

const MESSAGE: &[u8] = b"Hello, world! (written by a system call)\n";

// Linux returns a negative error number; macOS returns a positive one and
// sets the carry flag. Their callers supply the matching error indication.
#[cfg(any(target_os = "linux", target_os = "macos"))]
fn syscall_result(result: isize, failed: bool) -> io::Result<usize> {
    if failed {
        Err(io::Error::from_raw_os_error(result.unsigned_abs() as i32))
    } else {
        Ok(result as usize)
    }
}

// ---- Linux, x86-64: write = 1, exit = 60 -----------------------------------
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
mod kernel {
    use core::arch::asm;
    use std::io;

    pub fn write_stdout(bytes: &[u8]) -> io::Result<usize> {
        write_fd(1, bytes)
    }

    pub fn write_fd(fd: isize, bytes: &[u8]) -> io::Result<usize> {
        let result: isize;
        // SAFETY: write only reads the valid slice. rcx and r11 are clobbered.
        unsafe {
            asm!("syscall", inlateout("rax") 1isize => result,
                 in("rdi") fd, in("rsi") bytes.as_ptr(), in("rdx") bytes.len(),
                 out("rcx") _, out("r11") _, options(nostack));
        }
        super::syscall_result(result, result < 0)
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: raw exit ends the calling thread and never returns.
        // This demonstration creates no extra threads.
        unsafe { asm!("syscall", in("rax") 60, in("rdi") code, options(noreturn, nostack)) }
    }
}

// ---- Linux, AArch64: write = 64, exit = 93; number in x8 --------------------
#[cfg(all(target_os = "linux", target_arch = "aarch64"))]
mod kernel {
    use core::arch::asm;
    use std::io;

    pub fn write_stdout(bytes: &[u8]) -> io::Result<usize> {
        write_fd(1, bytes)
    }

    pub fn write_fd(fd: isize, bytes: &[u8]) -> io::Result<usize> {
        let result: isize;
        // SAFETY: write only reads the valid slice; x0 holds its return value.
        unsafe {
            asm!("svc #0", in("x8") 64, inlateout("x0") fd => result,
                 in("x1") bytes.as_ptr(), in("x2") bytes.len(), options(nostack));
        }
        super::syscall_result(result, result < 0)
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: raw exit ends the calling thread and never returns.
        unsafe { asm!("svc #0", in("x8") 93, in("x0") code, options(noreturn, nostack)) }
    }
}

// ---- macOS, AArch64: write = 4, exit = 1; number in x16 ---------------------
#[cfg(all(target_os = "macos", target_arch = "aarch64"))]
mod kernel {
    use core::arch::asm;
    use std::io;

    pub fn write_stdout(bytes: &[u8]) -> io::Result<usize> {
        write_fd(1, bytes)
    }

    pub fn write_fd(fd: isize, bytes: &[u8]) -> io::Result<usize> {
        let result: isize;
        let failed: u32;
        // SAFETY: write reads the valid slice. Darwin changes x0 and x1;
        // both are declared as outputs. Capture carry before any other code
        // can change the flags: set means result is an error number.
        unsafe {
            asm!("svc #0x80", "cset {failed:w}, cs",
                 in("x16") 4, inlateout("x0") fd => result,
                 inlateout("x1") bytes.as_ptr() => _, in("x2") bytes.len(),
                 failed = lateout(reg) failed, options(nostack));
        }
        super::syscall_result(result, failed != 0)
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: Darwin's exit ends the process and never returns.
        unsafe { asm!("svc #0x80", in("x16") 1, in("x0") code, options(noreturn, nostack)) }
    }
}

// ---- macOS, x86-64: BSD service numbers have the 0x2000000 prefix ----------
#[cfg(all(target_os = "macos", target_arch = "x86_64"))]
mod kernel {
    use core::arch::asm;
    use std::io;

    pub fn write_stdout(bytes: &[u8]) -> io::Result<usize> {
        write_fd(1, bytes)
    }

    pub fn write_fd(fd: isize, bytes: &[u8]) -> io::Result<usize> {
        let result: isize;
        let failed: u8;
        // SAFETY: write reads the valid slice. Darwin may change rdx as a
        // second return register; syscall also overwrites rcx and r11.
        // Capture carry immediately to distinguish errors from byte counts.
        unsafe {
            asm!("syscall", "setc {failed}",
                 inlateout("rax") 0x2000004isize => result,
                 in("rdi") fd, in("rsi") bytes.as_ptr(),
                 inlateout("rdx") bytes.len() => _,
                 out("rcx") _, out("r11") _, failed = lateout(reg_byte) failed,
                 options(nostack));
        }
        super::syscall_result(result, failed != 0)
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: Darwin's exit ends the process and never returns.
        unsafe { asm!("syscall", in("rax") 0x2000001, in("rdi") code, options(noreturn, nostack)) }
    }
}

// ---- Windows: use kernel32's documented API -------------------------------
#[cfg(windows)]
mod kernel {
    use core::ffi::c_void;
    use std::io;

    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetStdHandle(which: i32) -> *mut c_void;
        fn WriteFile(
            file: *mut c_void,
            buffer: *const u8,
            length: u32,
            written: *mut u32,
            overlapped: *mut c_void,
        ) -> i32;
        fn ExitProcess(code: u32) -> !;
    }

    pub fn write_stdout(bytes: &[u8]) -> io::Result<usize> {
        const STD_OUTPUT_HANDLE: i32 = -11;
        let mut written = 0u32;
        // WriteFile's length is u32. A longer slice is sent in chunks by
        // write_all_with; never truncate its length by a narrowing cast.
        let length = bytes.len().min(u32::MAX as usize) as u32;
        // SAFETY: WriteFile reads at most length bytes from the live slice
        // and writes its count into valid storage. A null overlapped pointer
        // requests synchronous I/O on this ordinary output handle.
        let ok = unsafe {
            WriteFile(
                GetStdHandle(STD_OUTPUT_HANDLE),
                bytes.as_ptr(),
                length,
                &mut written,
                core::ptr::null_mut(),
            )
        };
        if ok != 0 {
            Ok(written as usize)
        } else {
            Err(io::Error::last_os_error())
        }
    }

    pub fn exit(code: i32) -> ! {
        // SAFETY: ExitProcess ends the process and never returns.
        unsafe { ExitProcess(code as u32) }
    }
}

#[cfg(not(any(
    all(
        any(target_os = "linux", target_os = "macos"),
        any(target_arch = "x86_64", target_arch = "aarch64")
    ),
    windows
)))]
compile_error!("hello-syscall supports Linux and macOS on x86-64 and AArch64, and Windows");

/// Finish partial writes, retry interrupted requests, and reject no progress.
fn write_all_with(
    mut bytes: &[u8],
    mut write: impl FnMut(&[u8]) -> io::Result<usize>,
) -> io::Result<()> {
    while !bytes.is_empty() {
        match write(bytes) {
            Ok(0) => return Err(io::ErrorKind::WriteZero.into()),
            Ok(count) if count <= bytes.len() => bytes = &bytes[count..],
            Ok(_) => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "writer returned an impossible byte count",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn main() {
    let result = write_all_with(MESSAGE, kernel::write_stdout);
    kernel::exit(if result.is_ok() { 0 } else { 1 });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partial_and_interrupted_writes_finish_the_message() {
        let mut received = Vec::new();
        let mut interrupt = true;
        write_all_with(MESSAGE, |remaining| {
            if interrupt {
                interrupt = false;
                return Err(io::ErrorKind::Interrupted.into());
            }
            let count = remaining.len().min(3);
            received.extend_from_slice(&remaining[..count]);
            Ok(count)
        })
        .unwrap();
        assert_eq!(received, MESSAGE);
    }

    #[test]
    fn zero_progress_is_an_error() {
        let error = write_all_with(MESSAGE, |_| Ok(0)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::WriteZero);
    }

    #[test]
    fn failures_are_returned_to_the_caller() {
        let error =
            write_all_with(MESSAGE, |_| Err(io::ErrorKind::PermissionDenied.into())).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::PermissionDenied);
    }

    #[test]
    fn an_empty_message_needs_no_write() {
        write_all_with(b"", |_| panic!("no bytes to write")).unwrap();
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn error_numbers_are_not_successful_byte_counts() {
        assert_eq!(syscall_result(41, false).unwrap(), 41);
        assert_eq!(
            syscall_result(41, true).unwrap_err().raw_os_error(),
            Some(41)
        );
        assert_eq!(
            syscall_result(-9, true).unwrap_err().raw_os_error(),
            Some(9)
        );
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn the_os_reports_an_invalid_descriptor_as_an_error() {
        assert_eq!(
            kernel::write_fd(-1, MESSAGE).unwrap_err().raw_os_error(),
            Some(9)
        ); // EBADF
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn an_empty_os_write_returns_zero() {
        assert_eq!(kernel::write_stdout(b"").unwrap(), 0);
    }
}
