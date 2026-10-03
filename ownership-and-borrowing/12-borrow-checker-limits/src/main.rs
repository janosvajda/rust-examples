// Lesson 12: where the borrow checker is too strict.
//
// The borrow checker never accepts unsafe code, but it does reject some code
// that would actually be fine. This lesson shows the common cases and the
// standard way to write each one so it compiles.

use std::collections::HashMap;
use std::ops::Range;

// ---- 1. Return a borrow from one branch, change the data in another ----------
//
// This is correct, but Rust 1.99 rejects it:
//
// fn get_or_insert(map: &mut HashMap<u32, String>, key: u32) -> &String {
//     if let Some(value) = map.get(&key) {
//         return value;                 // the borrow is returned to the caller…
//     }
//     map.insert(key, String::new());   // …so the checker thinks it's still active here
//     map.get(&key).unwrap()
// }
// error[E0502]: cannot borrow `*map` as mutable because it is also borrowed as immutable
//
// Because the borrow is *returned* on one path, the current checker treats
// it as lasting for the rest of the function on every path. A future checker
// (called Polonius) is designed to accept this.

/// Workaround: the entry API does the lookup and the insert as one operation,
/// so there's only ever one borrow and one path for the checker to follow.
fn get_or_insert(map: &mut HashMap<u32, String>, key: u32) -> &String {
    map.entry(key).or_insert_with(|| format!("item {key}"))
}

// ---- 2. A struct can't hold a reference into its own field --------------------
//
// struct Parser {
//     text: String,
//     current_word: &str,   // wants to point into `text`
// }
// error[E0106]: missing lifetime specifier
//
// There's no lifetime to write: `text` moves whenever the Parser moves, so a
// reference into it could never be guaranteed valid. Workaround: store
// POSITIONS instead of references, and turn them into slices when needed.

struct Parser {
    text: String,
    current_word: Range<usize>, // byte positions inside `text`
}

impl Parser {
    fn new(text: &str) -> Self {
        let end = text.find(' ').unwrap_or(text.len());
        Parser {
            text: text.to_string(),
            current_word: 0..end,
        }
    }

    fn current_word(&self) -> &str {
        &self.text[self.current_word.clone()]
    }

    fn next_word(&mut self) -> bool {
        let start = self.current_word.end + 1;
        if start > self.text.len() {
            return false;
        }
        let end = self.text[start..].find(' ').map_or(self.text.len(), |i| start + i);
        self.current_word = start..end;
        true
    }
}

// ---- 3. A mutable reference into a collection blocks the whole collection ----
//
// let last = items.last_mut().unwrap();
// items.push(*last);
// error[E0499]: cannot borrow `items` as mutable more than once at a time
//
// Workaround: copy out what you need first, so the borrow ends before the change.

fn duplicate_last(items: &mut Vec<i32>) {
    if let Some(&last) = items.last() {
        items.push(last);
    }
}

// ---- 4. Data that points at itself: use indices (an arena) --------------------
//
// A graph where nodes reference each other can't be built from plain `&`
// references: each node would borrow the others, and nothing could ever be
// changed. Store the nodes in a Vec and link them by index instead.

struct Graph {
    names: Vec<&'static str>,
    edges: Vec<Vec<usize>>, // edges[i] = indices of i's neighbours
}

impl Graph {
    fn add(&mut self, name: &'static str) -> usize {
        self.names.push(name);
        self.edges.push(Vec::new());
        self.names.len() - 1
    }

    fn connect(&mut self, a: usize, b: usize) {
        self.edges[a].push(b);
        self.edges[b].push(a); // cycles are no problem with indices
    }

    fn neighbours(&self, node: usize) -> Vec<&str> {
        self.edges[node].iter().map(|&i| self.names[i]).collect()
    }
}

fn main() {
    println!("1. Get or insert: the entry API");
    let mut cache = HashMap::new();
    println!("    {}", get_or_insert(&mut cache, 7));
    println!("    {}", get_or_insert(&mut cache, 8));
    println!("    {}", get_or_insert(&mut cache, 7)); // already there: not inserted again
    println!("    cache has {} entries", cache.len());

    println!("\n2. Self-reference: store positions, not references");
    let mut parser = Parser::new("borrow checker limits");
    loop {
        println!("    word: {}", parser.current_word());
        if !parser.next_word() {
            break;
        }
    }

    println!("\n3. Copy out before changing");
    let mut items = vec![1, 2, 3];
    duplicate_last(&mut items);
    println!("    {items:?}");

    println!("\n4. Cycles: indices instead of references");
    let mut graph = Graph { names: vec![], edges: vec![] };
    let (a, b, c) = (graph.add("Budapest"), graph.add("Vienna"), graph.add("Prague"));
    graph.connect(a, b);
    graph.connect(b, c);
    graph.connect(c, a); // a cycle
    println!("    Vienna's neighbours: {:?}", graph.neighbours(b));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_or_insert_only_inserts_once() {
        let mut map = HashMap::new();
        map.insert(1, String::from("existing"));
        assert_eq!(get_or_insert(&mut map, 1), "existing");
        assert_eq!(get_or_insert(&mut map, 2), "item 2");
        assert_eq!(get_or_insert(&mut map, 2), "item 2");
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn parser_walks_through_words() {
        let mut p = Parser::new("a bb ccc");
        let mut words = vec![p.current_word().to_string()];
        while p.next_word() {
            words.push(p.current_word().to_string());
        }
        assert_eq!(words, vec!["a", "bb", "ccc"]);
    }

    #[test]
    fn parser_survives_being_moved() {
        // The reason positions work: moving the Parser moves `text`, but the
        // positions stay correct relative to it.
        let p = Parser::new("moved around");
        let boxed = Box::new(p);
        assert_eq!(boxed.current_word(), "moved");
    }

    #[test]
    fn duplicate_last_handles_empty() {
        let mut empty: Vec<i32> = vec![];
        duplicate_last(&mut empty);
        assert!(empty.is_empty());
    }

    #[test]
    fn graph_with_a_cycle() {
        let mut g = Graph { names: vec![], edges: vec![] };
        let x = g.add("x");
        let y = g.add("y");
        g.connect(x, y);
        g.connect(y, x);
        assert_eq!(g.neighbours(x), vec!["y", "y"]);
    }
}
