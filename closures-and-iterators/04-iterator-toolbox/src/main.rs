// Lesson 4: the iterator toolbox.
//
// An iterator chain has three parts:
//   a SOURCE       where the items come from: .iter(), .into_iter(), a range…
//   ADAPTERS       lazy steps that transform the stream: map, filter, zip…
//   a CONSUMER     the step that actually runs everything: collect, sum, for…
// Nothing happens until the consumer asks for items.

use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct Sale {
    product: &'static str,
    region: &'static str,
    amount: u32,
}

fn sales() -> Vec<Sale> {
    vec![
        Sale { product: "coffee", region: "north", amount: 120 },
        Sale { product: "tea", region: "south", amount: 80 },
        Sale { product: "coffee", region: "south", amount: 200 },
        Sale { product: "cake", region: "north", amount: 50 },
        Sale { product: "tea", region: "north", amount: 30 },
        Sale { product: "coffee", region: "north", amount: 90 },
    ]
}

fn main() {
    let sales = sales();

    println!("1. Adapters are lazy");
    let mut steps = Vec::new();
    let chain = (1..=3).map(|n| {
        steps.push(format!("map {n}"));
        n * 10
    });
    // Nothing has run yet: `steps` is empty. A chain with no consumer at all
    // gets a warning:
    //     warning: unused `Map` that must be used
    //     = note: iterators are lazy and do nothing unless consumed
    let result: Vec<i32> = chain.collect(); // NOW it runs
    println!("    result {result:?} after steps {steps:?}");

    println!("\n2. Transforming: map, filter, filter_map, flat_map");
    let big: Vec<u32> = sales.iter().map(|s| s.amount).filter(|&a| a >= 100).collect();
    let numbers: Vec<i32> = ["3", "x", "7"].iter().filter_map(|s| s.parse().ok()).collect();
    let letters: Vec<char> = ["ab", "cd"].iter().flat_map(|s| s.chars()).collect();
    println!("    amounts ≥ 100: {big:?}; parsed: {numbers:?}; letters: {letters:?}");

    println!("\n3. Position and grouping: enumerate, zip, windows, chunks");
    for (rank, sale) in sales.iter().take(2).enumerate() {
        println!("    #{} {} {}", rank + 1, sale.product, sale.amount);
    }
    let days = ["Mon", "Tue", "Wed"];
    let visitors = [120, 95, 140];
    let paired: Vec<_> = days.iter().zip(visitors.iter()).collect();
    let changes: Vec<i32> = visitors.windows(2).map(|w| w[1] - w[0]).collect();
    let batches: Vec<_> = [1, 2, 3, 4, 5].chunks(2).map(|c| c.to_vec()).collect();
    println!("    zip: {paired:?}");
    println!("    day-to-day change (windows): {changes:?}; batches (chunks): {batches:?}");
    // array_windows: the same windows, as fixed-size arrays you can destructure
    let rising: Vec<bool> = visitors.array_windows().map(|[before, after]| after > before).collect();
    println!("    visitors went up (array_windows): {rising:?}");

    println!("\n4. Cutting the stream: take, skip, take_while, skip_while, step_by");
    let evens: Vec<u32> = (0..).step_by(2).take(5).collect(); // an endless range, cut short
    let until_big: Vec<u32> = sales.iter().map(|s| s.amount).take_while(|&a| a < 150).collect();
    println!("    first five evens: {evens:?}; amounts until the first ≥ 150: {until_big:?}");

    println!("\n5. Answering questions: sum, count, min/max, any/all, find, position");
    let total: u32 = sales.iter().map(|s| s.amount).sum();
    let coffee_sales = sales.iter().filter(|s| s.product == "coffee").count();
    let biggest = sales.iter().max_by_key(|s| s.amount).unwrap();
    let any_huge = sales.iter().any(|s| s.amount > 500);
    let all_positive = sales.iter().all(|s| s.amount > 0);
    let first_tea = sales.iter().position(|s| s.product == "tea");
    println!("    total {total}, coffee sales {coffee_sales}, biggest {} {}", biggest.product, biggest.amount);
    println!("    any > 500: {any_huge}, all > 0: {all_positive}, first tea at index {first_tea:?}");

    println!("\n6. Building up a result: fold");
    let (count, sum) = sales.iter().fold((0, 0), |(c, s), sale| (c + 1, s + sale.amount));
    println!("    fold → {count} sales, average {}", sum / count);

    println!("\n7. collect into many kinds of collection");
    let products: HashSet<&str> = sales.iter().map(|s| s.product).collect();
    let mut by_region: HashMap<&str, u32> = HashMap::new();
    for sale in &sales {
        *by_region.entry(sale.region).or_insert(0) += sale.amount;
    }
    let sentence: String = ["iterators", "are", "lazy"].join(" ");
    let shouted: String = sentence.chars().map(|c| c.to_ascii_uppercase()).collect();
    let (north, south): (Vec<&Sale>, Vec<&Sale>) = sales.iter().partition(|s| s.region == "north");
    println!("    unique products: {}", products.len());
    println!("    total by region: north {}, south {}", by_region["north"], by_region["south"]);
    println!("    String from chars: {shouted}");
    println!("    partition: {} north, {} south", north.len(), south.len());
    // `collect` needs to know WHAT to build:
    //     let doubled = v.iter().map(|x| x * 2).collect();
    //     error[E0283]: type annotations needed

    println!("\n8. Combining streams: chain, rev, peekable");
    let all: Vec<i32> = (1..=2).chain(8..=9).rev().collect();
    println!("    chain + rev: {all:?}");
    let mut tokens = "12 + 30".split_whitespace().peekable();
    while let Some(token) = tokens.next() {
        let next = tokens.peek().copied().unwrap_or("(end)");
        println!("    token {token:<2} followed by {next}");
    }

    println!("\n9. Taking items out of a collection: retain, extract_if");
    let mut queue = vec!["urgent: server down", "lunch?", "urgent: invoice", "newsletter"];
    // extract_if removes the matching items AND gives them to you, in one pass
    let urgent: Vec<&str> = queue.extract_if(.., |message| message.starts_with("urgent")).collect();
    println!("    handled first: {urgent:?}");
    println!("    still queued:  {queue:?}");
    queue.retain(|message| *message != "newsletter"); // retain keeps matches and drops the rest
    println!("    after retain:  {queue:?}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapters_do_nothing_without_a_consumer() {
        let mut calls = 0;
        // `inspect` lets you watch items go by without changing them.
        let iter = (0..5).inspect(|_| calls += 1);
        drop(iter); // never consumed
        assert_eq!(calls, 0);
    }

    #[test]
    fn take_stops_an_infinite_iterator_early() {
        let mut calls = 0;
        let first: Vec<u32> = (1..)
            .map(|n| {
                calls += 1;
                n * n
            })
            .take(3)
            .collect();
        assert_eq!(first, [1, 4, 9]);
        assert_eq!(calls, 3); // only as many items as were needed
    }

    #[test]
    fn grouping_with_fold_and_entry() {
        let totals = sales().iter().fold(HashMap::new(), |mut map, s| {
            *map.entry(s.product).or_insert(0) += s.amount;
            map
        });
        assert_eq!(totals["coffee"], 410);
        assert_eq!(totals["tea"], 110);
    }

    #[test]
    fn collect_target_decides_the_result() {
        let v = [3, 1, 3];
        let as_vec: Vec<i32> = v.into_iter().collect();
        let as_set: HashSet<i32> = v.into_iter().collect();
        assert_eq!(as_vec.len(), 3);
        assert_eq!(as_set.len(), 2);
    }

    #[test]
    fn windows_and_zip() {
        let changes: Vec<i32> = [1, 4, 9].windows(2).map(|w| w[1] - w[0]).collect();
        assert_eq!(changes, [3, 5]);
        let pairs: Vec<(char, i32)> = "ab".chars().zip(1..).collect();
        assert_eq!(pairs, [('a', 1), ('b', 2)]);
    }

    #[test]
    fn array_windows_destructures_each_window() {
        let sums: Vec<i32> = [1, 2, 3, 4].array_windows().map(|[a, b, c]| a + b + c).collect();
        assert_eq!(sums, [6, 9]);
    }

    #[test]
    fn extract_if_moves_matches_out() {
        let mut numbers = vec![1, 2, 3, 4, 5, 6];
        let even: Vec<i32> = numbers.extract_if(.., |n| *n % 2 == 0).collect();
        assert_eq!(even, [2, 4, 6]);
        assert_eq!(numbers, [1, 3, 5]);
        // only look at part of the Vec: the range limits where it searches
        let mut more = vec![2, 4, 6, 8];
        let first_two: Vec<i32> = more.extract_if(..2, |_| true).collect();
        assert_eq!((first_two, more), (vec![2, 4], vec![6, 8]));
    }
}
