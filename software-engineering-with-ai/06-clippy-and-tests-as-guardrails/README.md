<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: Clippy and tests as guardrails

> Written in October 2026. AI tools change every month. Check whether what you read here still holds.

## The idea in one sentence

Clippy and tests are reviewers that never get tired. They matter even more when an AI writes the code, because there's more code than anyone reads carefully, and because the AI itself uses their messages to fix its mistakes.

## Why they matter more with AI

1. **There's more code.** An AI can write in a minute what used to take a day. Nobody reads every line of that with equal care. Tools do.
2. **The AI uses them too.** A coding agent that runs `cargo clippy` and `cargo test` after each change reads the errors and fixes its own mistakes, often before you see them. Every check you add makes the AI better at your project.
3. **Rust is a good fit.** Rust's compiler already refuses many bugs that other languages only find while the program runs: use-after-free, data races, a forgotten `None` case. Clippy and tests add the next layer on top. [Lesson 5](../05-why-rust-fits-ai/) explains this in detail.

## Clippy: what it catches

Here's the kind of code that's easy to write, or to accept from an AI. Every message below is the real output of clippy on Rust 1.99:

```rust
fn total(prices: &Vec<u32>) -> u32 {
    let mut sum = 0;
    for i in 0..prices.len() {
        sum += prices[i];
    }
    sum
}

fn is_valid_age(age: u32) -> bool {
    if age < 0 {
        return false;
    }
    true
}

fn first_word(text: &str) -> String {
    let words: Vec<&str> = text.split(' ').collect();
    if words.len() == 0 {
        return String::new();
    }
    words[0].to_string()
}
```

```text
warning: writing `&Vec` instead of `&[_]` involves a new object where a slice will do
warning: the loop variable `i` is only used to index `prices`
error: this comparison involving the minimum or maximum element for this type contains a case that is always true or always false
warning: length comparison to zero
```

| Message | What it means |
|---|---|
| `&Vec` instead of `&[_]` | take a slice, so callers can pass any list, not only a `Vec` |
| loop variable only used to index | write `for price in prices`: simpler, and no index to get wrong |
| **comparison … always true or always false** | `age` is a `u32`, so it can **never** be below 0. This check does nothing, and that's a sign the author misunderstood the type. Clippy rejects it with an **error**, not a warning. |
| length comparison to zero | write `words.is_empty()` |

The first, second and fourth are about clean, idiomatic code. The third is a real **logic bug** hiding in code that compiles. Clippy groups its checks, and the "correctness" group, code that is almost certainly wrong, is an error by default.

### Stricter rules for your project

Clippy has more checks than it turns on by default. You choose extra ones for your project in `Cargo.toml`, written down once and applied everywhere. This lesson's crate does exactly that:

```toml
[lints.clippy]
unwrap_used = "deny"   # no .unwrap(): handle the error or use .expect("why it can't fail")
todo = "deny"          # no todo!() left behind
dbg_macro = "deny"     # no forgotten dbg!() calls
```

These are shortcuts that are easy to take and easy to miss in a review. Now, if anyone, human or AI, writes `.unwrap()` in this crate:

```text
error: used `unwrap()` on a `Result` value
```

Choose rules that match **your** project: a small script doesn't need what a payment service needs.

## Tests: the trap of copying the output

Here's a specification, written by a person:

> Apply a percentage discount to a price given in cents. The result is rounded to the nearest cent. A discount above 100% is an error.

An AI writes this:

```rust
fn discounted_price(price_cents: u32, percent: u32) -> u32 {
    price_cents - price_cents * percent / 100
}
```

And then a test, by **running the code and copying what it printed**:

```rust
#[test]
fn discount() {
    assert_eq!(discounted_price(999, 15), 850);   // ✓ passes
}
```

The test passes, but the code is wrong. Work it out by hand: €9.99 minus 15% is €8.4915, which rounds to **€8.49**, so 849 cents, not 850. The code cuts off the fraction of the *discount* instead of rounding the *price*. The copied test **confirms the bug**, because it was written from the code, not from the specification.

A test written from the specification catches it:

```rust
#[test]
fn rounds_to_the_nearest_cent() {
    assert_eq!(discounted_price(999, 15), 849);   // worked out by hand
}
```

```text
assertion `left == right` failed
  left: 850
 right: 849
```

The other rule fails too. A discount of 101% should be an error, and instead the program crashes:

```text
attempt to subtract with overflow
```

**The rule: a test must say what the code *should* do, and that answer must come from the specification, not from running the code.** An AI writing both the code and the tests can make the same mistake twice, and then the two agree with each other perfectly.

### Watch out for "fixing" the test

When a test fails, there are two possibilities: the code is wrong, or the test is wrong. An AI asked to "make the tests pass" sometimes changes the **test**, because that's the shortest path to green. Treat every change to an existing test as a question: **why did the expected value change?** In your prompts, say it outright: "don't change the tests without telling me why."

### Property tests: rules instead of examples

A few hand-picked examples can miss rare cases. A **property test** checks a rule that must hold for **every** input, across thousands of inputs. This crate checks two rules from the specification for over 10,000 combinations:

```rust
for price in (0..=100_000).step_by(997) {
    for percent in 0..=100 {
        let now = discounted_price(price, percent).expect("0-100% is always valid");
        assert!(now <= price);         // a discount never raises the price
        assert!(now <= previous);      // more discount never costs more
        …
    }
}
```

These rules come straight from the meaning of "discount", and they're hard to get accidentally wrong. Crates like `proptest` go further and invent the inputs for you.

## The correct version

[`src/main.rs`](src/main.rs) contains the version that meets the specification:
- it returns an error above 100%;
- it works in `u64` so large prices can't overflow;
- it rounds to the nearest cent by adding half a cent before dividing.

Every expected value in its tests was worked out by hand.

```text
  999 cents,  15% off → Ok(849)
 1000 cents, 101% off → Err("a discount of 101% is more than the whole price")
```

## Run it

```bash
cargo run
cargo test
cargo clippy
```

## What may change

The tools will grow. Clippy gains new checks with almost every Rust release, and AI tools increasingly generate, run and judge tests themselves. The principle stays: **automatic checks catch what humans miss, and a human must make sure the checks test the right thing.**

Previous: [Lesson 5: Why Rust fits AI-assisted development](../05-why-rust-fits-ai/) · Next: [Lesson 7: From day one, or later?](../07-from-day-one-or-later/)
