<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: `Fn`, `FnMut` and `FnOnce`

## The idea in one sentence

Every closure implements one or more of three traits, depending on **what it does with the things it captured**, and that decides how often, and how, it can be called.

## The three traits

| Trait | The closure… | Can be called | Example |
|---|---|---|---|
| **`Fn`** | only **reads** what it captured | any number of times, even from several places at once | `\|\| greeting.clone()` |
| **`FnMut`** | **changes** what it captured | many times, but one call at a time | `\|n\| total += n` |
| **`FnOnce`** | **gives away** what it captured | **once** | `move \|\| name` (returns `name` itself) |

They're nested: every `Fn` closure is also `FnMut`, and every `FnMut` closure is also `FnOnce`:

```text
  ┌─ FnOnce: every closure ────────────────────────────────────┐
  │                                                            │
  │   ┌─ FnMut: closures that don't give anything away ─────┐  │
  │   │                                                     │  │
  │   │   ┌─ Fn: closures that don't change anything ────┐  │  │
  │   │   │                                              │  │  │
  │   │   └──────────────────────────────────────────────┘  │  │
  │   └─────────────────────────────────────────────────────┘  │
  └────────────────────────────────────────────────────────────┘
```

The compiler decides automatically which traits a closure implements, based on its body. `move` doesn't change that: `move || greeting.len()` owns `greeting` but only reads it, so it's still `Fn`.

Closures can also be **`Copy`** and **`Clone`**, if everything they capture is. A closure that only holds shared references, like `|| greeting.clone()`, is `Copy`, so passing it to a function copies it and you can keep using the original. The demo does exactly that.

## Taking closures as parameters: ask for as little as you need

```rust
fn call_twice<F: Fn() -> String>(f: F)          // calls it twice → needs Fn
fn for_each_item<F: FnMut(i32)>(…, mut f: F)    // calls it repeatedly → FnMut is enough
fn run_once<F: FnOnce() -> String>(f: F)        // calls it once → FnOnce accepts anything
```

The **less** a function demands, the **more** closures callers can pass. Pick the most permissive trait your code allows:
- call it once → `FnOnce`;
- call it many times, one after another → `FnMut`;
- call it from several places at once, e.g. several threads → `Fn`.

The standard library follows this rule. `Option::unwrap_or_else` calls its closure at most once, so it takes `FnOnce`, which means you can move things inside it. `Iterator::for_each` takes `FnMut`.

## The errors, and what they mean

**Calling an `FnMut` closure that isn't stored as `mut`:**

```text
error[E0596]: cannot borrow `increment` as mutable, as it is not declared as mutable
```

Calling an `FnMut` closure changes its captured state, so the variable holding it must be `let mut`.

**Calling an `FnOnce` closure twice:**

```text
error[E0382]: use of moved value: `give_away`
```

**Passing a closure that needs more than the function allows.** The message depends on how you pass it. Written **directly** in the call, the compiler treats the closure as the required `Fn` and points at the line that breaks it:

```text
error[E0594]: cannot assign to `count`, as it is a captured variable in a `Fn` closure
error[E0507]: cannot move out of `name`, a captured variable in an `Fn` closure
```

Stored in a **variable first**, its traits are already decided, so you get the general message:

```text
error[E0525]: expected a closure that implements the `Fn` trait, but this closure only implements `FnOnce`
```

They're the same mistake. Either change the closure so it does less, for example `.clone()` instead of giving the value away, or change the function to accept `FnMut` or `FnOnce`.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: Closures](../01-closures/) · Next: [Lesson 3: Storing and returning closures](../03-storing-and-returning-closures/)
