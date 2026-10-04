// A Rust program WITHOUT the standard library (`no_std`), that uses the C
// standard library (libc) for everything the operating system provides:
// printing, memory allocation, sorting, environment variables.
//
//   #![no_std]   don't link Rust's `std`; only `core` is available
//   #![no_main]  don't use Rust's normal startup code; we provide C's `main`
//
// Both only apply to the real program. `cargo test` builds with `std`, so
// the safe wrapper functions below can be tested like normal Rust code.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

use core::ffi::{CStr, c_char, c_int, c_void};

// ============================================================================
// Safe wrappers around C functions
//
// Every call into C is `unsafe`: the compiler can't check what C code does.
// The standard pattern is to keep each `unsafe` block small, check
// everything C needs (non-null pointers, nul-terminated strings, valid
// lengths), and expose a SAFE function the rest of the program can use.
// ============================================================================

/// Finds `needle` inside `haystack` with C's `strstr`.
///
/// `&CStr` is Rust's type for "a borrowed C string": bytes ending in a nul
/// (`\0`). Taking `&CStr` instead of raw pointers means callers can't pass
/// a string without the terminating nul, which C would read past.
pub fn find<'a>(haystack: &'a CStr, needle: &CStr) -> Option<&'a CStr> {
    // SAFETY: both pointers come from valid CStrs, so they're non-null and
    // nul-terminated, which is everything strstr requires.
    let found = unsafe { libc::strstr(haystack.as_ptr(), needle.as_ptr()) };
    if found.is_null() {
        None
    } else {
        // SAFETY: strstr returns a pointer INTO `haystack`, so it points to
        // a nul-terminated string that lives exactly as long as `haystack`.
        Some(unsafe { CStr::from_ptr(found) })
    }
}

/// The length of a C string, counted by C's `strlen`: it walks the bytes
/// until it finds the nul. (Rust's `CStr::count_bytes` does the same, and in
/// real code you'd use that. Clippy says so too; here the point is calling C.)
#[allow(clippy::strlen_on_c_strings)]
pub fn c_length(text: &CStr) -> usize {
    // SAFETY: a CStr is always nul-terminated.
    unsafe { libc::strlen(text.as_ptr()) }
}

/// The comparison function C's `qsort` calls. It must use the C calling
/// convention (`extern "C"`), because C code calls it, and it receives
/// untyped pointers (`*const c_void`) to two elements.
extern "C" fn compare_ints(a: *const c_void, b: *const c_void) -> c_int {
    // SAFETY: qsort only ever passes pointers to elements of the slice we
    // gave it, and that slice holds i32s.
    let (a, b) = unsafe { (*a.cast::<i32>(), *b.cast::<i32>()) };
    a.cmp(&b) as c_int // Less → -1, Equal → 0, Greater → 1
}

/// Sorts numbers with C's `qsort`, passing a Rust function as the callback.
pub fn sort_with_qsort(numbers: &mut [i32]) {
    // SAFETY: the pointer, element count and element size all describe the
    // same valid, writable slice, and compare_ints matches qsort's expected
    // callback signature.
    unsafe {
        libc::qsort(
            numbers.as_mut_ptr().cast::<c_void>(),
            numbers.len(),
            size_of::<i32>(),
            Some(compare_ints),
        );
    }
}

/// Allocates room for `count` numbers with C's `malloc`, fills it with
/// 1, 2, …, count, sums it, and frees it with `free`.
///
/// Without `std` (and without the `alloc` crate) there's no `Vec` or `Box`:
/// heap memory is managed by hand, exactly as in C. Forgetting `free` leaks
/// memory; using the buffer after `free` is undefined behaviour.
pub fn sum_of_malloc_buffer(count: usize) -> Option<i64> {
    // SAFETY: malloc may return null, which we check before using it.
    let buffer = unsafe { libc::malloc(count * size_of::<i32>()) }.cast::<i32>();
    if buffer.is_null() {
        return None;
    }
    // SAFETY: the buffer is non-null and big enough for `count` i32s, and
    // nothing else uses it while this slice exists.
    let numbers = unsafe { core::slice::from_raw_parts_mut(buffer, count) };
    for (i, slot) in numbers.iter_mut().enumerate() {
        *slot = i as i32 + 1;
    }
    let total = numbers.iter().map(|&n| n as i64).sum();
    // SAFETY: allocated by malloc above, freed exactly once, and `numbers`
    // isn't used after this line.
    unsafe { libc::free(buffer.cast()) };
    Some(total)
}

