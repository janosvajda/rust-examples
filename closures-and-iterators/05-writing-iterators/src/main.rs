// Lesson 5: writing your own iterators.
//
// Four ways to make something iterable, from least to most code:
//   1. iter::from_fn / iter::successors: an iterator from a closure, no struct
//   2. impl Iterator for a struct: full control
//   3. IntoIterator for your collection: so `for x in &collection` works
//   4. your own ADAPTER plus an extension trait: so `.every_nth(3)` works on any iterator

use std::iter;

// ---- 1. Iterators from closures -----------------------------------------------------------

/// Powers of two up to `limit`: each item is computed from the previous one.
/// `successors` stops when the closure returns None.
fn powers_of_two(limit: u32) -> impl Iterator<Item = u32> {
    iter::successors((limit >= 1).then_some(1u32), move |&n| {
        n.checked_mul(2).filter(|&next| next <= limit)
    })
}

/// A die that rolls the same "random" sequence every time (for tests).
/// `from_fn` calls the closure for each item; the closure keeps the state.
fn fake_dice(seed: u32) -> impl Iterator<Item = u32> {
    let mut state = seed;
    iter::from_fn(move || {
        state = state.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        Some(state % 6 + 1)
    })
}

// ---- 2. A struct implementing Iterator ---------------------------------------------------

/// Counts down to zero: 3, 2, 1, 0. Implementing `DoubleEndedIterator` too
/// means it can also be walked from the other end, so `.rev()` works.
/// Keeping a count of the items left makes every method simple.
struct Countdown {
    high: u32,      // the next number from the front
    low: u32,       // the next number from the back
    remaining: u64, // includes zero: u32::MAX + 1 items must fit too
}

fn countdown(from: u32) -> Countdown {
    Countdown {
        high: from,
        low: 0,
        remaining: u64::from(from) + 1,
    }
}

impl Iterator for Countdown {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let value = self.high;
        self.high = self.high.saturating_sub(1);
        Some(value)
    }

    /// Telling iterator methods exactly how many items are left lets
    /// `collect` allocate the right amount of memory up front.
    fn size_hint(&self) -> (usize, Option<usize>) {
        match usize::try_from(self.remaining) {
            Ok(left) => (left, Some(left)),
            Err(_) => (usize::MAX, None),
        }
    }
}

impl DoubleEndedIterator for Countdown {
    fn next_back(&mut self) -> Option<u32> {
        if self.remaining == 0 {
            return None;
        }
        self.remaining -= 1;
        let value = self.low;
        self.low = self.low.saturating_add(1);
        Some(value)
    }
}

/// The length is exact, so `.len()` is available too.
// A full u32 countdown fits usize on 64-bit targets, but not on 32-bit ones.
#[cfg(target_pointer_width = "64")]
impl ExactSizeIterator for Countdown {}

// ---- 3. Making your own collection iterable ---------------------------------------------

/// A playlist that hides how it stores its songs.
struct Playlist {
    songs: Vec<String>,
}

/// `for song in &playlist` → borrows each song.
impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a String;
    type IntoIter = std::slice::Iter<'a, String>;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.iter()
    }
}

/// `for song in playlist` → takes ownership of each song.
impl IntoIterator for Playlist {
    type Item = String;
    type IntoIter = std::vec::IntoIter<String>;

    fn into_iter(self) -> Self::IntoIter {
        self.songs.into_iter()
    }
}

// ---- 4. Your own adapter, available on every iterator -----------------------------------

/// Yields every n-th item of the iterator it wraps: the 1st, (n+1)th, …
/// (The standard library has `step_by` for this; writing it shows how
/// adapters like map and filter work inside.)
struct EveryNth<I> {
    inner: I,
    n: usize,
}

impl<I: Iterator> Iterator for EveryNth<I> {
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let item = self.inner.next()?; // take one…
        for _ in 1..self.n {
            self.inner.next(); // …and skip n-1
        }
        Some(item)
    }
}

/// An extension trait (traits course, lesson 6) with a blanket impl, so
/// `.every_nth(n)` works on ANY iterator, just like the built-in adapters.
trait EveryNthExt: Iterator + Sized {
    fn every_nth(self, n: usize) -> EveryNth<Self> {
        assert!(n > 0, "n must be at least 1");
        EveryNth { inner: self, n }
    }
}

impl<I: Iterator> EveryNthExt for I {}

