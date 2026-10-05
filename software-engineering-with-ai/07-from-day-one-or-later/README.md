<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 7: From day one, or later?

> Written in October 2026. AI tools change every month. Check whether what you read here still holds.

## The idea in one sentence

Add clippy and tests in the **first commit**: then they cost almost nothing, while adding them later costs a lot, and with AI, "later" arrives very quickly.

## Our opinion: from day one

"We'll add tests and linting once the project is real" sounds reasonable. In practice, it means they never get added, or get added painfully. Here's why:

| | On day one | Six months later |
|---|---|---|
| **Clippy** | 0 warnings; each new one is fixed in seconds, while you remember the code | hundreds of warnings at once; the usual result is that clippy gets switched off |
| **Tests** | you test the function you just wrote, while you know what it should do | you test code whose correct behaviour nobody remembers, maybe not even the AI that wrote it |
| **Design** | testable code from the start: small functions, clear inputs and outputs | code that's hard to test, because nobody had to test it |
| **AI agents** | the agent checks its own work after every change | the agent changes untested code blindly |

With AI, the right column comes **sooner**. A project that once grew by a few hundred lines a week can now grow by thousands a day, and the debt grows at the same speed.

## Setting it up takes five minutes

**1. Create the project:**

```bash
cargo new my-project
cd my-project
```

**2. Choose your extra lints** in `Cargo.toml` (see [Lesson 6](../06-clippy-and-tests-as-guardrails/)):

```toml
[lints.clippy]
unwrap_used = "deny"
todo = "deny"
dbg_macro = "deny"
```

**3. Write the first test** next to the first function:

```rust
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[cfg(test)]
mod tests {
    #[test]
    fn adds() {
        assert_eq!(super::add(2, 3), 5);
    }
}
```

**4. Make the computer check it on every change**, for example with GitHub Actions in `.github/workflows/ci.yml`:

```yaml
name: CI
on: [push, pull_request]
jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo fmt --check
      - run: cargo clippy --all-targets -- -D warnings
      - run: cargo test
```

`-D warnings` turns every clippy warning into an error, so a warning can't quietly stay.

**5. Tell your AI tools.** Put the rules in the project instruction file that your AI coding tool reads: "run `cargo clippy --all-targets -- -D warnings` and `cargo test` after every change; never weaken a lint or change a test without saying why."

That's it. From now on, every change, human or AI, passes the same checks.

### This repository is an example

This repository's CI runs `cargo clippy --all-targets -- -D warnings`, and that step stopped real pull requests. In one, an inline-assembly operand was formatted as a full 64-bit register where a 32-bit one was meant; that's a compiler warning, which `-D warnings` turned into an error. In another, clippy's `result_large_err` lint found an error type so large that every `Result` carrying it had to copy a lot of memory on each return. Neither broke the build or a test. Only that CI step noticed.

## The exception: a real prototype

Sometimes you really are just trying something out: does this idea work at all, is this library usable? Then skipping clippy and tests is reasonable. That's the "fine" side of [vibe coding](../01-what-is-vibe-coding/).

Two rules make the exception safe:

1. **Decide it on purpose, and write it down.** Put "PROTOTYPE: no tests, will be thrown away or rewritten" at the top of the README. A prototype nobody called a prototype becomes the product by accident.
2. **When the prototype survives, add the checks *before* the next feature.** That means:
   - turn on clippy and fix what it finds;
   - write tests for the behaviour you're keeping;
   - add CI.

   Not "someday": the next task.

## Adding them to an existing project

Maybe it's already six months later. Don't switch everything on at once: hundreds of failures at once can make the tool's output easy to ignore. Go step by step:

1. **Fix the errors first.** Run `cargo clippy` and fix everything in the error ("correctness") group. Those are likely real bugs.
2. **Stop new warnings, then reduce old ones.** Add `-D warnings` to CI so no *new* warning gets in. Where an old warning can't be fixed today, allow it right there, with a reason:
   ```rust
   #[allow(clippy::too_many_arguments)] // TODO(#123): group these into a Config struct
   ```
   Then remove those `allow`s one at a time. A count that only goes down is called a **ratchet**.
3. **Before changing old code, pin its current behaviour.** Write tests that record what the code does **now**, even where you're not sure it's right. These are called **characterization tests**.

   They're the one place where copying the output into a test, the trap from Lesson 6, is the right thing to do. The goal here isn't to prove the code correct. It's to **notice when a change alters something**. When such a test fails, find out whether the change was intended.
4. **Test the parts you touch.** You don't need to test the whole old project at once. Every bug fix gets a test that would have caught it, and every new feature arrives with tests. Coverage grows where the work happens.

AI is a great help here. Writing characterization tests, fixing hundreds of mechanical lint warnings and explaining old code are boring, careful work that AI does well. A human still reviews it.

## What may change

AI tools may soon set up checks like these automatically, or new tools may replace some of them. The reason behind this lesson won't change: **checks are cheapest at the start, and most valuable when code grows fast.**

Previous: [Lesson 6: Clippy and tests as guardrails](../06-clippy-and-tests-as-guardrails/) · Next: [Lesson 8: Fewer dependencies](../08-fewer-dependencies/)
