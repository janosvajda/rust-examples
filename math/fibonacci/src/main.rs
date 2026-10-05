// Compare recursive calls with an iterative fold on the same sequence.
// The recursive version repeats work exponentially; keep its input bounded.
const MAX_RECURSIVE_INDEX: u32 = 30;

fn fibonacci(n: u32) -> Result<u64, &'static str> {
    if n > MAX_RECURSIVE_INDEX {
        return Err("recursive input exceeds 30; use calculate_fibonacci");
    }
    fn recurse(n: u32) -> u64 {
        match n {
            0 => 0,
            1 => 1,
            _ => recurse(n - 1) + recurse(n - 2),
        }
    }
    Ok(recurse(n))
}

fn calculate_fibonacci(n: u32) -> Option<u64> {
    match n {
        0 => Some(0),
        1 => Some(1),
        _ => (2..=n)
            .try_fold((0_u64, 1_u64), |(a, b), _| Some((b, a.checked_add(b)?)))
            .map(|(_, b)| b),
    }
}

fn main() {
    for i in 0..=10 {
        println!(
            "Fibonacci({i}): recursive {:?}, iterative {:?}",
            fibonacci(i),
            calculate_fibonacci(i)
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn both_algorithms_agree() {
        for (i, value) in [0, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55]
            .into_iter()
            .enumerate()
        {
            assert_eq!(fibonacci(i as u32), Ok(value));
            assert_eq!(calculate_fibonacci(i as u32), Some(value));
        }
    }
    #[test]
    fn bounds_do_not_wrap_or_start_an_enormous_recursive_calculation() {
        assert!(fibonacci(MAX_RECURSIVE_INDEX + 1).is_err());
        assert_eq!(calculate_fibonacci(93), Some(12_200_160_415_121_876_738));
        assert_eq!(calculate_fibonacci(94), None);
        assert_eq!(calculate_fibonacci(u32::MAX), None);
    }
}