fn main() {
    println!("1. Iterators from closures");
    println!(
        "    powers of two up to 100: {:?}",
        powers_of_two(100).collect::<Vec<_>>()
    );
    println!(
        "    five dice rolls: {:?}",
        fake_dice(42).take(5).collect::<Vec<_>>()
    );

    println!("\n2. A struct implementing Iterator (+ DoubleEnded + ExactSize)");
    println!(
        "    countdown(5):       {:?}",
        countdown(5).collect::<Vec<_>>()
    );
    println!(
        "    countdown(5).rev(): {:?}",
        countdown(5).rev().collect::<Vec<_>>()
    );
    println!(
        "    countdown(5).size_hint(): {:?}",
        countdown(5).size_hint()
    );
    let mut both_ends = countdown(4);
    println!(
        "    from both ends: front {:?}, back {:?}, front {:?}, rest {:?}",
        both_ends.next(),
        both_ends.next_back(),
        both_ends.next(),
        both_ends.collect::<Vec<_>>()
    );

    println!("\n3. A collection that works in for loops");
    let playlist = Playlist {
        songs: vec![String::from("Imagine"), String::from("Hey Jude")],
    };
    for song in &playlist {
        println!("    playing {song}");
    }
    let titles: Vec<String> = playlist.into_iter().map(|s| s.to_uppercase()).collect();
    println!("    took ownership: {titles:?}");

    println!("\n4. Our own adapter, on any iterator");
    println!(
        "    (1..=10).every_nth(3): {:?}",
        (1..=10).every_nth(3).collect::<Vec<_>>()
    );
    println!(
        "    letters every 2nd:     {:?}",
        "abcdefg".chars().every_nth(2).collect::<String>()
    );
    println!(
        "    mixed with built-ins:  {:?}",
        (1..)
            .map(|n| n * n)
            .every_nth(2)
            .take(4)
            .collect::<Vec<_>>()
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn successors_stops_at_the_limit() {
        assert_eq!(powers_of_two(20).collect::<Vec<_>>(), [1, 2, 4, 8, 16]);
    }

    #[test]
    fn from_fn_is_repeatable_with_the_same_seed() {
        let a: Vec<_> = fake_dice(7).take(10).collect();
        let b: Vec<_> = fake_dice(7).take(10).collect();
        assert_eq!(a, b);
        assert!(a.iter().all(|&d| (1..=6).contains(&d)));
    }

    #[test]
    fn countdown_forwards_backwards_and_len() {
        assert_eq!(countdown(3).collect::<Vec<_>>(), [3, 2, 1, 0]);
        assert_eq!(countdown(3).rev().collect::<Vec<_>>(), [0, 1, 2, 3]);
        assert_eq!(countdown(3).size_hint(), (4, Some(4)));
        assert_eq!(countdown(0).collect::<Vec<_>>(), [0]);
    }

    #[test]
    fn countdown_meets_in_the_middle() {
        let mut c = countdown(3); // 3 2 1 0
        assert_eq!(c.next(), Some(3));
        assert_eq!(c.next_back(), Some(0));
        assert_eq!(c.size_hint(), (2, Some(2)));
        assert_eq!(c.collect::<Vec<_>>(), [2, 1]);
    }

    #[test]
    fn playlist_in_for_loops() {
        let playlist = Playlist {
            songs: vec![String::from("a"), String::from("b")],
        };
        let borrowed: Vec<&String> = (&playlist).into_iter().collect();
        assert_eq!(borrowed.len(), 2);
        let owned: Vec<String> = playlist.into_iter().collect();
        assert_eq!(owned, ["a", "b"]);
    }

    #[test]
    fn every_nth_adapter() {
        assert_eq!((0..10).every_nth(4).collect::<Vec<_>>(), [0, 4, 8]);
        assert_eq!((0..3).every_nth(1).collect::<Vec<_>>(), [0, 1, 2]);
        assert_eq!(std::iter::empty::<u8>().every_nth(2).count(), 0);
    }
    #[test]
    fn iterator_boundaries_keep_all_representable_items() {
        assert!(powers_of_two(0).next().is_none());
        let mut c = countdown(u32::MAX);
        assert_eq!(c.next(), Some(u32::MAX));
        assert_eq!(c.next_back(), Some(0));
        let mut zero = countdown(0);
        assert_eq!(zero.next_back(), Some(0));
        assert_eq!(zero.next(), None);
    }
}
