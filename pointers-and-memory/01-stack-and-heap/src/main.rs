// Lesson 1: the stack and the heap.
//
// Every value lives in one of two places:
//   the STACK   fast, automatic, fixed size known at compile time; freed when
//               the function returns
//   the HEAP    for data whose size is only known at run time, or that must
//               outlive the function; reached through a pointer
//
// A `String`, `Vec` or `Box` is a small, fixed-size value on the stack (a
// pointer plus some numbers) that owns data on the heap.

use std::mem::size_of;

/// A struct's size is (at least) the sum of its fields: all on the stack.
#[allow(dead_code)]
struct Point {
    x: f64,
    y: f64,
}

/// Where a String's text lives: the heap address its pointer points to.
fn heap_address(text: &str) -> *const u8 {
    text.as_ptr()
}

/// Takes ownership of a String. Only the 24-byte stack part is copied into
/// this function; the text on the heap stays exactly where it is.
fn take(text: String) -> *const u8 {
    heap_address(&text)
}

fn main() {
    println!("1. How big is a value on the stack? (size_of, in bytes)");
    let sizes = [
        ("u8", size_of::<u8>()),
        ("i32", size_of::<i32>()),
        ("f64", size_of::<f64>()),
        ("Point { x: f64, y: f64 }", size_of::<Point>()),
        ("[u8; 1000]", size_of::<[u8; 1000]>()),
        ("&i32 (a reference)", size_of::<&i32>()),
        ("&str (pointer + length)", size_of::<&str>()),
        ("String (pointer + length + capacity)", size_of::<String>()),
        ("Vec<u64> (the same three)", size_of::<Vec<u64>>()),
        ("Box<[u8; 1000]> (just a pointer)", size_of::<Box<[u8; 1000]>>()),
        ("Option<Box<i32>> (still one pointer)", size_of::<Option<Box<i32>>>()),
    ];
    for (name, size) in sizes {
        println!("    {name:<38} {size:>5}");
    }

    println!("\n2. A String's size doesn't depend on its text");
    let short = String::from("hi");
    let long = "x".repeat(1_000_000);
    println!("    \"hi\":        {} bytes on the stack, {} bytes of text on the heap", size_of_val(&short), short.len());
    println!("    a million x: {} bytes on the stack, {} bytes of text on the heap", size_of_val(&long), long.len());

    println!("\n3. Moving a String copies 24 bytes, never the text");
    let before = heap_address(&long);
    let after = take(long); // `long` is moved into `take`
    println!("    heap address before the move: {before:p}");
    println!("    heap address after the move:  {after:p}");
    println!("    the same: {}", before == after);

    println!("\n4. Cloning copies the heap data too");
    let original = String::from("copy me");
    let copy = original.clone();
    println!("    original text at {:p}, the clone's at {:p}", original.as_ptr(), copy.as_ptr());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pointer_types_are_pointer_sized() {
        let pointer = size_of::<usize>(); // 8 on 64-bit machines
        assert_eq!(size_of::<&u8>(), pointer);
        assert_eq!(size_of::<Box<[u8; 1000]>>(), pointer);
        assert_eq!(size_of::<&str>(), 2 * pointer);
        assert_eq!(size_of::<String>(), 3 * pointer);
        // Option<Box<T>> needs no extra space: a Box is never null, so null means None
        assert_eq!(size_of::<Option<Box<u64>>>(), pointer);
    }

    #[test]
    fn a_move_keeps_the_heap_data_where_it_is() {
        let text = "x".repeat(10_000);
        let before = heap_address(&text);
        assert_eq!(take(text), before);
    }

    #[test]
    fn a_clone_has_its_own_heap_data() {
        let original = String::from("abc");
        let copy = original.clone();
        assert_ne!(original.as_ptr(), copy.as_ptr());
        assert_eq!(original, copy);
    }
}
