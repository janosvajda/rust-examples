// Lesson 1: closures.
//
// A closure is an anonymous function you can store in a variable and pass
// around, and that can CAPTURE variables from the place where it's written.
//     |parameters| expression
//     |parameters| { statements }

/// A named operation: a label and a plain function pointer. Giving a
/// complicated type a short name with `type` keeps signatures readable.
type Operation = (&'static str, fn(i32) -> i32);

fn main() {
    println!("1. Closure syntax");
    let add_one = |x: i32| -> i32 { x + 1 }; // fully annotated
    let double = |x| x * 2; // types inferred from use
    let greet = || String::from("hello"); // no parameters
    println!("    add_one(4) = {}, double(4) = {}, greet() = {}", add_one(4), double(4), greet());

    println!("\n2. Closures capture their surroundings; functions can't");
    let tax_rate = 0.27;
    let with_tax = |price: f64| price * (1.0 + tax_rate); // uses `tax_rate` from outside
    println!("    100 with tax = {:.2}", with_tax(100.0));
    // A nested `fn` can't do this:
    //     fn with_tax_fn(price: f64) -> f64 { price * (1.0 + tax_rate) }
    //     error[E0434]: can't capture dynamic environment in a fn item

    println!("\n3. Three ways to capture: borrow, borrow mutably, take ownership");
    let names = vec![String::from("Ana"), String::from("Bob")];
    let count = || names.len(); // only reads → borrows `names`
    println!("    count() = {}, and names is still usable: {names:?}", count());

    let mut log = Vec::new();
    let mut record = |entry: &str| log.push(entry.to_string()); // changes → borrows mutably
    record("started");
    record("finished");
    println!("    log = {log:?}"); // the mutable borrow ended after the last `record`

    let owned = String::from("moved in");
    let keep = move || owned.len(); // `move` → takes ownership
    println!("    keep() = {}", keep());
    // println!("{owned}");   // error[E0382]: borrow of moved value: `owned`

    println!("\n4. Every closure has its own, unique type");
    // let mut list = vec![|x: i32| x + 1];
    // list.push(|x: i32| x + 1);
    //     error[E0308]: mismatched types
    //     = note: no two closures, even if identical, have the same type
    let a = |x: i32| x + 1;
    let b = |x: i32| x + 1;
    println!("    a(1) = {}, b(1) = {}: same code, different types", a(1), b(1));

    println!("\n5. Closures that capture nothing become plain function pointers");
    // `fn(i32) -> i32` is a function pointer type. Closures that don't
    // capture anything convert to it, so they CAN share one type.
    let operations: Vec<Operation> = vec![
        ("double", |x| x * 2),
        ("square", |x| x * x),
        ("negate", |x| -x),
    ];
    for (name, operation) in &operations {
        println!("    {name}(7) = {}", operation(7));
    }

    println!("\n6. Closures as arguments: the most common use");
    let mut words = vec!["banana", "Apple", "cherry"];
    words.sort_by_key(|w| w.to_lowercase()); // the closure says HOW to sort
    let long: Vec<_> = words.iter().filter(|w| w.len() > 5).collect();
    println!("    sorted: {words:?}, longer than 5: {long:?}");
}

#[cfg(test)]
mod tests {
    #[test]
    fn closure_captures_by_reference() {
        let limit = 10;
        let under = |x: i32| x < limit;
        assert!(under(3));
        assert_eq!(limit, 10); // still ours
    }

    #[test]
    fn closure_can_change_captured_state() {
        let mut total = 0;
        let mut add = |x: i32| total += x;
        add(5);
        add(7);
        assert_eq!(total, 12);
    }

    #[test]
    fn move_closure_owns_its_data() {
        let make = || {
            let secret = String::from("inside");
            move || secret.clone() // can outlive `secret`'s original scope
        };
        let reveal = make();
        assert_eq!(reveal(), "inside");
    }

    #[test]
    fn non_capturing_closures_coerce_to_fn_pointers() {
        let pick = |double: bool| -> fn(i32) -> i32 {
            if double { |x| x * 2 } else { |x| x + 1 }
        };
        assert_eq!(pick(true)(5), 10);
        assert_eq!(pick(false)(5), 6);
    }
}
