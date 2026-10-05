// Lesson 5: Rust on a target with NO operating system at all.
//
// `wasm32-unknown-unknown` is a truly bare target: the "unknown" parts mean
// no known operating system and no known environment. There are no files,
// no clock, no printing, no C library, no threads. The module can only
// compute and use its own memory. Whatever loads it (a browser, Node.js)
// calls the functions it EXPORTS.
//
// That's the same situation as a microcontroller without peripherals: only
// `core` makes sense, and every interaction with the outside world has to
// go through an explicit interface.
#![no_std]

// ---- Exported functions --------------------------------------------------------
//
// `#[unsafe(no_mangle)]` keeps the Rust name, so JavaScript can find it, and
// `extern "C"` uses the plain calling convention WebAssembly exports need.
// Only simple numbers cross the boundary: i32, i64, f32, f64.

/// The simplest possible export.
#[unsafe(no_mangle)]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a.wrapping_add(b)
}

/// Some real computation: the n-th Fibonacci number.
#[unsafe(no_mangle)]
pub extern "C" fn fibonacci(n: u32) -> u64 {
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 0..n {
        (a, b) = (b, a.wrapping_add(b));
    }
    a
}

/// How many primes are below `limit`. Heavy enough to be worth running in
/// WebAssembly instead of JavaScript.
#[unsafe(no_mangle)]
pub extern "C" fn count_primes(limit: u32) -> u32 {
    (2..limit).filter(|&n| is_prime(n)).count() as u32
}

fn is_prime(n: u32) -> bool {
    (2..)
        .take_while(|d| d * d <= n)
        .all(|d| !n.is_multiple_of(d))
}

// ---- Passing text: through shared memory ----------------------------------------
//
// Strings can't be passed as arguments; only numbers can. The standard
// technique is to share MEMORY: the module exposes a buffer, JavaScript
// writes bytes into it, then calls a function with the length.

const BUFFER_SIZE: usize = 1024;

/// The shared buffer. `static mut` is fine to use through raw pointers
/// (`&raw mut`), which avoids creating references to mutable statics.
static mut BUFFER: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

/// Tells JavaScript where the buffer is in the module's memory.
#[unsafe(no_mangle)]
pub extern "C" fn buffer_address() -> *mut u8 {
    (&raw mut BUFFER).cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn buffer_size() -> usize {
    BUFFER_SIZE
}

/// Counts the vowels in the first `len` bytes JavaScript wrote to the buffer.
#[unsafe(no_mangle)]
pub extern "C" fn count_vowels(len: usize) -> u32 {
    let text = buffer(len);
    text.iter().filter(|b| b"aeiouAEIOU".contains(b)).count() as u32
}

/// Turns the first `len` bytes of the buffer into upper case, IN PLACE, so
/// JavaScript can read the result back from the same memory.
#[unsafe(no_mangle)]
pub extern "C" fn to_upper_in_place(len: usize) {
    let len = len.min(BUFFER_SIZE);
    // SAFETY: the module is single-threaded and the slice doesn't outlive
    // this call, so nothing else can access BUFFER at the same time.
    let text = unsafe { core::slice::from_raw_parts_mut((&raw mut BUFFER).cast::<u8>(), len) };
    text.make_ascii_uppercase();
}

fn buffer(len: usize) -> &'static [u8] {
    let len = len.min(BUFFER_SIZE);
    // SAFETY: as above; the buffer is only read here.
    unsafe { core::slice::from_raw_parts((&raw const BUFFER).cast::<u8>(), len) }
}

// ---- Panics ------------------------------------------------------------------------

/// With no OS there's nobody to print a panic message to. WebAssembly's
/// `unreachable` instruction stops the module with a "trap", which the host
/// (JavaScript) sees as an exception.
#[panic_handler]
fn on_panic(_info: &core::panic::PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}
