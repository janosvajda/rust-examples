use stack::{Stack, is_balanced};

fn main() {
    println!("--- Basic push / pop ---");
    let mut stack = Stack::new();
    for page in ["home", "products", "product #42"] {
        println!("visit {page:?}");
        stack.push(page);
    }
    // The "back" button of a browser is a stack: it returns to the most recent page first.
    while let Some(page) = stack.pop() {
        println!("back from {page:?}");
    }

    println!("\n--- Memory ---");
    println!(
        "a Stack<u64> is {} bytes: capacity, pointer and length (the items live on the heap)",
        std::mem::size_of::<Stack<u64>>()
    );
    let mut numbers = Stack::with_capacity(1000);
    println!("with_capacity(1000): capacity {} before any push", numbers.capacity());
    // `extend` reserves once and writes in a tight, vectorised loop.
    numbers.extend(1..=1000u64);
    println!("after extend(1..=1000): len {}, top {:?}", numbers.len(), numbers.peek());

    println!("\n--- Balanced brackets ---");
    for text in ["(a + b) * [c - d]", "{ [ ( ) ] }", "(]", "((("] {
        println!("{text:<20} balanced: {}", is_balanced(text));
    }
}
