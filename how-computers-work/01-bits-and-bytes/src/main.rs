// Lesson 1: bits and bytes.
//
// A computer stores everything as bits: switches that are either off (0) or
// on (1). Eight bits make a byte. Numbers, letters, colours, pictures and
// programs are all just bytes; only the way we READ them differs.

/// The bits of a byte, as text: 5 → "00000101".
fn bits(byte: u8) -> String {
    format!("{byte:08b}")
}

fn main() {
    println!("1. Counting in binary: each place is worth twice the one to its right");
    for n in [0u8, 1, 2, 3, 4, 5, 10, 255] {
        println!("    {n:>3} = {}", bits(n));
    }

    println!("\n2. A byte holds 256 different values: 0 to 255");
    let full: u8 = 255;
    println!("    255 + 1 doesn't fit: checked_add says {:?}", full.checked_add(1));
    println!("    wrapping_add gives {}: like a car's odometer rolling over", full.wrapping_add(1));

    println!("\n3. Negative numbers: the same bits, read differently (two's complement)");
    for n in [1i8, -1, -5, 127, -128] {
        println!("    {n:>4} as i8 is stored as {}, which read as a u8 is {}", bits(n as u8), n as u8);
    }

    println!("\n4. Text is bytes too (UTF-8)");
    for text in ["A", "a", "é", "€", "🦀"] {
        let bytes: Vec<String> = text.bytes().map(bits).collect();
        println!("    {text}  → {} byte(s): {}", text.len(), bytes.join(" "));
    }

    println!("\n5. A colour is bytes: red, green and blue, 0 to 255 each");
    let orange: u32 = 0x00FF_8800;
    let [_, red, green, blue] = orange.to_be_bytes();
    println!("    0x00FF8800 → red {red}, green {green}, blue {blue}");

    println!("\n6. Some numbers can't be stored exactly");
    println!("    0.1 as 64 bits: {:064b}", 0.1f64.to_bits());
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
    fn negative_numbers_are_twos_complement() {
        assert_eq!(-1i8 as u8, 255); // all eight bits on
        assert_eq!(-5i8 as u8, 251); // 256 - 5
        assert_eq!((-128i8) as u8, 0b1000_0000);
    }

    #[test]
    fn text_is_utf8_bytes() {
        assert_eq!("A".as_bytes(), [65]);
        assert_eq!("é".len(), 2);
        assert_eq!("€".len(), 3);
        assert_eq!("🦀".len(), 4);
        assert_eq!("🦀".chars().count(), 1); // four bytes, one character
    }

    #[test]
    fn decimals_in_binary_are_approximate() {
        assert_ne!(0.1f64 + 0.2, 0.3);
        assert!((0.1f64 + 0.2 - 0.3).abs() < 1e-15);
    }
}
