// no_std + alloc: getting Vec, String and Box back, without std.
//
// The `alloc` crate contains Rust's heap types (Vec, String, Box, BTreeMap…).
// They work anywhere, as long as SOMEONE provides a heap allocator. On a
// normal OS, std provides one. Here we write our own: a "bump allocator"
// that hands out memory from a fixed 64 KB block, the way firmware on a
// microcontroller often does.
#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate alloc; // `alloc` isn't in scope automatically, unlike `core`

use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicUsize, Ordering};

// ============================================================================
// A bump allocator
// ============================================================================

/// The simplest possible allocator. It keeps one offset into a fixed block
/// of memory and "bumps" it forward on every allocation:
///
/// ```text
///   arena: [ used │ used │ used │ ← next │            free            ]
///          0                       ▲                                64 KB
///                                  every new allocation starts here
/// ```
///
/// Allocating is a few instructions, which is extremely fast. Freeing does
/// nothing: memory is only reclaimed when the whole arena is reset. That
/// suits programs that allocate during start-up and then run forever, or
/// that handle one request at a time and reset in between.
pub struct BumpAllocator<const SIZE: usize> {
    // UnsafeCell: the memory is changed through a shared reference (the
    // allocator is a `static`). Every access below goes through raw pointers.
    arena: UnsafeCell<[u8; SIZE]>,
    next: AtomicUsize,        // offset of the first free byte
    allocations: AtomicUsize, // statistics, for the demo
    bytes_freed: AtomicUsize, // freed, but not reusable until reset
}

// SAFETY: the only mutable state shared between callers is `next`, which is
// updated atomically, so two allocations can never receive the same bytes.
unsafe impl<const SIZE: usize> Sync for BumpAllocator<SIZE> {}

impl<const SIZE: usize> BumpAllocator<SIZE> {
    pub const fn new() -> Self {
        BumpAllocator {
            arena: UnsafeCell::new([0; SIZE]),
            next: AtomicUsize::new(0),
            allocations: AtomicUsize::new(0),
            bytes_freed: AtomicUsize::new(0),
        }
    }

    pub fn bytes_used(&self) -> usize {
        self.next.load(Ordering::Relaxed)
    }

    pub fn allocations(&self) -> usize {
        self.allocations.load(Ordering::Relaxed)
    }

    pub fn bytes_freed(&self) -> usize {
        self.bytes_freed.load(Ordering::Relaxed)
    }
}

impl<const SIZE: usize> Default for BumpAllocator<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: `alloc` returns either null or a pointer to `layout.size()` bytes,
// aligned to `layout.align()`, that no other allocation overlaps.
unsafe impl<const SIZE: usize> GlobalAlloc for BumpAllocator<SIZE> {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let base = self.arena.get() as usize;
        let mut current = self.next.load(Ordering::Relaxed);
        loop {
            // Round the address up to the alignment the type needs (e.g. a
            // u64 must start at a multiple of 8).
            let start = (base + current).next_multiple_of(layout.align()) - base;
            let end = match start.checked_add(layout.size()) {
                Some(end) if end <= SIZE => end,
                _ => return core::ptr::null_mut(), // out of memory: report failure
            };
            // Claim [start, end) atomically, in case of concurrent callers.
            match self
                .next
                .compare_exchange(current, end, Ordering::Relaxed, Ordering::Relaxed)
            {
                Ok(_) => {
                    self.allocations.fetch_add(1, Ordering::Relaxed);
                    return (base + start) as *mut u8;
                }
                Err(actual) => current = actual, // someone else allocated first: retry
            }
        }
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, layout: Layout) {
        // A bump allocator never reuses memory, so freeing is just counted.
        self.bytes_freed.fetch_add(layout.size(), Ordering::Relaxed);
    }
}

// ============================================================================
// The program
// ============================================================================

#[cfg(not(test))]
mod program {
    use super::BumpAllocator;
    use alloc::boxed::Box;
    use alloc::collections::BTreeMap;
    use alloc::format;
    use alloc::string::String;
    use alloc::vec::Vec;
    use core::ffi::{c_char, c_int};

