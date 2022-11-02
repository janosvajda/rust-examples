#![no_std]
#![no_main]

extern crate libc;

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    const TEXT: &'static str = "Hello, world from libc!";
    unsafe {
        libc::printf(TEXT.as_ptr() as *const _);
    }
    0
}

#[panic_handler]
#[cfg(not(test))]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}