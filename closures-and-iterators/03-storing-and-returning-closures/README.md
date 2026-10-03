<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Storing and returning closures

## The idea in one sentence

A closure's type has no name you can write, so to **return** one or **keep** one in a struct you use either `impl Fn(…)` (one specific closure, fastest) or `Box<dyn Fn(…)>` (any closure with that signature, most flexible).

## Returning a closure: `impl Fn`

```rust
fn make_adder(n: i32) -> impl Fn(i32) -> i32 {
    move |x| x + n
}

let add_five = make_adder(5);
add_five(1)      // 6
```

`impl Fn(i32) -> i32` means *"some closure that takes an `i32` and returns an `i32`"*. The caller can call it, and the compiler knows the exact type behind it, so there's no extra cost.

**`move` is required.** The closure must keep its own copy of `n` after `make_adder` has returned. Without `move`:

```text
error[E0373]: closure may outlive the current function, but it borrows `n`, which is owned by the current function
```

Returned closures can carry **state**. `make_counter()` returns an `impl FnMut() -> u32` that counts up each time it's called. Every counter you create has its own count.

## Returning one of several closures: `Box<dyn Fn>`

```rust
fn make_operation(name: &str, n: i32) -> Option<Box<dyn Fn(i32) -> i32>> {
    match name {
        "add"      => Some(Box::new(move |x| x + n)),
        "multiply" => Some(Box::new(move |x| x * n)),
        …
    }
}
```

Different branches create **different** closure types (lesson 1), and `impl Fn` means *one* type:

```text
error[E0308]: `if` and `else` have incompatible types
  = note: no two closures, even if identical, have the same type
```

`Box<dyn Fn(i32) -> i32>` is a trait object (traits course, lesson 3): it can hold any closure with that signature, at the cost of one heap allocation and an indirect call.

## Storing closures in structs

**One closure, known when the struct is created: a generic field.**

```rust
struct Memo<F: Fn(u64) -> u64> {
    calculation: F,
    cache: HashMap<u64, u64>,
}
```

Each `Memo` is specialised for its closure, with no box and no indirection. Call a closure stored in a field with **parentheses around the field**: `(self.calculation)(input)`. Plain `self.calculation(input)` would look for a *method* called `calculation`.

The demo uses `Memo` to cache a slow calculation: five requests, but only two real calculations.

**Many different closures: `Vec<Box<dyn Fn>>`.**

```rust
type Handler = Box<dyn Fn(&str) -> String>;    // name the long type once

struct Button {
    handlers: Vec<Handler>,
}

button.on_click(|label| format!("clicked {label}"));
button.on_click(move |label| format!("{user} pressed {label}"));
```

Each handler is a different closure, so they can only share a `Vec` as trait objects. This is the classic callback / event-handler pattern. The `'static` bound on `on_click` says a handler can't borrow short-lived data, because the button may keep it for a long time. Use `move` to give it its own copies.

## Which to choose

| Situation | Use |
|---|---|
| return a closure; always the same kind | `impl Fn(…)` |
| return one of several different closures | `Box<dyn Fn(…)>` |
| a struct holds one closure, fixed at creation | a generic field `F: Fn(…)` |
| a struct holds many, or changes them later | `Vec<Box<dyn Fn(…)>>` |
| the closure captures nothing | a plain `fn(…)` pointer works too (lesson 1) |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: Fn, FnMut and FnOnce](../02-fn-traits/) · Next: [Lesson 4: The iterator toolbox](../04-iterator-toolbox/)
