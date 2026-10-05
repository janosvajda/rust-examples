// Iterator pattern: walk through the items of a collection one at a time,
// without the caller knowing how the collection stores them.
//
// In Rust this pattern is built into the language: implement the standard
// `Iterator` trait and your type works with `for` loops and with every
// iterator method (`filter`, `map`, `sum`, `take`, ...) for free.

#[derive(Debug, Clone, PartialEq)]
struct Song {
    title: String,
    artist: String,
    seconds: u32,
}

impl Song {
    fn new(title: &str, artist: &str, seconds: u32) -> Song {
        Song {
            title: title.to_string(),
            artist: artist.to_string(),
            seconds,
        }
    }
}

// ---- Example 1: iterating over a custom collection ------------------------

/// A playlist. How it stores its songs (here a `Vec`, plus a set of skipped
/// positions) is private. Callers only get an iterator.
struct Playlist {
    songs: Vec<Song>,
    skipped: Vec<usize>, // positions the user chose to skip
}

impl Playlist {
    fn new(songs: Vec<Song>) -> Self {
        Playlist {
            songs,
            skipped: Vec::new(),
        }
    }

    fn skip(&mut self, position: usize) {
        self.skipped.push(position);
    }

    /// Returns an iterator over the songs that will actually play.
    fn iter(&self) -> PlaylistIter<'_> {
        PlaylistIter {
            playlist: self,
            position: 0,
        }
    }
}

/// The iterator. It remembers where it is (`position`) and borrows the
/// playlist, so the songs aren't copied. The lifetime `'a` says the iterator
/// can't outlive the playlist it borrows.
struct PlaylistIter<'a> {
    playlist: &'a Playlist,
    position: usize,
}

impl<'a> Iterator for PlaylistIter<'a> {
    type Item = &'a Song;

    /// The only method we have to write. Return the next item, or `None`
    /// when there are no more.
    fn next(&mut self) -> Option<Self::Item> {
        while self.position < self.playlist.songs.len() {
            let current = self.position;
            self.position += 1;
            if !self.playlist.skipped.contains(&current) {
                return Some(&self.playlist.songs[current]);
            }
        }
        None
    }
}

/// Lets callers write `for song in &playlist { ... }`.
impl<'a> IntoIterator for &'a Playlist {
    type Item = &'a Song;
    type IntoIter = PlaylistIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// ---- Example 2: an iterator that computes its items -----------------------

/// The Fibonacci sequence: 0, 1, 1, 2, 3, 5, 8, ...
/// There's no collection behind it. Each item is calculated when it's asked
/// for. This iterator ends after the last Fibonacci number that fits in u64.
struct Fibonacci {
    pair: Option<(u64, u64)>,
    last: Option<u64>,
}

fn fibonacci() -> Fibonacci {
    Fibonacci {
        pair: Some((0, 1)),
        last: None,
    }
}

impl Iterator for Fibonacci {
    type Item = u64;

    fn next(&mut self) -> Option<u64> {
        let Some((current, next)) = self.pair.take() else {
            return self.last.take();
        };
        if let Some(following) = current.checked_add(next) {
            self.pair = Some((next, following));
        } else {
            self.last = Some(next);
        }
        Some(current)
    }
}

fn format_duration(seconds: u32) -> String {
    format!("{}:{:02}", seconds / 60, seconds % 60)
}

fn main() {
    let mut playlist = Playlist::new(vec![
        Song::new("Bohemian Rhapsody", "Queen", 354),
        Song::new("Imagine", "John Lennon", 183),
        Song::new("Smells Like Teen Spirit", "Nirvana", 301),
        Song::new("Don't Stop Me Now", "Queen", 209),
        Song::new("Hey Jude", "The Beatles", 431),
    ]);
    playlist.skip(2); // not in the mood for Nirvana

    println!("--- Playing (song 3 skipped) ---");
    for (number, song) in playlist.iter().enumerate() {
        println!(
            "{}. {} – {} ({})",
            number + 1,
            song.title,
            song.artist,
            format_duration(song.seconds)
        );
    }

    // Because PlaylistIter implements Iterator, all of these come for free.
    println!("\n--- Iterator methods we didn't have to write ---");
    let total: u32 = playlist.iter().map(|song| song.seconds).sum();
    println!("total playing time: {}", format_duration(total));

    let queen: Vec<&str> = playlist
        .iter()
        .filter(|song| song.artist == "Queen")
        .map(|song| song.title.as_str())
        .collect();
    println!("Queen songs: {queen:?}");

    let longest = playlist.iter().max_by_key(|song| song.seconds).unwrap();
    println!("longest: {}", longest.title);

    println!("\n--- A generated sequence, used lazily ---");
    // Nothing is calculated until `take` asks for items, so an infinite
    // sequence is fine.
    let first_ten: Vec<u64> = fibonacci().take(10).collect();
    println!("first 10 Fibonacci numbers: {first_ten:?}");

    let first_over_1000 = fibonacci().find(|&n| n > 1000).unwrap();
    println!("first one over 1000: {first_over_1000}");

    let even_sum: u64 = fibonacci()
        .take_while(|&n| n < 100)
        .filter(|n| n % 2 == 0)
        .sum();
    println!("sum of the even ones below 100: {even_sum}");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn three_songs() -> Playlist {
        Playlist::new(vec![
            Song::new("A", "X", 60),
            Song::new("B", "Y", 120),
            Song::new("C", "X", 180),
        ])
    }

    fn titles(playlist: &Playlist) -> Vec<&str> {
        playlist.iter().map(|s| s.title.as_str()).collect()
    }

    #[test]
    fn iterates_in_order() {
        assert_eq!(titles(&three_songs()), vec!["A", "B", "C"]);
    }

    #[test]
    fn skipped_songs_are_left_out() {
        let mut playlist = three_songs();
        playlist.skip(0);
        playlist.skip(2);
        assert_eq!(titles(&playlist), vec!["B"]);
    }

    #[test]
    fn works_in_a_for_loop_and_with_iterator_methods() {
        let playlist = three_songs();
        let mut count = 0;
        for _song in &playlist {
            count += 1;
        }
        assert_eq!(count, 3);
        assert_eq!(playlist.iter().map(|s| s.seconds).sum::<u32>(), 360);
    }

    #[test]
    fn empty_playlist_yields_nothing() {
        assert_eq!(Playlist::new(vec![]).iter().next(), None);
    }

    #[test]
    fn fibonacci_sequence() {
        let first: Vec<u64> = fibonacci().take(8).collect();
        assert_eq!(first, vec![0, 1, 1, 2, 3, 5, 8, 13]);
    }

    #[test]
    fn fibonacci_stops_before_overflowing() {
        // Yields every representable term, then ends before an overflowing term.
        assert!(fibonacci().count() > 90);
    }
    #[test]
    fn fibonacci_keeps_the_last_two_representable_terms() {
        let terms: Vec<_> = fibonacci().collect();
        assert_eq!(terms.len(), 94);
        assert_eq!(terms[92], 7_540_113_804_746_346_429);
        assert_eq!(terms[93], 12_200_160_415_121_876_738);
    }
}
