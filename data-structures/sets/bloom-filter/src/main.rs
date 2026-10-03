use bloom_filter::BloomFilter;
use std::collections::HashSet;

fn main() {
    // A username sign-up form: before asking the (slow) database whether a
    // name is taken, ask the Bloom filter. "Definitely not taken" means we can
    // skip the database entirely.
    let taken: Vec<String> = (0..100_000).map(|i| format!("user{i}")).collect();

    let mut filter = BloomFilter::new(taken.len(), 0.01);
    for name in &taken {
        filter.insert(name.as_str());
    }

    println!(
        "{} names stored in {} bits ({} KB) using {} hash functions",
        filter.items_added(),
        filter.bit_count(),
        filter.bit_count() / 8 / 1024,
        filter.hash_count()
    );
    println!(
        "estimated false positive rate: {:.2}%",
        filter.estimated_false_positive_rate() * 100.0
    );

    println!();
    for name in ["user42", "user99999", "ferris", "crab_fan", "rustacean"] {
        let answer = if filter.might_contain(name) {
            "probably taken → ask the database"
        } else {
            "definitely free → skip the database"
        };
        println!("{name:<10} {answer}");
    }

    // Measure the real rate against an exact HashSet.
    let exact: HashSet<&str> = taken.iter().map(String::as_str).collect();
    let candidates: Vec<String> = (0..100_000).map(|i| format!("guest{i}")).collect();
    let false_positives = candidates
        .iter()
        .filter(|name| filter.might_contain(name.as_str()) && !exact.contains(name.as_str()))
        .count();
    println!(
        "\nmeasured: {false_positives} false positives out of {} names that were never added ({:.2}%)",
        candidates.len(),
        false_positives as f64 / candidates.len() as f64 * 100.0
    );
}
