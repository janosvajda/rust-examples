use avl_tree::AvlTree;
// The plain, unbalanced tree from the binary-search-tree example, for comparison.
use binary_search_tree::BinarySearchTree;

fn main() {
    println!("--- Watching rotations happen ---");
    println!("(the tree is drawn sideways: root on the left, tilt your head left)\n");
    let mut tree = AvlTree::new();
    for value in [10, 20, 30, 40, 50, 25] {
        tree.insert(value);
        println!("after inserting {value}:\n{}", tree.draw());
    }

    println!("--- AVL tree vs plain binary search tree ---");
    let mut avl = AvlTree::new();
    let mut bst = BinarySearchTree::new();
    for value in 0..1000 {
        avl.insert(value);
        bst.insert(value);
    }
    println!("height after inserting 0..1000 in sorted order:");
    println!("  plain BST: {}", bst.height());
    println!("  AVL tree:  {}", avl.height());
}
