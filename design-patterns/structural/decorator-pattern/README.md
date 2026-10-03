<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Decorator Pattern

## What is it?

The **Decorator** pattern adds extra behaviour to an object by *wrapping* it in another object with the same interface. You can stack as many wrappers as you like, in any order.

Think of ordering coffee. You start with an espresso, then say "with milk", "and sugar", "make it large". Each extra is added on top of what you already have, and the price goes up step by step. The coffee shop doesn't need a separate menu item for every possible combination like "large espresso with milk and two sugars". It has a few base drinks and a few extras that can wrap any drink.

## The parts

```text
  large( with_cream( with_milk( Espresso ) ) )

  ┌─ LargeSize ─────────────────────────────────┐
  │  ┌─ WhippedCream ────────────────────────┐  │
  │  │  ┌─ Milk ──────────────────────────┐  │  │
  │  │  │  ┌─ Espresso ───┐               │  │  │
  │  │  │  │ 2.50 €       │  + 0.50 €     │  │  │
  │  │  │  └──────────────┘               │  │  │
  │  │  └──────────────────────── 3.00 € ─┘  │  │
  │  │                            + 0.80 €   │  │
  │  └─────────────────────────────── 3.80 € ┘  │
  │                                     × 1.5   │
  └────────────────────────────────────  5.70 € ┘
```

- **Component** (`Beverage` trait): the interface shared by base drinks *and* extras: `description()` and `cost_cents()`.
- **Concrete components** (`Espresso`, `Tea`): the base drinks.
- **Decorators** (`Milk`, `Sugar`, `WhippedCream`, `LargeSize`): each one holds another `Beverage` inside it (`inner`) and also implements `Beverage`. When asked for its price, it asks the drink inside, then adds its own part.

Because a decorated drink is still a `Beverage`, it can be wrapped again, and code that works with a `Beverage` can't tell the difference.

## The example

| Item | Type | Effect |
|---|---|---|
| Espresso | base drink | 2.50 € |
| Tea | base drink | 2.00 € |
| Milk | extra | + 0.50 € |
| Sugar | extra | + 0.10 € |
| Whipped cream | extra | + 0.80 € |
| Large | extra | × 1.5 (makes everything inside it bigger) |

The demo prints four orders, from a plain espresso to a large espresso with milk and whipped cream. Two things to notice:

- **The same extra can be added twice:** tea with two sugars is just `Sugar` wrapping `Sugar`.
- **Order matters for `LargeSize`:** "large, then milk" makes only the espresso large, while "milk, then large" makes the milk large too. The tests check both.

Prices are stored in cents (`u32`) rather than euros (`f64`), so money never suffers rounding errors.

## The Rust way

Here each decorator stores the drink it wraps as `Box<dyn Beverage>`, so any drink can be wrapped at runtime. The helper functions (`with_milk`, `large`, …) just make orders read like plain English.

You've already used decorators in Rust's standard library, even if they weren't called that:

- `BufReader::new(file)` wraps anything you can read from and adds buffering. A `BufReader` is still something you can read from.
- Iterator adapters: `numbers.iter().filter(...).map(...)` wraps one iterator in another, and each layer adds one step.

## When to use it

- You want to add features to objects in many combinations, without creating a type for every combination.
- You want to add or remove behaviour at runtime.
- Typical examples: adding logging, caching, compression or retries around an existing service.

**Watch out for:** long chains of wrappers can be hard to debug, because the behaviour is spread across many small layers.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p decorator-pattern`.
