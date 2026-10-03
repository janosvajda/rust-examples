<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Iterator Pattern

## What is it?

The **Iterator** pattern gives you a way to go through the items of a collection one by one, without needing to know how the collection stores them.

Think of a TV remote's "next channel" button. You press it and get the next channel. You don't need to know how the TV stores its channel list, which channels are hidden, or what frequency each one uses. "Give me the next one" is all you need, and the TV keeps track of where you are.

## The parts

```text
  your code                   PlaylistIter (the iterator)              Playlist (the collection)
  ─────────                   ───────────────────────────              ─────────────────────────
  next() ───────────────────► remembers the current position ───────► songs: [ … ]
         ◄── Some(song) ───── skips the songs marked "skipped"         skipped: [ … ]
  next() ─────────────────►
         ◄── None ──────────  (no more songs)
```

- **Collection** (`Playlist`): owns the items. How it stores them is its own business.
- **Iterator** (`PlaylistIter`): walks through the items. It has one job: `next()` returns the next item, or `None` when there are no more.

## Built into Rust

Most languages need you to write this pattern yourself. **Rust has it built in.** The standard `Iterator` trait is this pattern, and it's used everywhere: every `for` loop in Rust runs on an iterator.

You only write **one method**, `next()`. In return you get dozens of methods for free: `map`, `filter`, `sum`, `max_by_key`, `take`, `enumerate`, `collect`, and many more.

## The example

**1. A playlist.** `Playlist` stores songs plus a list of positions the user chose to skip. `PlaylistIter` returns only the songs that will actually play. Then the demo uses methods we never wrote:

- `.map(...).sum()` adds up the total playing time;
- `.filter(...).map(...).collect()` lists only the Queen songs;
- `.max_by_key(...)` finds the longest song;
- `for song in &playlist` works because of `IntoIterator`.

**2. The Fibonacci sequence** (0, 1, 1, 2, 3, 5, 8, …). There's no collection behind this iterator at all. Each number is **calculated when it's asked for**. That's why the sequence can be endless: `take(10)` asks for exactly ten numbers, and nothing more is ever computed. This is called *lazy* evaluation. To avoid an overflow crash, the iterator simply ends once the numbers no longer fit in a `u64`.

## Things worth knowing

- **Lifetimes:** `PlaylistIter<'a>` *borrows* the playlist instead of copying it. The `'a` tells the compiler the iterator can't outlive the playlist, so you can never iterate over songs that no longer exist.
- **Three ways to iterate a collection** in Rust:
  - `.iter()` borrows the items (`&T`);
  - `.iter_mut()` borrows them so you can change them (`&mut T`);
  - `.into_iter()` takes ownership and hands out the items themselves (`T`).
- **Iterator chains are fast.** The compiler turns a chain like `.filter().map().sum()` into a single loop, usually as fast as one you'd write by hand.

## When to use it

In Rust: whenever your type holds or produces a sequence of items. Implement `Iterator` (and `IntoIterator` for `for` loops), and your type works with the entire iterator toolbox and with any function that accepts an iterator.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p iterator-pattern`.
