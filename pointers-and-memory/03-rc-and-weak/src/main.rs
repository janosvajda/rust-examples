// Lesson 3: Rc and Weak, several owners in one thread.
//
// A Box has exactly one owner. Sometimes a value really belongs to several
// parts of a program at once, and none of them knows which will be the last to
// need it. Rc ("reference counted") keeps a count of its owners, and frees the
// value when the count reaches zero. Weak points at the same value without
// owning it, which is how cycles are avoided.

use std::cell::RefCell;
use std::rc::{Rc, Weak};

// ---- 1. Shared ownership ---------------------------------------------------------------------

/// Several widgets share one theme. None of them owns it alone.
struct Theme {
    colour: &'static str,
}

struct Widget {
    name: &'static str,
    theme: Rc<Theme>,
}

// ---- 2. A cycle: two nodes that own each other -------------------------------------------

/// Records which nodes have been dropped (freed), so we can see leaks.
type DropLog = Rc<RefCell<Vec<&'static str>>>;

/// A node that may own the next node: `Rc` inside, so a cycle is possible.
struct Node {
    name: &'static str,
    next: RefCell<Option<Rc<Node>>>,
    log: DropLog,
}

impl Drop for Node {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.name);
    }
}

/// Builds a → b → a, then lets go of both. Returns a Weak to `a`, to check
/// afterwards whether `a` still exists.
fn make_a_cycle(log: &DropLog) -> Weak<Node> {
    let a = Rc::new(Node { name: "a", next: RefCell::new(None), log: Rc::clone(log) });
    let b = Rc::new(Node { name: "b", next: RefCell::new(Some(Rc::clone(&a))), log: Rc::clone(log) });
    *a.next.borrow_mut() = Some(Rc::clone(&b)); // a → b → a: a cycle
    Rc::downgrade(&a)
} // `a` and `b` go out of scope here, but each is still owned by the other

// ---- 3. The fix: Weak for the link that points "back" ------------------------------------

/// A tree: a parent OWNS its children (Rc), a child only POINTS to its parent (Weak).
struct TreeNode {
    name: &'static str,
    parent: RefCell<Weak<TreeNode>>,
    children: RefCell<Vec<Rc<TreeNode>>>,
    log: DropLog,
}

impl Drop for TreeNode {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.name);
    }
}

fn tree_node(name: &'static str, log: &DropLog) -> Rc<TreeNode> {
    Rc::new(TreeNode { name, parent: RefCell::new(Weak::new()), children: RefCell::new(Vec::new()), log: Rc::clone(log) })
}

fn add_child(parent: &Rc<TreeNode>, child: Rc<TreeNode>) {
    *child.parent.borrow_mut() = Rc::downgrade(parent); // points back, doesn't own
    parent.children.borrow_mut().push(child); // owns
}

/// The path from a node up to the root, following the Weak parent links.
fn path_to_root(node: &Rc<TreeNode>) -> Vec<&'static str> {
    let mut path = vec![node.name];
    let mut current = node.parent.borrow().upgrade(); // Weak → Option<Rc>
    while let Some(parent) = current {
        path.push(parent.name);
        current = parent.parent.borrow().upgrade();
    }
    path
}

fn main() {
    println!("1. Shared ownership: three widgets, one theme");
    let theme = Rc::new(Theme { colour: "dark blue" });
    println!("    owners: {}", Rc::strong_count(&theme));
    let widgets: Vec<Widget> = ["button", "menu", "title"]
        .into_iter()
        .map(|name| Widget { name, theme: Rc::clone(&theme) }) // a new owner, NOT a copy of the theme
        .collect();
    println!("    owners after 3 widgets: {}", Rc::strong_count(&theme));
    for w in &widgets {
        println!("    {} uses {}", w.name, w.theme.colour);
    }
    drop(widgets);
    println!("    owners after the widgets are gone: {}", Rc::strong_count(&theme));

    println!("\n2. A cycle leaks");
    let log = DropLog::default();
    let a = make_a_cycle(&log);
    println!("    both handles are gone. dropped so far: {:?}", log.borrow());
    println!("    does `a` still exist? {}", a.upgrade().is_some());
    println!("    owners of `a`: {}  ← owned by `b`, which is owned by `a`…", a.strong_count());

    println!("\n3. Weak back-links: a tree that cleans up");
    let log = DropLog::default();
    let root = tree_node("root", &log);
    let docs = tree_node("docs", &log);
    let readme = tree_node("readme", &log);
    add_child(&docs, Rc::clone(&readme));
    add_child(&root, docs);
    println!("    path from readme: {}", path_to_root(&readme).join(" → "));
    let parent_link = Rc::downgrade(&root);
    drop(root);
    println!("    after dropping root, dropped: {:?}", log.borrow());
    println!("    root still exists: {}", parent_link.upgrade().is_some());
    drop(readme);
    println!("    after dropping our readme handle too: {:?}", log.borrow());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clones_share_one_value() {
        let theme = Rc::new(Theme { colour: "red" });
        let widget = Widget { name: "w", theme: Rc::clone(&theme) };
        assert_eq!(Rc::strong_count(&theme), 2);
        assert!(Rc::ptr_eq(&theme, &widget.theme)); // the same value, not a copy
        drop(widget);
        assert_eq!(Rc::strong_count(&theme), 1);
    }

    #[test]
    fn a_cycle_is_never_freed() {
        let log = DropLog::default();
        let a = make_a_cycle(&log);
        assert!(log.borrow().is_empty(), "nothing was dropped");
        assert!(a.upgrade().is_some(), "and `a` is still alive, with no way to reach it but this Weak");
        // Break the cycle by hand, so the test itself doesn't leak:
        if let Some(a) = a.upgrade() {
            a.next.borrow_mut().take();
        }
        assert_eq!(log.borrow().len(), 2);
    }

    #[test]
    fn weak_parent_links_let_the_tree_be_freed() {
        let log = DropLog::default();
        let root = tree_node("root", &log);
        let child = tree_node("child", &log);
        add_child(&root, Rc::clone(&child));
        assert_eq!(path_to_root(&child), ["child", "root"]);
        assert_eq!((Rc::strong_count(&root), Rc::weak_count(&root)), (1, 1));

        drop(root);
        assert_eq!(*log.borrow(), ["root"]); // freed, although the child points to it
        assert_eq!(path_to_root(&child), ["child"]); // the Weak link now finds nothing
        drop(child);
        assert_eq!(*log.borrow(), ["root", "child"]);
    }
}