/// Writes a number into `buffer` as text with C's `snprintf`, and returns
/// the result as a C string. A fixed-size buffer on the stack replaces
/// `format!`, which needs a heap.
pub fn format_temperature(buffer: &mut [u8; 32], celsius: c_int) -> &CStr {
    // SAFETY: snprintf writes at most buffer.len() bytes including the nul,
    // and the %d format matches the c_int argument.
    unsafe {
        libc::snprintf(
            buffer.as_mut_ptr().cast::<c_char>(),
            buffer.len(),
            c"%d degrees".as_ptr(),
            celsius,
        );
    }
    // snprintf always nul-terminates within the buffer, so this can't fail.
    CStr::from_bytes_until_nul(buffer).unwrap_or(c"")
}

/// Reads an environment variable with C's `getenv`. Returns its length,
/// because the pointer getenv returns may be invalidated if the environment
/// changes later, so it's safer not to hand out a reference to it.
pub fn env_var_length(name: &CStr) -> Option<usize> {
    // SAFETY: `name` is a valid C string. The returned pointer is only used
    // immediately, before anything can change the environment.
    let value = unsafe { libc::getenv(name.as_ptr()) };
    if value.is_null() {
        None
    } else {
        Some(unsafe { libc::strlen(value) })
    }
}

// ---- 7. A variadic function written in Rust ----------------------------------

/// The C signature is `int sum_ints(int count, ...)`: `count` says how many
/// numbers follow. Like `printf`, nothing checks what the caller passes, so
/// the function is `unsafe` to call: `count` must match the number of `int`s
/// that follow. `#[unsafe(no_mangle)]` keeps the name, so C code can call it too.
///
/// # Safety
/// The caller must pass exactly `count` further arguments, each a C `int`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn sum_ints(count: c_int, mut numbers: ...) -> c_int {
    let mut total = 0;
    for _ in 0..count {
        // SAFETY: the caller promised `count` more `int` arguments.
        total += unsafe { numbers.next_arg::<c_int>() };
    }
    total
}

// ============================================================================
// The program itself: only compiled for the real no_std build
// ============================================================================

#[cfg(not(test))]
mod program {
    use super::*;

    /// With `#![no_main]`, there's no Rust `fn main`. Instead we export a
    /// function named exactly `main` with C's signature,
    /// `int main(int argc, char **argv)`, and the C runtime calls it.
    /// `#[unsafe(no_mangle)]` keeps the name `main` unchanged in the binary.
    #[unsafe(no_mangle)]
    pub extern "C" fn main(argc: c_int, _argv: *const *const c_char) -> c_int {
        // printf is a C *variadic* function: the format string decides how
        // many arguments it reads, and the compiler can't check them. Each
        // %-code must match its argument's C type exactly.
        unsafe {
            libc::printf(c"no_std Rust, talking directly to the C library\n\n".as_ptr());

            // 1. Searching a string
            let haystack = c"Hello World!";
            match find(haystack, c"World") {
                Some(rest) => libc::printf(c"1. strstr: found \"%s\"\n".as_ptr(), rest.as_ptr()),
                None => libc::printf(c"1. strstr: not found\n".as_ptr()),
            };

            // 2. String length (%zu is the format code for size_t / usize)
            libc::printf(c"2. strlen(\"%s\") = %zu\n".as_ptr(), haystack.as_ptr(), c_length(haystack));

            // 3. Sorting with a Rust callback
            let mut numbers = [42, 7, 19, 3, 25];
            sort_with_qsort(&mut numbers);
            libc::printf(
                c"3. qsort: %d %d %d %d %d\n".as_ptr(),
                numbers[0],
                numbers[1],
                numbers[2],
                numbers[3],
                numbers[4],
            );

            // 4. Manual heap memory
            match sum_of_malloc_buffer(1000) {
                Some(total) => libc::printf(c"4. malloc/free: 1 + 2 + ... + 1000 = %lld\n".as_ptr(), total),
                None => libc::printf(c"4. malloc failed\n".as_ptr()),
            };

            // 5. Formatting without format!
            let mut buffer = [0u8; 32];
            let text = format_temperature(&mut buffer, 21);
            libc::printf(c"5. snprintf: \"%s\"\n".as_ptr(), text.as_ptr());

            // 6. The environment and the process
            match env_var_length(c"HOME") {
                Some(len) => libc::printf(c"6. getenv: HOME is set (%zu characters)\n".as_ptr(), len),
                None => libc::printf(c"6. getenv: HOME is not set\n".as_ptr()),
            };
            libc::printf(c"   getpid: this process is %d, started with %d argument(s)\n".as_ptr(), libc::getpid(), argc);

            // 7. A variadic function written in Rust, called like a C one
            libc::printf(c"7. variadic Rust function: sum_ints(4, 10, 20, 30, 40) = %d\n".as_ptr(), sum_ints(4, 10, 20, 30, 40));
        }
        0 // the process exit code
    }

