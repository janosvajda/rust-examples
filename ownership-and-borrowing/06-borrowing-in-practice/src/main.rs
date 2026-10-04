// Lesson 6: borrowing in practice.
//
// The rules from lesson 3 never change, but real code meets them in a few
// typical situations. Each section shows one, and the usual way to handle it.

use std::collections::HashMap;
use std::mem;

#[derive(Debug)]
struct Book {
    title: String,
    pages: u32,
}

impl Book {
    fn pages(&self) -> u32 {
        self.pages
    }
}

#[derive(Debug, Default)]
struct Order {
    items: Vec<String>,
}

/// Takes the items out of an order through a `&mut`, leaving it empty.
///
/// `order.items` can't simply be moved out: the order is only borrowed, and
/// its owner still expects a valid `Vec` there afterwards.
///     error[E0507]: cannot move out of `order.items` which is behind a mutable reference
/// `mem::take` swaps in an empty `Vec` and hands back the old one.
fn take_items(order: &mut Order) -> Vec<String> {
    mem::take(&mut order.items)
}

/// Move money between two accounts. Two `&mut` into the same HashMap at once
/// would be refused by separate get_mut calls; get_disjoint_mut obtains both.
/// This wrapper rejects equal keys and checks both new balances before changing
/// either account. Every returned Err leaves the balances unchanged.
fn transfer(
    balances: &mut HashMap<&str, u32>,
    from: &str,
    to: &str,
    amount: u32,
) -> Result<(), String> {
    if from == to {
        return Err(String::from("choose two different accounts"));
    }
    let [Some(source), Some(target)] = balances.get_disjoint_mut([from, to]) else {
        return Err(format!("unknown account: `{from}` or `{to}`"));
    };
    let new_source = source
        .checked_sub(amount)
        .ok_or_else(|| format!("`{from}` can't pay {amount}"))?;
    let new_target = target
        .checked_add(amount)
        .ok_or_else(|| format!("`{to}` can't hold another {amount}"))?;
    *source = new_source;
    *target = new_target;
    Ok(())
}

