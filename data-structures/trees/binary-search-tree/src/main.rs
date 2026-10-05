use binary_search_tree::BinarySearchTree;

fn build(values: &[i32]) -> BinarySearchTree<i32> {
    let mut tree = BinarySearchTree::new();
    for &value in values {
        tree.insert(value);
    }
    tree
}

fn main() {
    println!("--- Basic operations ---");
    let mut tree = build(&[8, 3, 10, 1, 6, 14, 4, 7, 13]);
    println!("in order: {:?}", tree.in_order());
    println!("min: {:?}, max: {:?}", tree.min(), tree.max());
    println!(
        "contains 6: {}, contains 5: {}",
        tree.contains(&6),
        tree.contains(&5)
    );

    tree.remove(&3);
    println!("after removing 3: {:?}", tree.in_order());

    println!("\n--- Why balance matters ---");
    // The same 1000 values, inserted in two different orders.
    let sorted: Vec<i32> = (0..1000).collect();
    let shuffled: Vec<i32> = (0..1000).map(|i| (i * 617) % 1000).collect();

    println!(
        "height after inserting 0..1000 in sorted order:   {}",
        build(&sorted).height()
    );
    println!(
        "height after inserting 0..1000 in shuffled order: {}",
        build(&shuffled).height()
    );
    println!("(a perfectly balanced tree of 1000 values has height 10)");
}