    /// `#[global_allocator]` makes this THE heap for Vec, String, Box and
    /// friends. Exactly one per program.
    #[global_allocator]
    static HEAP: BumpAllocator<65_536> = BumpAllocator::new();

    /// Writes text to standard output with the C `write` system call.
    fn print(text: &str) {
        // SAFETY: the pointer and length describe a valid &str.
        unsafe { libc::write(1, text.as_ptr().cast(), text.len()) };
    }

    fn report(step: &str) {
        print(&format!(
            "    [heap: {:>5} of 65536 bytes used, {} allocations, {} bytes freed but not reusable] {step}\n",
            HEAP.bytes_used(),
            HEAP.allocations(),
            HEAP.bytes_freed()
        ));
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn main(_argc: c_int, _argv: *const *const c_char) -> c_int {
        print("no_std + alloc, with our own bump allocator\n\n");

        print("1. Vec and String work again\n");
        let mut names: Vec<String> = Vec::new();
        for name in ["Ana", "Bob", "Cleo"] {
            names.push(String::from(name));
        }
        print(&format!("    names = {names:?}\n"));
        report("after building the Vec");

        print("\n2. format!, Box and BTreeMap too\n");
        let boxed: Box<[u32]> = (1..=10).map(|n| n * n).collect();
        let mut stock: BTreeMap<&str, u32> = BTreeMap::new();
        stock.insert("apples", 12);
        stock.insert("pears", 7);
        print(&format!("    squares = {boxed:?}\n    stock = {stock:?}\n"));
        report("after the Box and the BTreeMap");

        print("\n3. Freeing doesn't give memory back (that's the bump trade-off)\n");
        drop(names);
        drop(boxed);
        report("after dropping the Vec and the Box");

        print("\n4. Running out of memory, handled gracefully\n");
        let mut big: Vec<u8> = Vec::new();
        match big.try_reserve(100_000) {
            Ok(()) => print("    reserved 100,000 bytes\n"),
            Err(_) => print("    try_reserve(100,000) refused: the arena only has 64 KB\n"),
        }
        // `Vec::with_capacity(100_000)` instead would call the allocation
        // error handler, which panics, and our panic handler aborts.
        0
    }

    #[panic_handler]
    fn on_panic(_info: &core::panic::PanicInfo) -> ! {
        print("panic: aborting\n");
        // SAFETY: abort has no preconditions; it ends the process.
        unsafe { libc::abort() }
    }

    /// Needed by the prebuilt `core` library when std isn't linked; never
    /// called because panics abort. See lesson 1 for the full explanation.
    #[unsafe(no_mangle)]
    pub extern "C" fn rust_eh_personality() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocations_are_aligned_and_do_not_overlap() {
        let heap: BumpAllocator<1024> = BumpAllocator::new();
        let byte = Layout::from_size_align(1, 1).unwrap();
        let word = Layout::from_size_align(8, 8).unwrap();
        // SAFETY: valid layouts; the pointers are only compared, not used.
        let (a, b) = unsafe { (heap.alloc(byte), heap.alloc(word)) };
        assert!(!a.is_null() && !b.is_null());
        assert_eq!(b as usize % 8, 0); // rounded up to the alignment
        assert!(b as usize > a as usize); // after the first allocation
        assert_eq!(heap.allocations(), 2);
    }

    #[test]
    fn returns_null_when_full() {
        let heap: BumpAllocator<64> = BumpAllocator::new();
        let layout = Layout::from_size_align(48, 1).unwrap();
        // SAFETY: valid layouts.
        unsafe {
            assert!(!heap.alloc(layout).is_null());
            assert!(heap.alloc(layout).is_null()); // only 16 bytes left
        }
    }

    #[test]
    fn freeing_is_counted_but_not_reused() {
        let heap: BumpAllocator<64> = BumpAllocator::new();
        let layout = Layout::from_size_align(32, 1).unwrap();
        // SAFETY: the pointer came from this allocator with this layout.
        unsafe {
            let p = heap.alloc(layout);
            heap.dealloc(p, layout);
        }
        assert_eq!(heap.bytes_used(), 32); // still used
        assert_eq!(heap.bytes_freed(), 32);
    }
}
