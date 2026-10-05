<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Writing your own iterators

## The idea in one sentence

You can make anything iterable, from a one-line closure to a full struct, and even add your own adapters that work on **every** iterator, just like `map` and `filter`.

## Four ways, from least to most code

| Way | Write | Good for |
|---|---|---|
| 1. from a closure | `iter::from_fn(…)`, `iter::successors(…)` | quick, one-off sequences |
| 2. a struct | `impl Iterator for MyType` | full control, extra traits like `rev()` |
| 3. your collection | `impl IntoIterator for &MyCollection` | `for x in &collection` |
| 4. an adapter | a wrapper struct + an extension trait | `.every_nth(3)` on any iterator |

## 1. Iterators from closures

```rust
iter::successors((limit >= 1).then_some(1), |&n| n.checked_mul(2).filter(|&next| next <= limit))   // 1, 2, 4, 8…
iter::from_fn(move || { state = …; Some(state % 6 + 1) })                        // dice rolls
```

- **`successors`**: each item is computed **from the previous one**, and the sequence ends when the closure returns `None`.
- **`from_fn`**: the closure is called for each item and keeps its own state, which it captured with `move`. Return `None` to end the sequence, or never return it for an endless one and use `.take(n)`.

No struct, no trait implementation, and the result still works with every iterator method.

## 2. A struct implementing `Iterator`

```rust
impl Iterator for Countdown {
    type Item = u32;
    fn next(&mut self) -> Option<u32> { … }
}
```

`next` is the only required method. Two optional extras make a struct-based iterator much more useful:

| Implement | And callers get |
|---|---|
| `DoubleEndedIterator` (`next_back`) | `.rev()`, and taking items from **both ends** of the same iterator |
| `size_hint` + `ExactSizeIterator` | `.len()` when the length fits `usize`; consumers can use the size hint for allocation |

The demo's `Countdown` keeps a count of the items **remaining**. That makes both ends easy: `next` takes from the top, `next_back` from the bottom, and both stop when nothing remains. A test takes one from each end and checks that the middle is left.

## 3. Making your own collection iterable

```rust
impl<'a> IntoIterator for &'a Playlist {        // for song in &playlist → &String
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;
    fn into_iter(self) -> Self::IntoIter { self.songs.iter() }
}

impl IntoIterator for Playlist {                // for song in playlist → String
    …
}
```

A `for` loop calls `into_iter()` on whatever you give it. Implementing `IntoIterator` for `&Playlist` and for `Playlist` gives you both "borrow each item" and "take each item", like a `Vec`. When your collection is backed by a standard one, just hand out *its* iterator, as here, rather than writing your own.

## 4. Your own adapter

An adapter is an iterator that **wraps another iterator**:

```rust
struct EveryNth<I> { inner: I, n: usize }

impl<I: Iterator> Iterator for EveryNth<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<I::Item> {
        let item = self.inner.next()?;              // take one…
        for _ in 1..self.n { self.inner.next(); }   // …skip n-1
        Some(item)
    }
}
```

To make it available as a method on **every** iterator, add an extension trait with a blanket implementation (traits course, lesson 6):

```rust
trait EveryNthExt: Iterator + Sized {
    fn every_nth(self, n: usize) -> EveryNth<Self> { EveryNth { inner: self, n } }
}
impl<I: Iterator> EveryNthExt for I {}

(1..=10).every_nth(3)                        // 1, 4, 7, 10
(1..).map(|n| n * n).every_nth(2).take(4)    // mixes freely with built-in adapters
```

This is exactly how `map`, `filter` and the others work inside the standard library: each is a small struct wrapping the previous iterator. (The standard library already has this one as `step_by`. Writing it yourself shows how adapters work.)

## The ends of a sequence

Edge values are where iterators go wrong, so the tests check them:

- `powers_of_two(0)` is **empty**: even the first value, 1, is above the limit. `checked_mul` ends the sequence before a multiplication could overflow.
- `countdown(n)` includes both `n` and `0`, so `countdown(u32::MAX)` has 4,294,967,296 items: one more than a `u32` can count. That's why `Countdown` keeps its remaining count in a `u64`. And since an iterator is lazy, `countdown(u32::MAX).take(2)` is instant: it never creates the other four billion items.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 4: The iterator toolbox](../04-iterator-toolbox/) · Back to the [course overview](../)
