// Lesson 5: data parallelism with rayon.
//
// Most parallel work is "do the same thing to every item". rayon turns an
// ordinary iterator chain into a parallel one by changing `.iter()` into
// `.par_iter()`: it splits the work across all CPU cores, balances it
// automatically, and puts the results back together. And because it follows
// Rust's borrowing rules, there are no data races to worry about.

use rayon::prelude::*;
use std::time::Instant;

/// Deliberately slow: counts primes below `n` the naive way.
fn is_prime(n: u64) -> bool {
    n >= 2 && (2..).take_while(|d| d * d <= n).all(|d| !n.is_multiple_of(d))
}

fn count_primes_sequential(limit: u64) -> usize {
    (0..limit).filter(|&n| is_prime(n)).count()
}

fn count_primes_parallel(limit: u64) -> usize {
    (0..limit).into_par_iter().filter(|&n| is_prime(n)).count()
}

/// rayon::join runs two closures, potentially in parallel, and waits for both.
/// A recursive split like this is how parallel sorts and searches work.
fn parallel_sum(numbers: &[u64]) -> u64 {
    if numbers.len() <= 10_000 {
        return numbers.iter().sum(); // small enough: just do it
    }
    let (left, right) = numbers.split_at(numbers.len() / 2);
    let (a, b) = rayon::join(|| parallel_sum(left), || parallel_sum(right));
    a + b
}

fn time<T>(label: &str, work: impl FnOnce() -> T) -> T {
    let start = Instant::now();
    let result = work();
    println!("    {label:<12} {:>9.3} ms", start.elapsed().as_secs_f64() * 1000.0);
    result
}

fn main() {
    println!("rayon is using {} threads\n", rayon::current_num_threads());

    println!("1. iter() → par_iter(): the same chain, on every core");
    let limit = 3_000_000;
    let a = time("sequential", || count_primes_sequential(limit));
    let b = time("parallel", || count_primes_parallel(limit));
    println!("    both found {a} primes: {}", a == b);

    println!("\n2. Parallel map + collect keeps the original order");
    let words = vec!["rust", "is", "fast", "and", "fearless"];
    let lengths: Vec<usize> = words.par_iter().map(|w| w.len()).collect();
    println!("    {words:?} → {lengths:?}");

    println!("\n3. Parallel sort");
    let mut data: Vec<u64> = (0..2_000_000u64).map(|i| (i * 7_919) % 1_000_003).collect();
    let mut copy = data.clone();
    time("sort", || copy.sort_unstable());
    time("par_sort", || data.par_sort_unstable());
    println!("    same result: {}", data == copy);

    println!("\n4. rayon::join: split the work yourself");
    let numbers: Vec<u64> = (1..=1_000_000).collect();
    println!("    sum of 1..=1,000,000 = {}", parallel_sum(&numbers));

    println!("\n5. When it doesn't help");
    let small: Vec<u64> = (0..100).collect();
    time("tiny seq", || small.iter().map(|x| x * 2).sum::<u64>());
    time("tiny par", || small.par_iter().map(|x| x * 2).sum::<u64>());
    println!("    splitting 100 trivial items across threads costs more than it saves");

    // The borrowing rules still apply inside parallel closures. This doesn't compile:
    //     let mut total = 0;
    //     data.par_iter().for_each(|x| total += x);
    //     error[E0596]: cannot borrow `total` as mutable, as it is a captured variable in a `Fn` closure
    // `for_each` may call the closure from several threads at once, so it
    // must be `Fn` (closures course, lesson 2). Use .sum(), or an atomic.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallel_and_sequential_agree() {
        assert_eq!(count_primes_parallel(10_000), count_primes_sequential(10_000));
        assert_eq!(count_primes_parallel(100), 25);
    }

    #[test]
    fn par_iter_collect_preserves_order() {
        let v: Vec<u32> = (0..1000).collect();
        let doubled: Vec<u32> = v.par_iter().map(|x| x * 2).collect();
        assert_eq!(doubled, v.iter().map(|x| x * 2).collect::<Vec<_>>());
    }

    #[test]
    fn par_sort_sorts() {
        let mut v: Vec<i32> = (0..10_000).rev().collect();
        v.par_sort();
        assert!(v.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn recursive_join_sum() {
        let numbers: Vec<u64> = (1..=100_000).collect();
        assert_eq!(parallel_sum(&numbers), 100_000 * 100_001 / 2);
    }
}
