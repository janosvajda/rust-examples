#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]

extern crate libc;
extern crate libm;

use libc::{c_char, printf};
use libm::{cos, ceil};

#[cfg(not(test))]
mod no_std_specific {
    use super::*;

    #[no_mangle]
    pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
        // Calculate the cosine of the angle
        let angle: f64 = 1.0;
        let result_cos = cos(angle);

        // Calculate the ceiling of the cosine result
        let result_ceil = ceil(result_cos);

        // Print the results
        unsafe {
            printf("Cosine of %f is %f\n\0".as_ptr() as *const c_char, angle, result_cos);
            printf("Ceiling of %f is %f\n\0".as_ptr() as *const c_char, result_cos, result_ceil);
        }

        0
    }

    #[panic_handler]
    #[cfg(not(test))]
    fn my_panic(_info: &core::panic::PanicInfo) -> ! {
        loop {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libm::F64Ext;

    #[test]
    fn test_cosine() {
        // Test case to check the correctness of the cosine and ceiling calculations

        // Define the test angle
        let angle: f64 = 1.0;

        // Calculate the expected results using Rust's math functions
        let expected_result_cos = angle.cos();
        let expected_result_ceil = expected_result_cos.ceil();

        // Calculate the results using the C math library functions
        let result_cos = cos(angle);
        let result_ceil = ceil(result_cos);

        // Compare the results
        assert_eq!(result_cos, expected_result_cos);
        assert_eq!(result_ceil, expected_result_ceil);
    }
}
