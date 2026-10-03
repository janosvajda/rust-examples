<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Composite Pattern

## What is it?

The **Composite** pattern lets you build tree structures, where a group of things can be treated exactly like a single thing.

Think of the folders on your computer. A folder can contain files *and* other folders. Yet you can ask a folder the same questions you ask a file: "how big are you?", "copy yourself", "delete yourself". A file answers directly. A folder answers by asking everything inside it and combining the answers. You don't need to care which one you're talking to.

## The parts

```text
  home/                         ◄── composite (a folder)
  ├── notes.txt                 ◄── leaf (a file)
  ├── photos/                   ◄── composite
  │   ├── beach.jpg             ◄── leaf
  │   ├── mountains.jpg
  │   └── 2024/                 ◄── composite inside a composite
  │       └── birthday.jpg
  └── projects/
      ├── rust-examples/
      │   ├── main.rs
      │   └── README.md
      └── empty/                ◄── a composite with no children is fine too
```

- **Component** (`Node`): what both files and folders are, and the operations they all support.
- **Leaf** (`Node::File`): a single item with no children. It answers questions about itself.
- **Composite** (`Node::Folder`): contains other components (leaves or composites) and answers by asking its children.

## The example

`main` builds the folder tree shown above and asks it questions:

| Operation | A file… | A folder… |
|---|---|---|
| `size_kb()` | returns its own size | adds up the sizes of everything inside it |
| `file_count()` | returns 1 | adds up the counts of its children |
| `find_by_extension(".jpg")` | matches its own name | searches every child, at any depth |
| `print()` | prints one line | prints itself, then each child indented |

Because a folder asks its children, and a child folder asks *its* children, a single call on the top folder covers the whole tree. That's recursion, and it's the heart of this pattern.

## The Rust way: an enum

In many languages each kind of node is a class, implementing a shared interface. In Rust the natural fit is an **enum**: `Node` is either a `File` or a `Folder`, and every operation is one `match` that handles both cases.

- **All node kinds are listed in one place**, and the compiler makes sure every operation handles every kind.
- **No `Box<dyn ...>` needed:** a `Vec<Node>` already puts children on the heap, so the recursive type has a known size.

The alternative is a trait (`trait Component { fn size_kb(&self) -> u64; }`) with `Vec<Box<dyn Component>>` children. Choose that when *other code* must be able to add new kinds of node, for example plugins adding new shapes to a drawing.

## When to use it

- Your data is naturally a tree: files and folders, menus and submenus, an organisation chart, shapes grouped into drawings, HTML elements.
- You want callers to treat single items and groups the same way.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p composite-pattern`.
