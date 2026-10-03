// Lesson 7: interior mutability.
//
// Normally you can only change a value through `&mut`. Interior mutability
// lets you change it through a shared `&` reference, safely, by moving the
// "one writer OR many readers" check from compile time to run time.
//
//   Cell<T>     for small Copy values: you swap whole values in and out,
//               never hand out references to the inside. No runtime check needed.
//   RefCell<T>  for any value: hands out borrows, and counts them at runtime.
//               Breaking the rule panics instead of failing to compile.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

// ---- Cell: a counter that changes inside a `&self` method --------------------

struct Document {
    text: String,
    // Reading a document shouldn't need `&mut`, but we still want to count reads.
    views: Cell<u32>,
}

impl Document {
    fn read(&self) -> &str {
        self.views.set(self.views.get() + 1); // changes through `&self`
        &self.text
    }
}

// ---- RefCell: a log that many parts of the program can append to -----------

struct Logger {
    lines: RefCell<Vec<String>>,
}

impl Logger {
    fn log(&self, message: &str) {
        // `borrow_mut()` is the runtime version of `&mut`. The borrow ends
        // when the returned guard is dropped, here at the end of the line.
        self.lines.borrow_mut().push(message.to_string());
    }

    fn count(&self) -> usize {
        self.lines.borrow().len() // the runtime version of `&`
    }
}

// ---- Rc<RefCell<T>>: shared ownership AND shared mutation ------------------

#[derive(Debug)]
struct Account {
    balance: i64,
}

fn main() {
    println!("1. Cell: change a small value through a shared reference");
    let doc = Document { text: String::from("Rust book"), views: Cell::new(0) };
    let shared = &doc; // only a shared reference
    shared.read();
    shared.read();
    println!("    \"{}\" has been read {} times", shared.read(), doc.views.get());

    println!("\n2. RefCell: borrow rules checked while the program runs");
    let logger = Logger { lines: RefCell::new(Vec::new()) };
    let a = &logger;
    let b = &logger; // two shared references, both can log
    a.log("started");
    b.log("loaded config");
    println!("    {} lines logged: {:?}", logger.count(), logger.lines.borrow());

    println!("\n3. Breaking the rule at runtime: try_borrow_mut");
    let cell = RefCell::new(5);
    {
        let reader = cell.borrow(); // a runtime shared borrow is active…
        println!("    reading {}", *reader);
        // cell.borrow_mut() here would panic: "RefCell already borrowed"
        let attempt = cell.try_borrow_mut(); // …so a mutable one is refused
        println!("    try_borrow_mut while reading: {}", if attempt.is_err() { "refused" } else { "allowed" });
    } // `reader` dropped: the shared borrow ends
    *cell.borrow_mut() += 1;
    println!("    after the reader is gone: {}", cell.borrow());

    println!("\n4. Rc<RefCell<T>>: two owners that can both change the value");
    let account = Rc::new(RefCell::new(Account { balance: 100 }));
    let alice = Rc::clone(&account); // Rc: shared ownership
    let bob = Rc::clone(&account);
    alice.borrow_mut().balance -= 30; // RefCell: shared mutation
    bob.borrow_mut().balance += 50;
    println!(
        "    balance = {} (owners: {})",
        account.borrow().balance,
        Rc::strong_count(&account)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_changes_through_shared_reference() {
        let doc = Document { text: String::from("x"), views: Cell::new(0) };
        let r = &doc;
        r.read();
        r.read();
        assert_eq!(doc.views.get(), 2);
    }

    #[test]
    fn refcell_allows_many_readers() {
        let cell = RefCell::new(1);
        let a = cell.borrow();
        let b = cell.borrow();
        assert_eq!(*a + *b, 2);
    }

    #[test]
    fn refcell_refuses_a_writer_while_reading() {
        let cell = RefCell::new(1);
        let _reader = cell.borrow();
        assert!(cell.try_borrow_mut().is_err());
    }

    #[test]
    fn refcell_refuses_two_writers() {
        let cell = RefCell::new(1);
        let _writer = cell.borrow_mut();
        assert!(cell.try_borrow_mut().is_err());
        assert!(cell.try_borrow().is_err());
    }

    #[test]
    #[should_panic(expected = "already borrowed")]
    fn breaking_the_rule_with_borrow_mut_panics() {
        let cell = RefCell::new(1);
        let _reader = cell.borrow();
        let _writer = cell.borrow_mut(); // panics
    }

    #[test]
    fn rc_refcell_shares_changes_between_owners() {
        let shared = Rc::new(RefCell::new(Vec::new()));
        let other = Rc::clone(&shared);
        other.borrow_mut().push(1);
        shared.borrow_mut().push(2);
        assert_eq!(*shared.borrow(), vec![1, 2]);
    }
}
