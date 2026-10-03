use trie::Trie;

fn main() {
    let mut trie = Trie::new();
    let dictionary = [
        "rust", "rustacean", "rusty", "ruby", "run", "runtime", "cargo", "crate", "crates",
    ];
    for word in dictionary {
        trie.insert(word);
    }
    println!("stored {} words", trie.len());

    println!("\n--- Autocomplete ---");
    for prefix in ["ru", "rust", "cra", "go"] {
        println!("{prefix:<5} -> {:?}", trie.words_with_prefix(prefix));
    }

    println!("\n--- Word vs prefix ---");
    for text in ["run", "ru", "crates", "crab"] {
        println!(
            "{text:<7} is a word: {:<5}  is a prefix: {}",
            trie.contains(text),
            trie.starts_with(text)
        );
    }
}
