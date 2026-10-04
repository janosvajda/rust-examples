// Lesson 5: real machine code.
//
// A function is bytes in memory, exactly like lesson 4's tiny CPU program;
// only the instruction set is a real one. This program contains a function
// written directly in assembly (a "naked" function), calls it, and then reads
// its machine code bytes straight out of memory.

cfg_select! {
    target_arch = "aarch64" => {
        /// AArch64 (Apple Silicon, many phones): arguments arrive in x0 and x1,
        /// the result goes in x0.
        #[unsafe(naked)]
        extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
            core::arch::naked_asm!(
                "add x0, x0, x1",
                "ret",
            )
        }
        /// The encodings from the Arm manual: every AArch64 instruction is
        /// 4 bytes, stored little-endian.
        const EXPECTED: &[u8] = &[
            0x00, 0x00, 0x01, 0x8b, // add x0, x0, x1   (0x8b010000)
            0xc0, 0x03, 0x5f, 0xd6, // ret              (0xd65f03c0)
        ];
        const ARCHITECTURE: &str = "AArch64 (ARM, 64-bit)";
    }
    all(target_arch = "x86_64", target_os = "windows") => {
        /// x86-64 on Windows: arguments arrive in rcx and rdx, the result goes in rax.
        #[unsafe(naked)]
        extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
            core::arch::naked_asm!(
                "lea rax, [rcx + rdx]",
                "ret",
            )
        }
        const EXPECTED: &[u8] = &[
            0x48, 0x8d, 0x04, 0x11, // lea rax, [rcx + rdx]
            0xc3,                   // ret
        ];
        const ARCHITECTURE: &str = "x86-64 (Intel/AMD, Windows)";
    }
    target_arch = "x86_64" => {
        /// x86-64 on macOS and Linux: arguments arrive in rdi and rsi, the result goes in rax.
        #[unsafe(naked)]
        extern "C" fn add_numbers(a: u64, b: u64) -> u64 {
            core::arch::naked_asm!(
                "lea rax, [rdi + rsi]",
                "ret",
            )
        }
        /// x86-64 instructions have different lengths: here 4 bytes and 1 byte.
        const EXPECTED: &[u8] = &[
            0x48, 0x8d, 0x04, 0x37, // lea rax, [rdi + rsi]
            0xc3,                   // ret
        ];
        const ARCHITECTURE: &str = "x86-64 (Intel/AMD)";
    }
    _ => {
        compile_error!("this lesson has machine code for AArch64 and x86-64 only");
    }
}

/// Copies instruction bytes on targets where function pointers can be used
/// to read code memory. This is a platform-specific experiment.
///
/// # Safety
/// The caller must ensure that converting this function pointer to a data
/// pointer is valid on the target and that the pointer and length satisfy
/// std::slice::from_raw_parts: initialized, readable bytes within one
/// allocation, unchanged during the copy. The length must not exceed
/// isize::MAX or make the address calculation wrap.
unsafe fn machine_code_of(function: extern "C" fn(u64, u64) -> u64, length: usize) -> Vec<u8> {
    let start = function as *const u8;
    // SAFETY: the caller must uphold the pointer, readable-region, length and
    // immutability requirements documented above.
    unsafe { std::slice::from_raw_parts(start, length) }.to_vec()
}

static GREETING: &str = "hello";

fn main() {
    println!("1. Calling a function written in assembly");
    println!("    this machine: {ARCHITECTURE}");
    println!("    add_numbers(40, 2) = {}", add_numbers(40, 2));

    println!("\n2. The function is bytes in memory: here they are");
    // SAFETY: on the demonstrated desktop targets, this naked function's
    // instruction bytes are readable and remain unchanged. EXPECTED.len()
    // is exactly the length of the complete assembly body written above.
    let bytes = unsafe { machine_code_of(add_numbers, EXPECTED.len()) };
    let hex: Vec<String> = bytes.iter().map(|b| format!("{b:02x}")).collect();
    println!("    at {:p}: {}", add_numbers as *const u8, hex.join(" "));
    println!(
        "    the processor manual's encoding: {}",
        if bytes == EXPECTED {
            "matches exactly"
        } else {
            "DIFFERS"
        }
    );

    println!("\n3. Code, constants, the stack and the heap: different parts of memory");
    let local = 0u64;
    let boxed = Box::new(0u64);
    println!("    code   (add_numbers)   {:p}", add_numbers as *const u8);
    println!("    static (GREETING text) {:p}", GREETING.as_ptr());
    println!("    stack  (a local)       {:p}", &local);
    println!("    heap   (a Box)         {:p}", &*boxed);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_assembly_function_adds() {
        assert_eq!(add_numbers(40, 2), 42);
        assert_eq!(add_numbers(u64::MAX, 1), 0); // the hardware wraps around
    }

    #[test]
    fn the_bytes_in_memory_are_the_manuals_encoding() {
        // SAFETY: the same known function, byte length and readable code
        // mapping as the demonstration in main.
        assert_eq!(
            unsafe { machine_code_of(add_numbers, EXPECTED.len()) },
            EXPECTED
        );
    }
}
