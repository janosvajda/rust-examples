// Lesson 1: bits and bytes.
//
// A bit has one of two values, 0 or 1. Eight bits make a byte. The same
// bits can mean a number, a letter or a colour: the meaning comes from the
// type or the encoding we choose to read them with.

/// The eight bits of a byte, as text: 5 → "00000101".
fn bits(byte: u8) -> String {
    format!("{byte:08b}")
}

/// The red, green and blue amounts packed in a colour like 0x00FF8800.
///   `>> 16` shifts the bits 16 places to the right, so red ends up in the lowest byte;
///   `& 0xFF` (a "mask") keeps only the lowest 8 bits and clears all the others.
fn red_green_blue(colour: u32) -> (u32, u32, u32) {
    ((colour >> 16) & 0xFF, (colour >> 8) & 0xFF, colour & 0xFF)
}

fn main() {
    println!("1. Counting in binary");
    for n in [0u8, 1, 2, 3, 4, 5, 10, 255] {
        println!("    {n:>3} = {}", bits(n));
    }

    println!("\n2. When the number doesn't fit in a u8 (0 to 255)");
    println!("    255.checked_add(1)  = {:?}", 255u8.checked_add(1));
    println!("    255.wrapping_add(1) = {}", 255u8.wrapping_add(1));

    println!("\n3. The same eight bits, read as u8 and as i8");
    for pattern in [
        0b0000_0001u8,
        0b0111_1111,
        0b1000_0000,
        0b1111_1011,
        0b1111_1111,
    ] {
        println!(
            "    {}  as u8: {:>3}   as i8: {:>4}",
            bits(pattern),
            pattern,
            pattern as i8
        );
    }

    println!("\n4. Text in UTF-8: each character's Unicode number, and its bytes");
    for character in ['A', 'é', '€', '🦀'] {
        let mut buffer = [0u8; 4];
        let encoded = character.encode_utf8(&mut buffer);
        let hex: Vec<String> = encoded.bytes().map(|b| format!("{b:02x}")).collect();
        println!(
            "    {character}  U+{:04X}  → {} byte(s): {}",
            character as u32,
            encoded.len(),
            hex.join(" ")
        );
    }

    println!("\n5. A colour: three amounts packed into one number");
    let (red, green, blue) = red_green_blue(0x00FF_8800);
    println!("    0x00FF8800 → red {red}, green {green}, blue {blue}");

    println!("\n6. 0.1 can't be stored exactly in binary");
    let bits_of_tenth = format!("{:064b}", 0.1f64.to_bits());
    println!("    0.1 is stored as {:.30}", 0.1f64);
    println!(
        "    its 64 bits: sign {}  exponent {}  fraction {}",
        &bits_of_tenth[..1],
        &bits_of_tenth[1..12],
        &bits_of_tenth[12..]
    );
    println!("    0.1 + 0.2 = {}", 0.1f64 + 0.2);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binary_digits() {
        assert_eq!(bits(5), "00000101");
        assert_eq!(bits(255), "11111111");
        assert_eq!(0b0000_0101, 5);
    }

    #[test]
    fn the_same_bits_as_unsigned_and_signed() {
        assert_eq!(0b1111_1111u8 as i8, -1);
        assert_eq!(0b1111_1011u8 as i8, -5); // 251 - 256
        assert_eq!(0b1000_0000u8 as i8, -128);
        assert_eq!(0b1111_1110u8 as i8, -2); // the README's "try it"
    }

    #[test]
    fn text_is_utf8_bytes() {
        assert_eq!("A".as_bytes(), [0x41]);
        assert_eq!("é".as_bytes(), [0xc3, 0xa9]);
        assert_eq!("€".len(), 3);
        assert_eq!("🦀".len(), 4); // four bytes…
        assert_eq!("🦀".chars().count(), 1); // …one Unicode scalar value
        assert_eq!("e\u{301}".len(), 3); // e + a combining accent: three bytes…
        assert_eq!("e\u{301}".chars().count(), 2); // …two scalar values
    }

    #[test]
    fn shifts_and_masks_unpack_a_colour() {
        assert_eq!(red_green_blue(0x00FF_8800), (255, 136, 0));
        assert_eq!(red_green_blue(0x0012_3456), (0x12, 0x34, 0x56));
    }

    #[test]
    fn decimals_in_binary_are_approximate() {
        assert_ne!(0.1f64 + 0.2, 0.3);
        assert_eq!(format!("{}", 0.1f64 + 0.2), "0.30000000000000004");
    }
}
