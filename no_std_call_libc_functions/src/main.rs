#![no_std]
#![no_main]

extern crate libc;

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    const HAYSTACK: & str = "Hello World!\n\0";
    const NEEDLE: & str = "World!\n\0";
    unsafe {
        let result = libc::strstr(HAYSTACK.as_ptr() as *const i8, NEEDLE.as_ptr() as *const i8);
        libc::printf(result);
    }
    0
}

#[panic_handler]
#[cfg(not(test))]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}