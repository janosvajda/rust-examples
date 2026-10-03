use hash_table::HashTable;

fn main() {
    println!("--- Watching the table grow ---");
    let mut table = HashTable::new();
    let mut last_bucket_count = 0;
    for i in 0..100 {
        table.insert(i, i * 10);
        if table.bucket_count() != last_bucket_count {
            println!(
                "after {:>3} entries: {:>3} buckets (load factor {:.2})",
                table.len(),
                table.bucket_count(),
                table.load_factor()
            );
            last_bucket_count = table.bucket_count();
        }
    }
    println!(
        "final: {} entries, {} buckets, load factor {:.2}, longest chain {}",
        table.len(),
        table.bucket_count(),
        table.load_factor(),
        table.longest_chain()
    );

    println!("\n--- Counting words ---");
    let text = "the quick brown fox jumps over the lazy dog the fox";
    let mut counts = HashTable::new();
    for word in text.split_whitespace() {
        match counts.get_mut(&word) {
            Some(count) => *count += 1,
            None => {
                counts.insert(word, 1);
            }
        }
    }

    // A hash table has no order, so sort the entries before printing.
    let mut sorted: Vec<_> = counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    for (word, count) in sorted {
        println!("{word:<6} {count}");
    }
}
