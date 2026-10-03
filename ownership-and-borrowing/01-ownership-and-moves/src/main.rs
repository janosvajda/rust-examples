// Lesson 1: ownership and moves.
//
// The three ownership rules:
//   1. Every value has exactly one owner (a variable, a field, a collection...).
//   2. Ownership can be moved to a new owner. The old owner can't be used after that.
//   3. When the owner goes out of scope, the value is dropped (its memory is freed).

/// A type that announces when it's dropped, so we can watch rule 3 happen.
struct Noisy(&'static str);

impl Drop for Noisy {
    fn drop(&mut self) {
        println!("    dropping {}", self.0);
    }
}

/// Takes ownership of `text`. When this function ends, `text` is dropped.
fn consume(text: String) -> usize {
    text.len()
} // `text` goes out of scope here and its memory is freed

/// Takes ownership, then gives it back to the caller by returning it.
fn shout(mut text: String) -> String {
    text.push('!');
    text // ownership moves out to whoever called us
}

fn main() {
    println!("1. A move: ownership changes hands");
    let a = String::from("hello");
    let b = a; // the String moves from `a` to `b`
    // println!("{a}");
    // error[E0382]: borrow of moved value: `a`
    //   move occurs because `a` has type `String`, which does not implement the `Copy` trait
    println!("    b = {b}");

    println!("\n2. A copy: simple values are duplicated instead");
    let x = 5;
    let y = x; // i32 implements `Copy`, so `x` is copied, not moved
    println!("    x = {x}, y = {y} (both still usable)");

    println!("\n3. A clone: an explicit, full copy");
    let original = String::from("data");
    let duplicate = original.clone(); // copies the text on the heap too
    println!("    original = {original}, duplicate = {duplicate}");

    println!("\n4. Moving into a function");
    let message = String::from("goodbye");
    let length = consume(message); // `message` moves into `consume`
    // println!("{message}");
    // error[E0382]: borrow of moved value: `message`
    println!("    length was {length}; `message` can't be used any more");

    println!("\n5. Moving out of a function");
    let greeting = shout(String::from("hi")); // ownership comes back to us
    println!("    greeting = {greeting}");

    println!("\n6. Drop happens at the end of the owner's scope, in reverse order");
    {
        let _first = Noisy("first");
        let _second = Noisy("second");
        println!("    leaving the block…");
    } // `_second` is dropped, then `_first`

    println!("\n7. Moving a value into a collection");
    let item = Noisy("item in a Vec");
    let items = vec![item]; // `item` now belongs to the Vec
    println!("    the Vec owns {} item; dropping the Vec drops it too:", items.len());
    drop(items); // `drop` just takes ownership and lets it go out of scope
    println!("    done");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_types_stay_usable_after_assignment() {
        let x = 7;
        let y = x;
        assert_eq!(x + y, 14);
    }

    #[test]
    fn clone_gives_an_independent_copy() {
        let original = String::from("a");
        let mut copy = original.clone();
        copy.push('b');
        assert_eq!(original, "a");
        assert_eq!(copy, "ab");
    }

    #[test]
    fn ownership_can_go_in_and_come_back() {
        let s = shout(String::from("hey"));
        assert_eq!(s, "hey!");
        assert_eq!(consume(s), 4);
    }

    #[test]
    fn a_move_does_not_copy_the_heap_data() {
        // A move copies only the String's small header (pointer, capacity,
        // length). The text itself stays where it is on the heap.
        let a = String::from("same heap memory");
        let address_before = a.as_ptr();
        let b = a;
        assert_eq!(b.as_ptr(), address_before);
    }
}
