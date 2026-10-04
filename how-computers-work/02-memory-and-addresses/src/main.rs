// Lesson 2: memory and addresses.
//
// In the memory model this course uses, every byte has a number: its address.
// A value bigger than a byte takes several neighbouring bytes. This lesson
// looks at real addresses, at the order of the bytes, and at the rules for
// where a value may start (alignment), which can leave gaps in a struct.

use std::mem::{align_of, offset_of, size_of};

/// A C-style struct: fields stay in the order written, with gaps for alignment.
#[repr(C)]
struct InCOrder {
    small: u8,
    big: u32,
    other: u8,
}

/// The same fields with Rust's default layout: the compiler may reorder them
/// to avoid gaps.
struct RustOrder {
    small: u8,
    big: u32,
    other: u8,
}

fn main() {
    println!("1. The addresses of three local values (the number of each one's first byte)");
    let a: u8 = 1;
    let b: u32 = 2;
    let c: u64 = 3;
    println!("    a (u8,  1 byte)  is at {:p}", &a);
    println!("    b (u32, 4 bytes) is at {:p}", &b);
    println!("    c (u64, 8 bytes) is at {:p}", &c);

    println!("\n2. An array is neighbouring mailboxes");
    let numbers: [u32; 4] = [10, 20, 30, 40];
    for (i, n) in numbers.iter().enumerate() {
        let address = n as *const u32 as usize;
        let step = address - numbers.as_ptr() as usize;
        println!("    numbers[{i}] = {n}: address {address:#x} = start + {step} bytes");
    }

    println!("\n3. A u32 is 4 bytes: in which order are they stored?");
    let value: u32 = 0x1234_5678;
    let in_memory = value.to_ne_bytes(); // the bytes exactly as they lie in memory
    println!("    0x12345678 in memory: {:02x?}", in_memory);
    println!(
        "    this computer is {}-endian",
        if cfg!(target_endian = "little") {
            "little"
        } else {
            "big"
        }
    );

    println!("\n4. Alignment on this computer: where each type may start");
    for (name, size, align) in [
        ("u8", size_of::<u8>(), align_of::<u8>()),
        ("u16", size_of::<u16>(), align_of::<u16>()),
        ("u32", size_of::<u32>(), align_of::<u32>()),
        ("u64", size_of::<u64>(), align_of::<u64>()),
    ] {
        println!("    {name:<4} size {size}, must start at a multiple of {align}");
    }
    let alignment = align_of::<u32>();
    println!(
        "    b's address modulo {alignment} = {}",
        &b as *const u32 as usize % alignment
    );

    println!("\n5. Gaps: the same three fields, two layouts");
    println!(
        "    in C order (repr(C)): {} bytes: small at {}, big at {}, other at {}",
        size_of::<InCOrder>(),
        offset_of!(InCOrder, small),
        offset_of!(InCOrder, big),
        offset_of!(InCOrder, other)
    );
    println!(
        "    Rust's choice:        {} bytes: small at {}, big at {}, other at {}",
        size_of::<RustOrder>(),
        offset_of!(RustOrder, small),
        offset_of!(RustOrder, big),
        offset_of!(RustOrder, other)
    );
    // Use the fields, so the compiler doesn't warn that they're never read.
    let (x, y) = (
        InCOrder {
            small: 1,
            big: 2,
            other: 3,
        },
        RustOrder {
            small: 1,
            big: 2,
            other: 3,
        },
    );
    let _ = (x.small, x.big, x.other, y.small, y.big, y.other);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_elements_are_size_apart() {
        let numbers: [u64; 3] = [1, 2, 3];
        let first = &numbers[0] as *const u64 as usize;
        let second = &numbers[1] as *const u64 as usize;
        assert_eq!(second - first, size_of::<u64>());
    }

    #[test]
    fn values_are_aligned() {
        let value: u64 = 7;
        assert_eq!(&value as *const u64 as usize % align_of::<u64>(), 0);
    }

    #[test]
    fn byte_order() {
        assert_eq!(0x1234_5678u32.to_le_bytes(), [0x78, 0x56, 0x34, 0x12]); // little-endian
        assert_eq!(0x1234_5678u32.to_be_bytes(), [0x12, 0x34, 0x56, 0x78]); // big-endian
    }

    #[test]
    fn layouts_obey_their_documented_rules() {
        // repr(C) keeps field order, adding padding for this target's alignment.
        let big_offset = size_of::<u8>().next_multiple_of(align_of::<u32>());
        let other_offset = big_offset + size_of::<u32>();
        let alignment = align_of::<u8>().max(align_of::<u32>());
        let c_size = (other_offset + size_of::<u8>()).next_multiple_of(alignment);
        assert_eq!(offset_of!(InCOrder, small), 0);
        assert_eq!(offset_of!(InCOrder, big), big_offset);
        assert_eq!(offset_of!(InCOrder, other), other_offset);
        assert_eq!(align_of::<InCOrder>(), alignment);
        assert_eq!(size_of::<InCOrder>(), c_size);

        // Default Rust layout guarantees aligned, non-overlapping fields, not
        // a particular order or a size smaller than repr(C).
        let fields = [
            (
                offset_of!(RustOrder, small),
                size_of::<u8>(),
                align_of::<u8>(),
            ),
            (
                offset_of!(RustOrder, big),
                size_of::<u32>(),
                align_of::<u32>(),
            ),
            (
                offset_of!(RustOrder, other),
                size_of::<u8>(),
                align_of::<u8>(),
            ),
        ];
        for (i, &(start, size, field_alignment)) in fields.iter().enumerate() {
            assert_eq!(start % field_alignment, 0);
            assert!(start + size <= size_of::<RustOrder>());
            for &(other_start, other_size, _) in &fields[i + 1..] {
                assert!(start + size <= other_start || other_start + other_size <= start);
            }
        }
        assert!(align_of::<RustOrder>() >= alignment);
        assert_eq!(size_of::<RustOrder>() % align_of::<RustOrder>(), 0);
    }
}
