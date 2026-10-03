#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate libc;

#[cfg(not(test))]
mod no_std_specific {
    use libc::{c_char, printf};
    
    #[no_mangle]
    pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
        let res = super::call_strstr();

        unsafe {
            match res {
                // `c"..."` is a C string literal: Rust adds the terminating nul byte for us.
                Some(substring) => printf(c"Substring found: %s\n".as_ptr(), substring.as_ptr() as *const c_char),
                None => printf(c"Substring not found!\n".as_ptr()),
            };
        }

        0
    }

    #[panic_handler]
    #[cfg(not(test))]
    fn my_panic(_info: &core::panic::PanicInfo) -> ! {
        loop {}
    }
}

pub fn call_strstr() -> Option<&'static [u8]> {
    let haystack: &'static [u8] = b"Hello World!\n\0";
    let needle: &'static [u8] = b"World!\n\0";

    unsafe {
        let found_ptr = libc::strstr(haystack.as_ptr() as *const i8, needle.as_ptr() as *const i8);

        if found_ptr.is_null() {
            None
        } else {
            let found_index = found_ptr as usize - haystack.as_ptr() as usize;
            Some(&haystack[found_index..])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_call_strstr() {
        let result = call_strstr().unwrap();
        assert_eq!(result, b"World!\n\0");
    }
}
