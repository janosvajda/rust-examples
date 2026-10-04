// Lesson 3: pointers are addresses.
//
// A pointer is nothing more than an address: a number saying which mailbox a
// value starts in. A Rust reference (&T) is a pointer with guarantees: it's
// never null, never dangling, and always points to the right type.

/// The address inside a reference, as a plain number.
fn address_of<T>(value: &T) -> usize {
    value as *const T as usize
}

/// Finds `numbers[index]` the way the processor does: start + index × size.
/// Returns the address it computed, and the address Rust's indexing used.
fn where_is(numbers: &[u32], index: usize) -> (usize, usize) {
    let computed = address_of(&numbers[0]) + index * size_of::<u32>();
    (computed, address_of(&numbers[index]))
}

fn main() {
    println!("1. A reference holds an address");
    let score: u32 = 42;
    let pointer: &u32 = &score;
    println!("    score lives at       {:#x}", address_of(&score));
    println!("    the reference holds  {:#x}", address_of(pointer));
    println!("    following it gives   {}", *pointer);
    println!("    the reference itself is {} bytes: just the address", size_of::<&u32>());

    println!("\n2. Indexing is arithmetic: start + index × size");
    let numbers = [10u32, 20, 30, 40];
    for i in 0..numbers.len() {
        let (computed, actual) = where_is(&numbers, i);
        println!("    numbers[{i}]: start + {i} × 4 = {computed:#x}  (Rust used {actual:#x})");
    }

    println!("\n3. A pointer to a pointer: an address of an address");
    let reference_to_reference: &&u32 = &pointer;
    println!("    the outer reference holds {:#x}, where the inner one is stored", address_of(reference_to_reference));
    println!("    following both gives {}", **reference_to_reference);

    println!("\n4. A wrong address: Rust checks every index");
    let index = std::hint::black_box(10); // a value the compiler can't see in advance
    match numbers.get(index) {
        Some(n) => println!("    numbers[{index}] = {n}"),
        None => println!("    numbers.get({index}) = None: there is no mailbox {index} in this array"),
    }
    println!("    (numbers[{index}] would stop the program: index out of bounds)");

    println!("\n5. \"Maybe no address\" is an Option, and costs nothing");
    let found: Option<&u32> = numbers.iter().find(|&&n| n > 25);
    let missing: Option<&u32> = numbers.iter().find(|&&n| n > 99);
    println!("    found {found:?}, missing {missing:?}");
    println!("    Option<&u32> is {} bytes, like &u32: None is stored as address 0", size_of::<Option<&u32>>());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_reference_is_the_address_of_its_value() {
        let x = 7u64;
        let r = &x;
        assert_eq!(address_of(r), address_of(&x));
        assert_eq!(size_of::<&u64>(), size_of::<usize>());
    }

    #[test]
    fn indexing_is_start_plus_index_times_size() {
        let numbers = [1u32, 2, 3, 4, 5];
        for i in 0..numbers.len() {
            let (computed, actual) = where_is(&numbers, i);
            assert_eq!(computed, actual);
        }
    }

    #[test]
    #[should_panic(expected = "index out of bounds: the len is 4 but the index is 10")]
    fn a_wrong_index_stops_the_program_instead_of_reading_other_memory() {
        let numbers = [10u32, 20, 30, 40];
        let _ = numbers[std::hint::black_box(10)];
    }

    #[test]
    fn option_of_a_reference_needs_no_extra_space() {
        assert_eq!(size_of::<Option<&u32>>(), size_of::<&u32>());
    }
}
