// Lesson 6: the cache.
//
// Main memory is slow compared with the processor. So the processor keeps
// copies of recently used memory in small, fast memories on the chip: the
// caches. This program MEASURES the difference on the machine it runs on.
// Run it with `cargo run --release`: timing a debug build measures mostly
// the debug build's own overhead.

use std::hint::black_box;
use std::time::Instant;

/// A tiny pseudo-random generator (xorshift), so a fixed seed repeats the
/// visiting orders. Timings can still change from run to run.
struct Random(u64);

impl Random {
    fn below(&mut self, n: usize) -> usize {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 % n as u64) as usize
    }
}

/// A random order of 0..n (Fisher–Yates shuffle).
fn shuffled(n: usize, random: &mut Random) -> Vec<u32> {
    let mut order: Vec<u32> = (0..n as u32).collect();
    for i in (1..n).rev() {
        order.swap(i, random.below(i + 1));
    }
    order
}

// ---- 1. How long does ONE memory read take? ---------------------------------------------

/// Builds a table where `next[i]` says which slot to visit after slot i, forming
/// one big random cycle through every slot (Sattolo's algorithm).
fn random_cycle(slots: usize, random: &mut Random) -> Vec<u32> {
    let mut next: Vec<u32> = (0..slots as u32).collect();
    for i in (1..slots).rev() {
        next.swap(i, random.below(i)); // `below(i)`, not `i + 1`: guarantees one single cycle
    }
    next
}

/// Follows the cycle for `steps` steps. Each read's address depends on the
/// previous read's result, limiting overlap. Hardware may still predict or
/// prefetch; the average also includes loop and address-translation work.
fn nanoseconds_per_read(next: &[u32], steps: usize) -> f64 {
    let mut slot = 0u32;
    let start = Instant::now();
    for _ in 0..steps {
        slot = next[slot as usize];
    }
    black_box(slot);
    start.elapsed().as_nanos() as f64 / steps as f64
}

// ---- 2. In order vs random order ----------------------------------------------------------

fn sum_in_order(numbers: &[u32]) -> u64 {
    numbers.iter().map(|&n| u64::from(n)).sum()
}

fn sum_in_given_order(numbers: &[u32], order: &[u32]) -> u64 {
    order.iter().map(|&i| u64::from(numbers[i as usize])).sum()
}

// ---- 3. Row by row vs column by column -----------------------------------------------------

/// A grid stored in one flat vector, row after row: position = row × side + column.
fn sum_rows_first(grid: &[u32], side: usize) -> u64 {
    let mut total = 0u64;
    for row in 0..side {
        for column in 0..side {
            total += u64::from(grid[row * side + column]); // neighbours in memory
        }
    }
    total
}

fn sum_columns_first(grid: &[u32], side: usize) -> u64 {
    let mut total = 0u64;
    for column in 0..side {
        for row in 0..side {
            total += u64::from(grid[row * side + column]); // `side` elements apart
        }
    }
    total
}

fn time<T>(work: impl FnOnce() -> T) -> (T, f64) {
    let start = Instant::now();
    let result = black_box(work());
    (result, start.elapsed().as_secs_f64() * 1000.0)
}

fn main() {
    if cfg!(debug_assertions) {
        println!("(a debug build: timings are only meaningful with `cargo run --release`)\n");
    }
    let mut random = Random(0x2545_F491_4F6C_DD1D);

    println!("1. The time of one memory read, depending on how much memory is in use");
    for (label, bytes) in [
        ("16 KiB", 16 << 10),
        ("1 MiB", 1 << 20),
        ("256 MiB", 256 << 20),
    ] {
        let next = random_cycle(bytes / size_of::<u32>(), &mut random);
        let ns = nanoseconds_per_read(&next, 20_000_000);
        println!("    {label:>7} of data: {ns:6.1} ns per read");
    }

    println!("\n2. Reading 64 MiB: in order, or in random order");
    let count = 16 << 20; // 16 Mi numbers × 4 bytes = 64 MiB
    let numbers: Vec<u32> = (0..count as u32).collect();
    let order = shuffled(count, &mut random);
    let (a, in_order) = time(|| sum_in_order(&numbers));
    let (b, scattered) = time(|| sum_in_given_order(&numbers, &order));
    assert_eq!(a, b);
    println!("    in order:     {in_order:7.1} ms");
    println!(
        "    random order: {scattered:7.1} ms   ({:.0}× slower, the same numbers)",
        scattered / in_order
    );

    println!("\n3. A 4096 × 4096 grid: row by row, or column by column");
    let side = 4096;
    let grid: Vec<u32> = (0..(side * side) as u32).collect();
    let (a, rows) = time(|| sum_rows_first(&grid, black_box(side)));
    let (b, columns) = time(|| sum_columns_first(&grid, black_box(side)));
    assert_eq!(a, b);
    println!("    row by row:       {rows:7.1} ms");
    println!(
        "    column by column: {columns:7.1} ms   ({:.0}× slower, the same sum)",
        columns / rows
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_cycle_visits_every_slot_exactly_once() {
        let next = random_cycle(1000, &mut Random(1));
        let mut seen = vec![false; 1000];
        let mut slot = 0usize;
        for _ in 0..1000 {
            assert!(!seen[slot], "visited {slot} twice before the cycle ended");
            seen[slot] = true;
            slot = next[slot] as usize;
        }
        assert_eq!(slot, 0); // back to the start after exactly 1000 steps
    }

    #[test]
    fn every_order_gives_the_same_sum() {
        let numbers: Vec<u32> = (0..10_000).collect();
        let order = shuffled(numbers.len(), &mut Random(7));
        assert_eq!(sum_in_order(&numbers), sum_in_given_order(&numbers, &order));
        let grid: Vec<u32> = (0..64 * 64).collect();
        assert_eq!(sum_rows_first(&grid, 64), sum_columns_first(&grid, 64));
    }

    #[test]
    fn the_shuffle_is_a_permutation() {
        let mut order = shuffled(500, &mut Random(3));
        order.sort_unstable();
        assert_eq!(order, (0..500).collect::<Vec<u32>>());
    }
}