    /// Rust's prebuilt `core` library is compiled with support for
    /// unwinding (cleaning up during a panic), so it refers to an
    /// "exception handling personality" function that `std` normally
    /// provides. With `panic = "abort"` (see Cargo.toml) it's never actually
    /// called, but the linker still needs the symbol to exist:
    ///     Undefined symbols for architecture arm64: "_rust_eh_personality"
    /// An empty function satisfies it. This is the usual fix for no_std
    /// programs on a desktop OS with stable Rust.
    #[unsafe(no_mangle)]
    pub extern "C" fn rust_eh_personality() {}

    /// `core` doesn't know how to report a panic: without std there's no
    /// stderr, no threads and no unwinding. Every no_std program must say
    /// what a panic does. On a normal OS, abort the process.
    #[panic_handler]
    fn on_panic(_info: &core::panic::PanicInfo) -> ! {
        // SAFETY: abort has no preconditions; it ends the process immediately.
        unsafe { libc::abort() }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_returns_the_rest_of_the_string() {
        assert_eq!(find(c"Hello World!", c"World"), Some(c"World!"));
        assert_eq!(find(c"Hello", c"xyz"), None);
        assert_eq!(find(c"abc", c""), Some(c"abc")); // C: an empty needle matches at once
    }

    #[test]
    fn strlen_agrees_with_rust() {
        let text = c"twelve chars";
        assert_eq!(c_length(text), 12);
        assert_eq!(c_length(text), text.count_bytes());
    }

    #[test]
    fn qsort_with_a_rust_callback() {
        let mut numbers = [5, -2, 9, 0, 5];
        sort_with_qsort(&mut numbers);
        assert_eq!(numbers, [-2, 0, 5, 5, 9]);
    }

    #[test]
    fn malloc_buffer_is_filled_and_summed() {
        assert_eq!(sum_of_malloc_buffer(100), Some(5050));
        assert_eq!(sum_of_malloc_buffer(1), Some(1));
    }

    #[test]
    fn snprintf_formats_into_the_buffer() {
        let mut buffer = [0u8; 32];
        assert_eq!(format_temperature(&mut buffer, -5), c"-5 degrees");
    }

    #[test]
    fn getenv_finds_set_variables() {
        // PATH is set in practically every environment cargo runs in.
        assert!(env_var_length(c"PATH").is_some());
        assert_eq!(env_var_length(c"RUST_EXAMPLES_SURELY_NOT_SET"), None);
    }

    #[test]
    fn a_rust_variadic_function_reads_its_arguments() {
        // SAFETY: each call passes exactly `count` further ints.
        unsafe {
            assert_eq!(sum_ints(3, 1, 2, 3), 6);
            assert_eq!(sum_ints(0), 0);
            assert_eq!(sum_ints(2, -5, 5), 0);
        }
    }
}
