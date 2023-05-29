fn get_value_from_raw_pointer(raw_p: *const u32) -> u32 {
    unsafe { *raw_p }
}

fn main() {
    let raw_p: *const u32 = &99;
    let p = get_value_from_raw_pointer(raw_p);

    if p == 99 {
        println!("{}", p);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_value_from_raw_pointer() {
        let raw_p: *const u32 = &99;
        let value = get_value_from_raw_pointer(raw_p);
        assert_eq!(value, 99);
    }
}
