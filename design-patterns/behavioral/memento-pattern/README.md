<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Memento Pattern

## What is it?

The **Memento** pattern saves a snapshot of an object's state, so it can be restored later, without exposing what's inside the snapshot to anyone else.

Think of saving a game. Before the boss fight you save, and if things go badly you load the save and you're back exactly where you were: same health, same items, same position. You don't need to know how the save file is structured. You just keep it and hand it back to the game when you want to return.

The best-known use is **undo**: every Ctrl+Z in an editor goes back to a saved snapshot.

## The parts

```text
  Editor (originator)                            History (caretaker)
  ┌────────────────────┐   save() → Memento    ┌──────────────────┐
  │ text, cursor       │ ────────────────────► │ undo_stack       │
  │                    │ ◄──────────────────── │ redo_stack       │
  └────────────────────┘   restore(Memento)    └──────────────────┘
```

- **Originator** (`Editor`): the object whose state is saved. It's the only one that creates snapshots (`save`) and uses them (`restore`).
- **Memento** (`Memento`): the snapshot. It holds the text and cursor position.
- **Caretaker** (`History`): keeps the snapshots and decides when to hand one back, but never reads or changes what's inside.

## The example

A tiny text editor with undo and redo. The demo types "Hello", then " world", then "!", then presses Backspace six times. Then it undoes, undoes again, and redoes. The cursor is shown as `|`.

How undo and redo work, with two stacks:

| Action | What `History` does |
|---|---|
| Before every edit | push a snapshot of the current state onto the **undo** stack, and clear the **redo** stack |
| Undo | save the current state onto the **redo** stack, then restore the newest snapshot from the **undo** stack |
| Redo | the opposite: save the current state onto **undo**, then restore from **redo** |

Two details that real editors also handle:
- **Making a new edit after undoing clears "redo".** You can't go "forward" into a future that no longer exists. The demo shows this.
- **History is limited** (`max_steps`), so memory doesn't grow forever. The oldest snapshots are forgotten first.

## The Rust way

- **Privacy comes from modules.** The memento's fields aren't `pub`, so if `Editor` and `Memento` lived in their own module, other code could keep and pass around mementos but never read or modify them. That's exactly the guarantee this pattern is about.
- **`restore(memento: Memento)` takes the snapshot by value.** Once restored, it's used up, so it can't accidentally be restored twice by mistake.
- **Snapshots are full copies** (`self.text.clone()`). That's simple and fine for small states. For large documents, real editors store only what changed in each step instead. That's closer to the Command pattern, where each action knows how to undo itself.
- **Text positions:** Rust strings are UTF-8, where "é" takes two bytes. The editor counts the cursor in *characters* and converts to byte positions when editing, so it never cuts a character in half. A test checks this.

## When to use it

- You need undo/redo, checkpoints, or the ability to roll back after a failed operation.
- You want to save an object's state without making its internals public.

Typical examples: editors, drawing apps, games (save points), form wizards with a "back" button, transactions that must be rolled back on error.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p memento-pattern`.
