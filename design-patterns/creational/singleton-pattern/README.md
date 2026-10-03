<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Singleton Pattern

## What is it?

The **Singleton** pattern makes sure there is **exactly one instance** of something in the whole program, and gives every part of the program a way to reach it.

Think of a country's official clock, or the one printer queue in an office. Everyone uses the same one. If each department kept its own copy of the printer queue, jobs would get lost or printed twice.

Common examples in software: the application's configuration, a logger, a connection pool, a cache.

## How it works

```text
  first call to config():
      CONFIG is empty ──► load the configuration ──► store it in CONFIG ──► return it

  every later call, from any thread:
      CONFIG is filled ──────────────────────────────────────────────────► return it
```

Two ideas make a singleton:

1. **Lazy creation:** the instance is created the first time someone asks for it, not when the program starts.
2. **One access point:** a function (`config()`) is the only way to get it, and it always returns the same instance.

## The example

**1. Application configuration.** `config()` returns the app's settings (name, maximum connections, debug mode). The first call reads them from environment variables, and every later call returns that same instance. The demo prints a message inside the loading code, so you can see it runs only once. Then four threads ask for the configuration at the same time, and all get the same memory address.

**2. A request ID generator.** `next_request_id()` hands out 1, 2, 3, … to any thread that asks. The tests run 8 threads that each take 1,000 IDs, and check that none was handed out twice.

## The Rust way

In many languages a singleton is a class with a private constructor and a static `getInstance()` method. Getting it right with multiple threads is famously tricky: two threads can create two instances at the same moment.

Rust's standard library does the hard part for you:

- **`std::sync::OnceLock`** holds a value that is set **once**. `CONFIG.get_or_init(|| ...)` runs the setup code on the first call only. If several threads call it at the same moment, one runs the setup and the others wait for it. It's safe by design.
- **`std::sync::LazyLock`** is a shortcut for the same idea, when the setup code can be written right where the `static` is declared:
  ```rust
  static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| AppConfig::load(...));
  ```
- **Atomics** (`AtomicU64`) are for simple global counters and flags. `fetch_add` reads, adds and writes back as one step that can't be interrupted, so two threads can never get the same number.

What Rust deliberately makes hard is a global you can freely **change** (`static mut`). Using one requires `unsafe`, because any thread could change it at any time. If shared data must change, wrap it in a `Mutex` inside the `OnceLock`/`LazyLock`, or use an atomic.

## Use it sparingly

A singleton is a global variable with good manners, and it brings the usual problems of globals:

- **Hidden dependencies:** a function that calls `config()` inside doesn't show in its signature that it depends on the configuration.
- **Harder testing:** every test in the same program shares the one instance, and you can't give one test different settings. That's why `AppConfig::load` takes a `lookup` function: the tests check the loading logic separately, with their own values, without touching the shared instance.

Often the better design is to create the object once in `main` and **pass it** to the code that needs it (`&AppConfig`). Reach for a singleton when that would be impractical, for example for a logger used everywhere.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p singleton-pattern`.
