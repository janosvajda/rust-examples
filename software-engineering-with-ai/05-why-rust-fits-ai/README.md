<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Why Rust fits AI-assisted development

> Written in October 2026. AI tools, and how well they write each language, change every month. Check whether what you read here still holds.

## The idea in one sentence

When an AI writes the code, the hard part is **checking** it, and Rust's compiler checks more, earlier and more precisely than almost any other mainstream language, on every single build, for free.

## The feedback loop

An AI coding agent works in a loop:

```text
   write code ──► build ──► read the errors ──► fix ──► build again …
```

The more mistakes the **build** can find, the more the AI fixes on its own, before a human ever looks. In many languages, the build checks little, and mistakes show up only when the program runs, maybe in front of a user. Rust moves many of them to compile time:

| The mistake | Typical dynamically typed language, no extra tools | Rust |
|---|---|---|
| using a value that might be missing (`null`, `None`) | crashes at runtime, when it happens | **compile error** |
| forgetting to handle a new case | silently does nothing, or crashes | **compile error** |
| two threads changing the same data | rare, random corruption at runtime | **compile error** |
| using memory after it was freed (C, C++) | random crashes, security holes | **compile error** |
| passing the wrong type | runtime error, if that line ever runs | **compile error** |

Some languages close part of this gap with extra tools, for example TypeScript or Python's type checkers. Rust has it built in, and you can't switch it off, except inside `unsafe`.

## What the compiler catches: real errors

Every message below is the real output of Rust 1.99.

### "This might be missing"

```rust
fn find_user(id: u32) -> Option<String> { … }

let name: String = find_user(2);
```

```text
error[E0308]: mismatched types
expected `String`, found `Option<String>`
```

The type `Option<String>` says "maybe there's no user". The compiler won't let anyone, human or AI, forget that. In many languages, this is the famous "null pointer" crash.

### "You forgot a case"

An AI adds a new payment method in one file:

```rust
enum Payment {
    Card,
    Cash,
    Voucher, // new
}
```

Every `match` on `Payment` elsewhere in the project now fails to compile:

```text
error[E0004]: non-exhaustive patterns: `Payment::Voucher` not covered
```

This is gold for AI-assisted work. When an AI changes something in one place, the compiler produces a **complete list of every other place that must change too**. An agent works through that list until it's empty. In a language without this check, the forgotten places are found by users.

### "Two threads, one piece of data"

```rust
let mut log = Vec::new();
let worker = thread::spawn(|| log.push("from the thread"));
log.push("from main");
```

```text
error[E0373]: closure may outlive the current function, but it borrows `log`, which is owned by the current function
error[E0499]: cannot borrow `log` as mutable more than once at a time
```

```rust
let counter = Rc::new(5);
thread::spawn(move || println!("{counter}"));
```

```text
error[E0277]: `Rc<i32>` cannot be sent between threads safely
```

Concurrency bugs are among the hardest to find by testing, because they depend on timing and might appear once in a million runs. Rust refuses them while compiling. (The [concurrency course](../../concurrency/) and the [ownership and borrowing course](../../ownership-and-borrowing/) explain how.)

## More reasons Rust fits

- **Precise error messages.** They show the exact line, explain the problem, and often suggest a fix. That's useful for a human and very useful for an AI, which reads them literally.
- **Types are a specification.** `fn parse_port(line: &str) -> Result<u16, String>` already tells you, and the AI, that the function can fail and what it returns. The signature is a contract the compiler enforces.
- **One standard toolchain.** Every Rust project builds, tests, lints, formats and documents itself the same way: `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`, `cargo doc`. An agent knows how to check **any** Rust project without project-specific instructions.
- **Tests are built in.** `#[test]` and `cargo test` need no extra framework, so there's no excuse not to have them (see [Lesson 6](../06-clippy-and-tests-as-guardrails/)).
- **Safe refactoring.** Rename a field, change a type or add a variant, and the compiler finds every place that's affected. Large changes, the kind AIs make quickly, are much less scary.

## The honest downsides

Rust isn't magic, and this course wouldn't be honest without these:

- **"It compiles" doesn't mean "it's correct."** Every plausible function in [Lesson 4](../04-why-you-still-need-to-know/) compiled. The compiler checks **types and memory**, not **logic**. An average that rounds wrongly or a discount that doesn't round to the nearest cent compiles fine. That's what tests are for.
- **The compiler's suggestion isn't always the right fix.** For the "might be missing" error above, the compiler suggests:

  ```text
  help: consider using `Option::expect` to unwrap the `Option<String>` value, panicking if the value is an `Option::None`
  ```

  That makes the error go away, and turns it into a **crash** when the user isn't found. An AI that follows every hint literally ends up with `expect`, `unwrap`, `clone` and `'static` everywhere. The hint is a suggestion. Deciding what *should* happen when there's no user is still your job.
- **Fighting the borrow checker.** When an AI can't satisfy the borrow checker, it often reaches for `.clone()`, `Rc<RefCell<…>>` or `Arc<Mutex<…>>` until the errors stop. The code compiles, but it's slower and more complicated than it needs to be. Recognising this takes knowledge of ownership.
- **Slower feedback.** Compiling Rust takes longer than starting a script, so each round of the loop takes longer. The trade is usually worth it, because each round catches more.
- **AIs may write Rust less fluently.** There's far more Python and JavaScript in the world than Rust, and in our experience AIs still stumble more often on lifetimes, async Rust and trait bounds. This is changing quickly.
- **`unsafe` switches the guarantees off.** Inside an `unsafe` block, the compiler trusts you. AI-written `unsafe` deserves the most careful review of all.

## In short

AI makes writing code cheap and checking it expensive. A language whose compiler does a large part of the checking automatically, precisely and on every build is a natural partner for AI-written code. Rust doesn't replace tests, review or knowledge, but it removes whole classes of bugs before any of those are needed.

## What may change

Other languages are adding stronger checks, AI models are getting better at Rust, and new tools may close some gaps. The underlying idea is likely to last: **the more a machine can verify, the more safely you can let a machine write.**

Previous: [Lesson 4: Why you still need to know](../04-why-you-still-need-to-know/) · Next: [Lesson 6: Clippy and tests as guardrails](../06-clippy-and-tests-as-guardrails/)
