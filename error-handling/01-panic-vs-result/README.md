<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 1: panic vs Result

## The idea in one sentence

Rust separates **bugs** (use `panic!`) from **expected failures** (return a `Result`), and makes you decide which one you're dealing with.

## Two kinds of "something went wrong"

| | Expected failure | Bug |
|---|---|---|
| Examples | the user typed "eighty" as a port; the file doesn't exist; the network is down | a percentage of 150; an index you were sure was valid; a "this can't happen" branch |
| Who's at fault | nobody: the world is like that | the programmer |
| Rust tool | **`Result<T, E>`** | **`panic!`** (and `assert!`, `unwrap`, `expect`) |
| What happens | the caller gets an error value and decides | the thread stops, printing a message |

There are **no exceptions** in Rust. An error can't jump invisibly past functions. If a function can fail, its return type says so, and the caller has to deal with it.

## `Result`: failure in the signature

```rust
enum Result<T, E> {
    Ok(T),    // it worked: here's the value
    Err(E),   // it failed: here's why
}

fn parse_port(text: &str) -> Result<u16, ParseIntError>
```

The caller can't use the `u16` without first checking which case it got:

```rust
match parse_port(input) {
    Ok(port) => …,
    Err(error) => …,
}
```

Ignoring a `Result` completely gets a warning, because `Result` is marked `#[must_use]`:

```text
warning: unused `Result` that must be used
```

## `Option`: missing isn't an error

`Option<T>` (`Some(value)` or `None`) is for things that may simply not be there, like looking up a user who doesn't exist. Use `Result` when the caller needs to know **why** something failed, and `Option` when "not there" is the whole story.

## `unwrap` and `expect`: shortcuts that panic

```rust
parse_port("443").unwrap()                    // 443
parse_port("4x3").unwrap()                    // panics
parse_port("80").expect("80 is a valid port") // panics with your message if it fails
```

The panic messages, exactly as Rust 1.99 prints them:

```text
called `Result::unwrap()` on an `Err` value: ParseIntError { kind: InvalidDigit }
the port must be a number: ParseIntError { kind: InvalidDigit }
```

`unwrap` is fine in tests and quick experiments. In real code, prefer `expect` with a message explaining **why** you're sure it can't fail. Or better, don't panic at all.

**Fallbacks that never panic:**

| Method | On `Err` / `None` it gives |
|---|---|
| `.unwrap_or(8080)` | the value you supply |
| `.unwrap_or_default()` | the type's default (`0` for numbers, `""` for strings) |
| `.unwrap_or_else(\|e\| …)` | the result of a closure, computed only when needed |

## When `panic!` is the right answer

Panic when continuing would be **wrong**, not just inconvenient. That usually means the caller broke a promise, so the program has a bug:

```rust
fn apply_discount(price_cents: u32, percent: u32) -> u32 {
    assert!(percent <= 100, "discount must be 0..=100, got {percent}");
    …
}
```

A panic stops the current thread and prints the message, plus a backtrace if you set `RUST_BACKTRACE=1`. If it's the main thread, the program exits with code **101**.

**Rule of thumb:** if a *user* or the *outside world* can cause it, return a `Result`. If only a *programmer mistake* can cause it, panicking is acceptable.

## Run it

```bash
cargo run
cargo test
```

The tests use `#[should_panic]` to prove that `unwrap` on an error and a broken promise really do panic.

Next: [Lesson 2: The `?` operator](../02-the-question-mark/)
