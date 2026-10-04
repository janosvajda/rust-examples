// Lesson 3: the borrowing rules.
//
// Ordinary shared and exclusive accesses to the same region must not conflict.
// Different fields or disjoint slices can have separate mutable borrows.
// Reborrowing temporarily limits the original reference's access; interior
// mutability types provide controlled changes through shared references.
//
// A borrow lasts for every use that needs it. In simple examples that means
// the reference's last use, but copied references or destructors can extend it.

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
    grow.resize(grow.capacity(), 10); // fill the actual reported capacity
    let (capacity_before, address_before) = (grow.capacity(), grow.as_ptr());
    grow.push(20); // full: capacity must grow; the allocator may grow in place
    println!(
        "    capacity {} → {}; the items {} to a new address",
        capacity_before,
        grow.capacity(),
        if grow.as_ptr() != address_before {
            "moved"
        } else {
            "did not move"
        }
    );
    println!("    the compiler must assume they CAN move, so it forbids the reference");

    println!("\n5. Pushing to a Vec while iterating over &items: not allowed");
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
    lengths.push(lengths.len()); // shared read during reservation, before exclusive activation
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
    fn pushing_into_a_full_vec_increases_capacity() {
        let mut v = Vec::with_capacity(1);
        v.resize(v.capacity(), 1); // with_capacity guarantees at least the request
        let capacity_before = v.capacity();
        v.push(2);
        // Capacity grows because the actual buffer was full. The address may
        // stay the same if the allocator can grow that allocation in place.
        assert!(v.capacity() > capacity_before);
    }

    #[test]
    fn two_phase_borrow() {
        let mut v = vec![10, 20];
        v.push(v.len() as i32);
        assert_eq!(v, vec![10, 20, 2]);
    }
}
