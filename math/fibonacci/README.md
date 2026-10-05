<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Fibonacci: recursion and an iterative fold

Start with `0, 1`. Each following number is the sum of the previous two:

```text
index:  0  1  2  3  4  5  6  7  8  9  10
value:  0  1  1  2  3  5  8 13 21 34  55
```

Leonardo of Pisa, known as Fibonacci, discussed the sequence in his 1202 book *Liber Abaci* through an idealized rabbit-population problem. The rabbit story is an idealized mathematical model. [Fibonacci biography and rabbit problem](https://mathshistory.st-andrews.ac.uk/Biographies/Fibonacci/)

## Compare two algorithms

`fibonacci(n)` follows the recurrence with recursive calls. To calculate `F(5)`, it asks for `F(4)` and `F(3)`, then repeats many smaller calculations. Its running time grows exponentially and its call depth grows linearly. This version accepts only `0..=MAX_RECURSIVE_INDEX` (30); larger requests return an error before starting that expensive work.

`calculate_fibonacci(n)` keeps just the previous two values. Its `try_fold` advances them once per index: O(n) time for representable results and O(1) auxiliary memory. A checked addition ends the calculation with `None` if the next result does not fit `u64`.

```rust
assert_eq!(calculate_fibonacci(10), Some(55));
assert_eq!(calculate_fibonacci(93), Some(12_200_160_415_121_876_738));
assert_eq!(calculate_fibonacci(94), None);
```

The program prints both algorithms' answers for indices 0–10. A recursive function is not inherently “imperative”, and an iterator is not inherently better: the repeated work and overflow behavior are the differences that matter here.

```bash
cargo run
cargo test
```
