<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: `Rc<T>` and `Weak<T>`

## The idea in one sentence

`Rc` lets several parts of a program own the same value by **counting** its owners and freeing it when the last one is gone, and `Weak` points at a value **without** owning it, which is what keeps reference cycles from leaking memory.

## Several owners: `Rc`

Three widgets share one theme. None of them owns it alone, and nobody knows which widget will be closed last:

```rust
let theme = Rc::new(Theme { colour: "dark blue" });
let button = Widget { name: "button", theme: Rc::clone(&theme) };   // a new owner, not a copy
```

```text
owners: 1
owners after 3 widgets: 4
owners after the widgets are gone: 1
```

`Rc::clone` doesn't copy the theme. It adds one to the **reference count** and returns another pointer to the same value. Dropping an `Rc` subtracts one, and when the count reaches zero, the value is freed. Write `Rc::clone(&x)` rather than `x.clone()`: it does the same thing, but makes it obvious that only a counter changes.

### `Rc` is read-only

Several owners can't all be allowed to change the value, or they'd trip over each other:

```rust
let shared = Rc::new(String::from("hi"));
shared.push('!');
```

```text
error[E0596]: cannot borrow data in an `Rc` as mutable
```

When shared data really must change, use `Rc<RefCell<T>>`: `RefCell` checks the borrowing rules while the program runs instead of at compile time. [Ownership lesson 7](../../ownership-and-borrowing/07-interior-mutability/) explains it. This lesson's nodes use `RefCell` for their links.

`Rc` only works within one thread, because its counter isn't safe to change from two threads at once. [Lesson 4](../04-arc/) is about the thread-safe version.

## A cycle leaks memory

Here are two nodes, each owning the next, in a circle:

```rust
let a = Rc::new(Node { name: "a", next: RefCell::new(None), … });
let b = Rc::new(Node { name: "b", next: RefCell::new(Some(Rc::clone(&a))), … });
*a.next.borrow_mut() = Some(Rc::clone(&b));      // a → b → a
```

When `a` and `b` go out of scope:

```text
both handles are gone. dropped so far: []
does `a` still exist? true
owners of `a`: 1  ← owned by `b`, which is owned by `a`…
```

Each node's count goes from 2 to 1, never to 0: `a` keeps `b` alive, and `b` keeps `a` alive. Nothing in the program can reach them any more, and they're never freed. That's a **memory leak**, in completely safe Rust.

Rust prevents use-after-free and double-free, but it doesn't promise that memory is always freed. An `Rc` cycle is the classic way to leak, and `std::mem::forget` is the other.

## The fix: `Weak` for the link back

Most "cycles" have a direction: a parent **owns** its children, and a child only needs to **find** its parent. The link back should be a `Weak`:

```rust
struct TreeNode {
    name: &'static str,
    parent: RefCell<Weak<TreeNode>>,          // points back: doesn't own
    children: RefCell<Vec<Rc<TreeNode>>>,     // owns
}
```

A `Weak` doesn't count as an owner, so it doesn't keep the value alive. To use it, call `upgrade()`, which returns `Some(Rc)` if the value still exists, and `None` if it has already been freed:

```rust
let mut current = node.parent.borrow().upgrade();     // Weak → Option<Rc>
while let Some(parent) = current {
    path.push(parent.name);
    current = parent.parent.borrow().upgrade();
}
```

```text
path from readme: readme → docs → root
after dropping root, dropped: ["root", "docs"]
root still exists: false
after dropping our readme handle too: ["root", "docs", "readme"]
```

Dropping `root` frees `root`, and with it `docs`, its only owner. `readme` survives a little longer, because the program still holds a handle to it. Everything is freed in the end.

| | `Rc<T>` | `Weak<T>` |
|---|---|---|
| keeps the value alive | yes | no |
| counted by | `Rc::strong_count` | `Rc::weak_count` |
| using the value | directly | `upgrade()` first, which may give `None` |
| use it for | ownership: "this is mine, too" | references back or across: parent links, caches, observers |

## Proving it with tests

Memory leaks are usually invisible. The tests make them visible with a `Drop` implementation that records which nodes were freed:
- `a_cycle_is_never_freed` shows that after the cycle is out of reach, nothing has been dropped and the node is still alive. Then it breaks the cycle by hand, and both nodes are freed.
- `weak_parent_links_let_the_tree_be_freed` shows that the root is freed even though its child points to it, and that the child's `Weak` link then finds nothing.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: Box](../02-box/) · Next: [Lesson 4: Arc](../04-arc/)