fn main() {
    println!("1. Different fields of a struct can be borrowed separately");
    let mut book = Book {
        title: String::from("Dune"),
        pages: 412,
    };
    let title = &mut book.title;
    let pages = &mut book.pages; // fine: a different field
    title.push_str(" Messiah");
    *pages = 256;
    println!("    {book:?}");

    println!("\n2. …but a method call borrows the whole struct");
    let title = &mut book.title;
    // let count = book.pages();
    // title.push('!');
    // error[E0502]: cannot borrow `book` as immutable because it is also borrowed as mutable
    title.push('!');
    let count = book.pages(); // fine: `title` is finished
    println!("    {} has {count} pages", book.title);
    // A method's signature (`&self`) says it may read ANY field, so the
    // compiler can't let it run while one field is mutably borrowed.

    println!("\n3. Two mutable parts of one collection: split_at_mut, get_disjoint_mut");
    let mut temperatures = [18, 21, 19, 25, 30, 28];
    let (morning, afternoon) = temperatures.split_at_mut(3);
    // `&mut temperatures[..3]` and `&mut temperatures[3..]` at the same time
    // wouldn't compile: the compiler doesn't reason about index ranges.
    // `split_at_mut` guarantees the two halves don't overlap.
    morning[0] += afternoon[0];
    println!("    {temperatures:?}");

    // get_disjoint_mut: mutable access to several positions at once, wherever they are.
    let mut scores = [10, 20, 30, 40];
    if let Ok([first, last]) = scores.get_disjoint_mut([0, 3]) {
        mem::swap(first, last);
    }
    println!("    swapped first and last: {scores:?}");
    println!(
        "    same index twice:       {:?}",
        scores.get_disjoint_mut([1, 1]).map(|_| ())
    );

    let mut balances = HashMap::from([("ana", 100), ("bob", 50)]);
    println!(
        "    transfer 30 ana → bob:  {:?}",
        transfer(&mut balances, "ana", "bob", 30)
    );
    println!(
        "    balances: ana {}, bob {}",
        balances["ana"], balances["bob"]
    );
    println!(
        "    transfer 30 ana → zoe:  {:?}",
        transfer(&mut balances, "ana", "zoe", 30)
    );

    println!("\n4. Iterating: borrow, borrow mutably, or take ownership");
    let mut names = vec![String::from("ana"), String::from("bob")];
    for name in &names {
        // `&names`, same as `names.iter()`: each `name` is `&String`
        print!("    {name}");
    }
    println!();
    for name in &mut names {
        // `names.iter_mut()`: each `name` is `&mut String`
        name.make_ascii_uppercase();
    }
    println!("    after iter_mut: {names:?}");
    for name in names {
        // `names.into_iter()`: each `name` is a `String`, moved out of the Vec
        print!("    {}", name.len());
    }
    println!();
    // println!("{names:?}");   // error[E0382]: the loop moved `names`

    println!("\n5. Closures borrow what they use");
    let mut count = 0;
    let mut increment = || count += 1; // borrows `count` mutably for as long as `increment` is used
    // println!("{count}");
    // increment();
    // error[E0502]: cannot borrow `count` as immutable because it is also borrowed as mutable
    increment();
    increment();
    println!("    count = {count}"); // fine: `increment` is no longer used

    let greeting = String::from("hello");
    let shout = move || greeting.to_uppercase(); // `move`: the closure takes ownership
    println!("    {}", shout());
    // println!("{greeting}"); // error[E0382]: borrow of moved value: `greeting`

    println!("\n6. Taking a value out from behind a &mut: mem::take");
    let mut order = Order {
        items: vec![String::from("coffee"), String::from("cake")],
    };
    let items = take_items(&mut order);
    println!("    took {items:?}, the order now has {:?}", order.items);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn separate_fields_can_be_changed_at_once() {
        let mut book = Book {
            title: String::from("A"),
            pages: 1,
        };
        let (title, pages) = (&mut book.title, &mut book.pages);
        title.push('B');
        *pages += 1;
        assert_eq!(book.title, "AB");
        assert_eq!(book.pages, 2);
    }

    #[test]
    fn split_at_mut_gives_two_independent_halves() {
        let mut data = [1, 2, 3, 4];
        let (left, right) = data.split_at_mut(2);
        left[0] = 10;
        right[1] = 40;
        assert_eq!(data, [10, 2, 3, 40]);
    }

    #[test]
    fn get_disjoint_mut_checks_the_indexes() {
        let mut data = [1, 2, 3];
        let [a, c] = data
            .get_disjoint_mut([0, 2])
            .expect("different, in-bounds indexes");
        mem::swap(a, c);
        assert_eq!(data, [3, 2, 1]);
        assert!(data.get_disjoint_mut([1, 1]).is_err()); // the same index twice
        assert!(data.get_disjoint_mut([0, 9]).is_err()); // out of bounds
    }

    #[test]
    fn transfer_between_two_accounts() {
        let mut balances = HashMap::from([("ana", 100), ("bob", 50)]);
        assert_eq!(transfer(&mut balances, "ana", "bob", 30), Ok(()));
        assert_eq!((balances["ana"], balances["bob"]), (70, 80));
        assert!(transfer(&mut balances, "ana", "bob", 500).is_err()); // not enough money
        assert!(transfer(&mut balances, "ana", "zoe", 1).is_err()); // no such account
    }

    #[test]
    fn failed_transfers_leave_both_balances_unchanged() {
        let cases = [
            ("ana", "bob", 2), // source cannot pay
            ("ana", "bob", 1), // target would overflow
            ("ana", "ana", 0), // overlapping accounts
            ("ana", "zoe", 1), // missing target
            ("zoe", "bob", 1), // missing source
        ];
        for (from, to, amount) in cases {
            let mut balances = HashMap::from([("ana", 1), ("bob", u32::MAX)]);
            let before = balances.clone();
            assert!(transfer(&mut balances, from, to, amount).is_err());
            assert_eq!(balances, before, "failed transfer {from} -> {to}");
        }
    }

    #[test]
    fn transfer_can_reach_the_maximum_balance_exactly() {
        let mut balances = HashMap::from([("ana", 1), ("bob", u32::MAX - 1)]);
        assert_eq!(transfer(&mut balances, "ana", "bob", 1), Ok(()));
        assert_eq!((balances["ana"], balances["bob"]), (0, u32::MAX));
    }

    #[test]
    fn iter_mut_changes_items_in_place() {
        let mut v = vec![1, 2, 3];
        for x in &mut v {
            *x *= 10;
        }
        assert_eq!(v, vec![10, 20, 30]);
    }

    #[test]
    fn mem_take_leaves_an_empty_value_behind() {
        let mut order = Order {
            items: vec![String::from("x")],
        };
        assert_eq!(take_items(&mut order), vec!["x"]);
        assert!(order.items.is_empty());
    }

    #[test]
    fn move_closure_owns_its_data() {
        let make = || {
            let data = String::from("abc");
            move || data.len() // the closure can outlive this block: it owns `data`
        };
        let len = make();
        assert_eq!(len(), 3);
    }
}
