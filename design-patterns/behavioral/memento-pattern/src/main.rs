// Memento pattern: undo and redo in a text editor. Before each change, the
// editor saves a snapshot of its state (a "memento"). A separate history
// object keeps the snapshots, and can hand one back to restore the editor,
// without ever looking inside it.

// ---- The originator: the object whose state we save --------------------------

struct Editor {
    text: String,
    cursor: usize, // position in the text, in characters
}

/// A snapshot of the editor. Its fields are private to this file, so code
/// elsewhere could store and pass mementos around but never read or change
/// what's inside. Only the editor creates and uses them.
#[derive(Debug, Clone, PartialEq)]
struct Memento {
    text: String,
    cursor: usize,
}

impl Editor {
    fn new() -> Self {
        Editor {
            text: String::new(),
            cursor: 0,
        }
    }

    /// Inserts text at the cursor and moves the cursor after it.
    fn type_text(&mut self, new_text: &str) {
        let byte_index = self.byte_index(self.cursor);
        self.text.insert_str(byte_index, new_text);
        self.cursor += new_text.chars().count();
    }

    /// Deletes up to `count` characters before the cursor, like Backspace.
    fn backspace(&mut self, count: usize) {
        let count = count.min(self.cursor);
        let start = self.byte_index(self.cursor - count);
        let end = self.byte_index(self.cursor);
        self.text.replace_range(start..end, "");
        self.cursor -= count;
    }

    fn move_cursor_to(&mut self, position: usize) {
        self.cursor = position.min(self.text.chars().count());
    }

    /// Takes a snapshot of the current state.
    fn save(&self) -> Memento {
        Memento {
            text: self.text.clone(),
            cursor: self.cursor,
        }
    }

    /// Puts the editor back exactly as it was when the snapshot was taken.
    fn restore(&mut self, memento: Memento) {
        self.text = memento.text;
        self.cursor = memento.cursor;
    }

    /// Shows the text with a `|` where the cursor is.
    fn show(&self) -> String {
        let byte_index = self.byte_index(self.cursor);
        format!(
            "\"{}|{}\"",
            &self.text[..byte_index],
            &self.text[byte_index..]
        )
    }

    /// Converts a position in characters to a position in bytes. Rust strings
    /// are UTF-8, where a character like "é" takes more than one byte.
    fn byte_index(&self, char_position: usize) -> usize {
        self.text
            .char_indices()
            .nth(char_position)
            .map_or(self.text.len(), |(index, _)| index)
    }
}

// ---- The caretaker: stores snapshots, never looks inside them -------------------

/// Two stacks of snapshots: one to go back, one to go forward again.
struct History {
    undo_stack: Vec<Memento>,
    redo_stack: Vec<Memento>,
    max_steps: usize,
}

impl History {
    fn new(max_steps: usize) -> Self {
        History {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_steps,
        }
    }

    /// Call before every change, with a snapshot of the state *before* it.
    fn record(&mut self, before: Memento) {
        self.undo_stack.push(before);
        // Keep memory bounded: forget the oldest step when there are too many.
        if self.undo_stack.len() > self.max_steps {
            self.undo_stack.remove(0);
        }
        // A new change makes the old "redo" future impossible.
        self.redo_stack.clear();
    }

    /// Goes back one step. Returns `false` if there's nothing to undo.
    fn undo(&mut self, editor: &mut Editor) -> bool {
        match self.undo_stack.pop() {
            Some(previous) => {
                self.redo_stack.push(editor.save());
                editor.restore(previous);
                true
            }
            None => false,
        }
    }

    /// Re-applies a step that was undone. Returns `false` if there's nothing to redo.
    fn redo(&mut self, editor: &mut Editor) -> bool {
        match self.redo_stack.pop() {
            Some(next) => {
                self.undo_stack.push(editor.save());
                editor.restore(next);
                true
            }
            None => false,
        }
    }
}

/// One step of the demo: a label to print, and the change to make.
type Edit = (&'static str, fn(&mut Editor));

fn main() {
    let mut editor = Editor::new();
    let mut history = History::new(50);

    // Each edit: record a snapshot first, then change the text.
    let edits: [Edit; 4] = [
        ("type \"Hello\"", |e| e.type_text("Hello")),
        ("type \" world\"", |e| e.type_text(" world")),
        ("type \"!\"", |e| e.type_text("!")),
        ("backspace × 6", |e| e.backspace(6)),
    ];
    for (label, edit) in edits {
        history.record(editor.save());
        edit(&mut editor);
        println!("{label:<16} → {}", editor.show());
    }

    println!();
    history.undo(&mut editor);
    println!("{:<16} → {}", "undo", editor.show());
    history.undo(&mut editor);
    println!("{:<16} → {}", "undo", editor.show());
    history.redo(&mut editor);
    println!("{:<16} → {}", "redo", editor.show());

    // A new edit after undoing clears the redo history.
    history.record(editor.save());
    editor.move_cursor_to(5);
    editor.type_text(",");
    println!("{:<16} → {}", "insert \",\"", editor.show());
    if !history.redo(&mut editor) {
        println!(
            "{:<16} → nothing to redo (the new edit replaced that future)",
            "redo"
        );
    }

    while history.undo(&mut editor) {}
    println!("{:<16} → {}", "undo everything", editor.show());
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Applies an edit the way the app does: snapshot first, then change.
    fn edit(editor: &mut Editor, history: &mut History, change: impl FnOnce(&mut Editor)) {
        history.record(editor.save());
        change(editor);
    }

    #[test]
    fn undo_restores_the_previous_state() {
        let mut editor = Editor::new();
        let mut history = History::new(10);
        edit(&mut editor, &mut history, |e| e.type_text("abc"));
        edit(&mut editor, &mut history, |e| e.backspace(1));
        assert_eq!(editor.text, "ab");

        assert!(history.undo(&mut editor));
        assert_eq!(editor.text, "abc");
        assert!(history.undo(&mut editor));
        assert_eq!(editor.text, "");
        assert!(!history.undo(&mut editor)); // nothing left
    }

    #[test]
    fn redo_reapplies_what_was_undone() {
        let mut editor = Editor::new();
        let mut history = History::new(10);
        edit(&mut editor, &mut history, |e| e.type_text("abc"));
        history.undo(&mut editor);
        assert!(history.redo(&mut editor));
        assert_eq!(editor.text, "abc");
        assert_eq!(editor.cursor, 3);
    }

    #[test]
    fn a_new_edit_clears_redo() {
        let mut editor = Editor::new();
        let mut history = History::new(10);
        edit(&mut editor, &mut history, |e| e.type_text("a"));
        history.undo(&mut editor);
        edit(&mut editor, &mut history, |e| e.type_text("b"));
        assert!(!history.redo(&mut editor));
    }

    #[test]
    fn history_size_is_limited() {
        let mut editor = Editor::new();
        let mut history = History::new(3);
        for _ in 0..10 {
            edit(&mut editor, &mut history, |e| e.type_text("x"));
        }
        let mut undos = 0;
        while history.undo(&mut editor) {
            undos += 1;
        }
        assert_eq!(undos, 3);
        assert_eq!(editor.text, "xxxxxxx"); // only the last 3 steps were undone
    }

    #[test]
    fn handles_multi_byte_characters() {
        let mut editor = Editor::new();
        editor.type_text("café");
        editor.backspace(1);
        assert_eq!(editor.text, "caf");
        editor.move_cursor_to(0);
        editor.type_text("é");
        assert_eq!(editor.show(), "\"é|caf\"");
    }
}
