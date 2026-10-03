<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Strategy Pattern

## What is it?

The **Strategy** pattern lets you swap *how* something is done without changing the code that does it.

Think of a navigation app. You type in where you want to go, then choose: car, bike or walking. The app does the same job every time (work out how long the trip takes), but the way it calculates the answer depends on your choice. Each of those ways of calculating is a **strategy**.

Without the pattern, you'd end up with one function full of `if` statements:

```rust
if mode == "car" { ... } else if mode == "bike" { ... } else if mode == "walk" { ... }
```

Every new way of travelling means editing that function again. With the Strategy pattern, each way of travelling is its own small piece of code, and adding a new one doesn't touch the existing ones.

## The parts

```text
                    ┌──────────────────────┐
                    │ «trait»              │
  RoutePlanner ────►│ TravelStrategy       │
  (the context)     │   minutes(route)     │
                    └──────────▲───────────┘
                               │ implemented by
              ┌────────────────┼────────────────┐
              │                │                │
          ┌───────┐        ┌────────┐       ┌────────┐
          │ ByCar │        │ ByBike │       │ OnFoot │
          └───────┘        └────────┘       └────────┘
```

- **Strategy** (`TravelStrategy` trait): the common interface. Every strategy must answer the same question: how many minutes does this route take?
- **Concrete strategies** (`ByCar`, `ByBike`, `OnFoot`): the different ways of answering it.
- **Context** (`RoutePlanner`): the code that uses a strategy. It holds one, calls it, and can swap it for another at any time. It never needs to know which one it has.

## The example

A trip has two legs: a flat 4 km stretch and a hilly 6 km one. The same `RoutePlanner` estimates it three ways:

| Strategy | Speed | Hills |
|---|---|---|
| `ByCar` | 50 km/h, plus 5 minutes to park | don't matter |
| `ByBike` | 15 km/h | +3 minutes each |
| `OnFoot` | 5 km/h | +5 minutes each |

The demo also shows choosing a strategy at runtime (it's raining, so take the car), and a fourth strategy, an e-scooter, written as a closure.

## The Rust way

In Rust there are two common ways to write a strategy, and this example shows both:

1. **A trait + structs** (`TravelStrategy`, `ByCar`, …). Use this when a strategy has a name, some settings, or several methods. The context stores it as `Box<dyn TravelStrategy>`, which means "any type that implements the trait, decided at runtime".
2. **A closure** (`trip_minutes_with(legs, |route| ...)`). When a strategy is just one function, you don't need a trait at all: pass the function itself. Rust's iterator methods work this way. `.map(...)`, `.filter(...)` and `.sort_by(...)` all take a strategy as a closure.

## When to use it

- You have several ways of doing the same task and want to pick one at runtime.
- You see a growing `if`/`match` that chooses between algorithms.
- You want to test each algorithm on its own.

**When not to:** if there are only two options and they'll never change, a simple `if` is clearer.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p strategy-pattern`.
