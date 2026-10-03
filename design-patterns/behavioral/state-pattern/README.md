<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# State Pattern

## What is it?

The **State** pattern makes an object behave differently depending on what state it's in, and keeps the rules for moving between states in one clear place.

Think of a parcel you ordered online. What you're allowed to do with it depends on where it is:

- Before you pay, you can pay or cancel.
- After you pay, it can be shipped, or you can still cancel and get a refund.
- Once it's shipped, it's too late to cancel.
- Once it's delivered, the story is over.

The order is the same object the whole time, but the same action ("cancel") succeeds in one state and is refused in another.

## The states and transitions

```text
              pay()             ship()               deliver()
  Pending ───────────► Paid ───────────► Shipped ───────────► Delivered
     │                  │
     │ cancel()         │ cancel()  (with refund)
     ▼                  ▼
  Cancelled ◄───────────┘
```

Every arrow is an allowed action. Anything else, like shipping an unpaid order or cancelling a delivered one, is refused with an error, and the order stays as it was.

## The example

`Order` holds its current state in an `OrderState` enum:

| State | Extra data it carries |
|---|---|
| `Pending` | – |
| `Paid` | the amount paid |
| `Shipped` | the tracking number |
| `Delivered` | – |
| `Cancelled` | the reason |

Each action (`pay`, `ship`, `deliver`, `cancel`) checks the current state with a `match`, then either moves to the next state or returns an `InvalidAction` error explaining what was refused. `status_message` shows that ordinary behaviour, not just transitions, can depend on the state too.

The demo walks one order through the happy path, then tries some actions that aren't allowed, and finally cancels a paid order, which triggers a refund.

## The Rust way: enums

In many languages the State pattern means one class per state, each implementing a common interface. Rust's **enums** make it simpler:

- **All states are listed in one place**, the `OrderState` enum.
- **Each state carries only the data that belongs to it.** A tracking number only exists once the order is `Shipped`, so you can't accidentally read one from a `Pending` order. The compiler won't let you.
- **`match` must handle every state.** If you add a new state, like `Returned`, the compiler points out every `match` that needs updating.

The one-type-per-state version (a trait with a struct per state, stored as `Box<dyn State>`) is still useful when states are complex or added by other code, such as plugins. The Rust Book's chapter on object-oriented design patterns walks through that version with a blog post example.

There's also a third, Rust-specific variant called **typestate**, where each state is a separate *type* (`Order<Pending>`, `Order<Paid>`, …). Then calling `ship()` on an unpaid order isn't a runtime error: the program doesn't compile at all.

## When to use it

- An object's behaviour depends on its state, and there are rules about which changes are allowed.
- You notice the same `if status == ...` checks spread across many methods.

Typical examples: orders, documents with a draft → review → published workflow, network connections, game characters, traffic lights.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p state-pattern`.
