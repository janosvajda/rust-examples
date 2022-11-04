#![no_std]
#![no_main]
extern crate libc;

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    unsafe {
        //Prints characters and values to stdout.
        libc::printf(call_strstr());
    }
    0
}

fn call_strstr() -> *mut i8 {
    const HAYSTACK: &str = "Hello World!\n\0";
    const NEEDLE: &str = "World!\n\0";
    // Finds the first occurrence of a substring in a string
    unsafe {
        let result = libc::strstr(HAYSTACK.as_ptr() as *const i8, NEEDLE.as_ptr() as *const i8);
        return result;
    }
}

#[panic_handler]
#[cfg(not(test))]
fn my_panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_strstr() {
        let haystack = "Hello Gorld!\n\0";
        let result = call_strstr() as *mut char;
        let the_bits = result as *const _ as usize;
        assert_eq!(&haystack[..the_bits], "World!");
    }
    #[test]
    fn test_call_strstr_position() {
        let result = call_strstr() as *mut char;
        let the_bits = result as *const _ as usize;
        assert_eq!(the_bits, 2);
    }
}
