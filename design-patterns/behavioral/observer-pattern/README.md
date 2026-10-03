<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Observer Pattern

## What is it?

The **Observer** pattern lets one object automatically tell many others when something changes, without knowing anything about them.

Think of subscribing to a YouTube channel. You click "subscribe" once, and from then on you're notified whenever a new video comes out. The channel doesn't know who you are or what you'll do with the notification. It just tells every subscriber. You can unsubscribe at any time.

In the pattern's terms, the channel is the **subject** and the subscribers are **observers**.

## The parts

```text
  ┌────────────────┐  publish(reading)
  │ WeatherStation │ ─────────┬────────────────────────┬─────────────────────┐
  │   (subject)    │          ▼                        ▼                     ▼
  └────────────────┘  CurrentConditionsDisplay   StatisticsDisplay       HeatAlert

                      all three implement the Observer trait: update(reading)
```

- **Subject** (`WeatherStation`): keeps a list of observers and has `subscribe`, `unsubscribe` and `publish`.
- **Observer** (the `Observer` trait): the one method every observer must have, `update(reading)`.
- **Concrete observers**: each one reacts to the same reading in its own way.

## The example

A weather station publishes temperature and humidity readings. Three displays are subscribed:

| Observer | What it does with each reading |
|---|---|
| `CurrentConditionsDisplay` | prints the latest values |
| `StatisticsDisplay` | remembers every temperature and prints the minimum and maximum |
| `HeatAlert` | stays quiet unless it's 30 °C or hotter |

Halfway through, the heat alert unsubscribes and stops receiving readings, while the others carry on.

The important part: **adding a new kind of display doesn't change the weather station at all.** You write a new type that implements `Observer` and subscribe it.

## The Rust way: who owns the observers?

In languages with a garbage collector, the subject simply keeps references to its observers. In Rust you have to decide who **owns** them, which is the tricky part of this pattern. This example uses two tools:

- **`Rc<dyn Observer>`**: `Rc` is a shared pointer, so both the station *and* your own code can hold the same observer. That's how `main` can still read the statistics display after giving it to the station.
- **`RefCell`**: `update` receives `&self`, a read-only reference, because several parts of the program share the observer. `StatisticsDisplay` still needs to add to its list of temperatures. `RefCell` makes that possible by checking Rust's borrowing rules while the program runs instead of at compile time.

`subscribe` returns a `SubscriptionId`, so you can unsubscribe without needing to compare observers with each other.

This single-threaded version uses `Rc` and `RefCell`. If observers were shared between threads, you'd use their thread-safe versions, `Arc` and `Mutex`. For sending events between threads, Rust's channels (`std::sync::mpsc`) are often a simpler alternative to observers.

## When to use it

- Several parts of a program need to react when something changes.
- The thing that changes shouldn't depend on the code that reacts to it.
- Listeners need to come and go while the program runs.

Typical examples: UI events (button clicks), "a new message arrived", "the settings changed".

**Watch out for:** with many observers it can get hard to follow *what happens when*, because one `publish` can trigger a chain of reactions elsewhere in the program.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p observer-pattern`.
