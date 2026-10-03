// Lesson 3: the borrowing rules.
//
// At any moment, a value can have EITHER
//   - any number of shared references (&T), OR
//   - exactly one mutable reference (&mut T),
// but not both. And while a value is borrowed, its owner can't move it,
// change it, or drop it either.
//
// A borrow lasts from where the reference is created to where it's LAST USED,
// not to the end of the block.

fn add_one(number: &mut i32) {
    *number += 1;
}

fn main() {
    println!("1. Many readers at once: allowed");
    let numbers = vec![1, 2, 3];
    let a = &numbers;
    let b = &numbers;
    println!("    a = {a:?}, b = {b:?}");

    println!("\n2. Two writers at once: not allowed");
    let mut score = 10;
    let first = &mut score;
    // let second = &mut score;
    // *first += 1;
    // error[E0499]: cannot borrow `score` as mutable more than once at a time
    *first += 1;
    let second = &mut score; // fine: `first` is no longer used after the line above
    *second += 1;
    println!("    score = {score}");

    println!("\n3. A reader and a writer at once: not allowed");
    let mut list = vec![1, 2, 3];
    let first_item = &list[0];
    // list.push(4);
    // println!("{first_item}");
    // error[E0502]: cannot borrow `list` as mutable because it is also borrowed as immutable
    println!("    first item = {first_item}");
    list.push(4); // fine: `first_item` was last used on the line above
    println!("    list = {list:?}");

    println!("\n4. Why rule 3 matters: the reference could end up pointing at freed memory");
    let mut grow = Vec::with_capacity(1);
    grow.push(10);
    let (capacity_before, address_before) = (grow.capacity(), grow.as_ptr());
    grow.push(20); // full, so the Vec must get a bigger buffer
    println!(
        "    capacity {} → {}; the items {} to a new address",
        capacity_before,
        grow.capacity(),
        if grow.as_ptr() != address_before { "moved" } else { "happened not to move" }
    );
    println!("    the compiler must assume they CAN move, so it forbids the reference");

    println!("\n5. Changing a collection while looping over it: not allowed");
    let mut items = vec![1, 2, 3];
    // for x in &items {
    //     items.push(*x);
    // }
    // error[E0502]: cannot borrow `items` as mutable because it is also borrowed as immutable
    let copies: Vec<i32> = items.clone();
    items.extend(copies); // read first, then change: fine
    println!("    items = {items:?}");

    println!("\n6. The owner can't move a value away while it's borrowed");
    let text = String::from("hi");
    let borrowed = &text;
    // let moved = text;
    // println!("{borrowed}");
    // error[E0505]: cannot move out of `text` because it is borrowed
    println!("    borrowed = {borrowed}");
    let moved = text; // fine now: the borrow has ended
    println!("    moved = {moved}");

    println!("\n7. Passing a &mut to a function lends it again, briefly (a reborrow)");
    let mut count = 0;
    let handle = &mut count;
    add_one(handle); // lends `*handle` to the function; `handle` is usable again after
    add_one(handle);
    println!("    count = {count}");

    println!("\n8. `v.push(v.len())`: allowed, even though it looks like rule 3 is broken");
    let mut lengths = vec![0];
    lengths.push(lengths.len()); // the argument is read before the mutable borrow starts
    println!("    lengths = {lengths:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn borrows_end_at_their_last_use() {
        let mut v = vec![1];
        let r = &v[0];
        assert_eq!(*r, 1); // last use of `r`
        v.push(2); // so this is allowed
        assert_eq!(v, vec![1, 2]);
    }

    #[test]
    fn reborrowing_keeps_the_mutable_reference_usable() {
        let mut n = 0;
        let r = &mut n;
        add_one(r);
        add_one(r);
        *r += 1;
        assert_eq!(n, 3);
    }

    #[test]
    // Pushing one item at a time into a Vec that's too small is the point here.
    #[allow(clippy::vec_init_then_push)]
    fn pushing_into_a_full_vec_reallocates() {
        let mut v = Vec::with_capacity(1);
        v.push(1);
        v.push(2);
        // The Vec had to get a bigger buffer. Whether the allocator moved it
        // or grew it in place is up to the allocator, so a reference to `v[0]`
        // taken before the push could have been left pointing at freed memory.
        // That possibility is why the borrowing rules forbid it.
        assert!(v.capacity() >= 2);
    }

    #[test]
    fn two_phase_borrow() {
        let mut v = vec![10, 20];
        v.push(v.len() as i32);
        assert_eq!(v, vec![10, 20, 2]);
    }
}
