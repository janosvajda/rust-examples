<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 12: Where the borrow checker is too strict

## The idea in one sentence

The borrow checker **never accepts unsafe code**, but it sometimes **rejects code that would actually be fine**, and each of those cases has a standard way to rewrite it.

## Why this happens

Checking every possible program perfectly is impossible, so the borrow checker follows simpler rules that are always safe. Being safe means erring on the side of caution: when it can't *prove* something is fine, it says no. Most of the time you won't notice. The cases below are the ones you'll actually run into.

## 1. Returning a borrow from one branch, changing the data in another

```rust
fn get_or_insert(map: &mut HashMap<u32, String>, key: u32) -> &String {
    if let Some(value) = map.get(&key) {
        return value;                   // borrow returned to the caller
    }
    map.insert(key, String::new());     // ✗ but the map isn't borrowed on this path!
    map.get(&key).unwrap()
}
```

```text
error[E0502]: cannot borrow `*map` as mutable because it is also borrowed as immutable
```

This code is correct: on the path that reaches `insert`, nothing is borrowed any more. But because the borrow from `get` is **returned** on one path, the current checker treats it as lasting until the end of the function on **every** path. Rust 1.99 still rejects it. A next-generation borrow checker, *Polonius*, is designed to accept it.

**Workaround: use the entry API**, which does the lookup and the insert as one operation, so there's only one borrow and one path:

```rust
fn get_or_insert(map: &mut HashMap<u32, String>, key: u32) -> &String {
    map.entry(key).or_insert_with(|| String::new())
}
```

It's also faster: the key is looked up once instead of up to three times. When no single-step API exists, restructure the code so that the borrow you return is only created *after* any changes, for example by checking first and borrowing at the end.

## 2. A struct that holds a reference into itself

```rust
struct Parser {
    text: String,
    current_word: &str,    // wants to point into `text`
}
```

```text
error[E0106]: missing lifetime specifier
```

There's no lifetime you could write here. When a `Parser` moves, `text` moves with it (lesson 1), so a reference into it would be left pointing at the old location. Rust can't guarantee it's valid, so it can't be expressed in safe Rust.

**Workaround: store positions instead of references.** Keep `current_word: Range<usize>`, the byte positions inside `text`, and create the `&str` only when you need it: `&self.text[self.current_word.clone()]`. Positions stay correct wherever the struct moves, and a test checks this by moving the parser into a `Box`.

(For the rare cases that really need self-references, like async code, Rust has `Pin`, which guarantees a value will never move again. It's an advanced topic of its own.)

## 3. Holding a `&mut` into a collection while changing it

```rust
let last = items.last_mut().unwrap();   // &mut into the Vec
items.push(*last);                      // ✗
```

```text
error[E0499]: cannot borrow `items` as mutable more than once at a time
```

`push` could move the `Vec`'s items (lesson 3), so a reference into it can't be held across the call. Here we only need a copy of the number anyway.

**Workaround: copy out what you need first**, so the borrow ends before the change:

```rust
if let Some(&last) = items.last() {
    items.push(last);
}
```

**The other direction: add an item, then keep changing it.** A common pattern is to push something and then fill it in:

```rust
tasks.push(title.to_string());
let new_task = tasks.last_mut().unwrap();   // find it again…
new_task.push_str(" (urgent)");
```

It works, but it looks the item up again, and the `unwrap` only *assumes* that the push worked. `push_mut` adds the item and hands back a `&mut` to it in one step:

```rust
let new_task = tasks.push_mut(title.to_string());
new_task.push_str(" (urgent)");
```

As long as `new_task` is in use, it borrows `tasks` mutably, so you can't change `tasks` in any other way. That's the same rule as above, now working for you. `VecDeque` has `push_front_mut` and `push_back_mut`, and `Vec` also has `insert_mut`.

## 4. Data that points at itself: graphs and cycles

A graph where nodes refer to each other, especially in cycles (A → B → C → A), can't be built from plain `&` references. Every node would borrow the others, so nothing could ever be changed or freed.

**Workaround: store the nodes in a `Vec` and link them by index.** A link is just a `usize`, so cycles are no problem and the whole graph is owned by one value. This is the *arena* pattern, used throughout this repository's [data structures](../../data-structures/) (the linked list, the LRU cache and the graphs). The alternative is `Rc<RefCell<…>>` with `Weak` for back-links (see the doubly linked list there), which is more flexible but more complicated.

## A general strategy

When the borrow checker rejects code you believe is correct:

1. **Shorten the borrow.** Copy or clone the small piece you need, so the borrow ends sooner.
2. **Split the borrow.** Borrow separate fields, or use `split_at_mut` (lesson 6).
3. **Use an API that does both steps at once**, like `entry`, `retain` or `mem::take`.
4. **Store indices instead of references** when data needs to point at other data.
5. **Use runtime checks** (`RefCell`, `Mutex`) only when the structure really needs shared mutation.

`unsafe` is almost never the answer to a borrow checker error. The workarounds above are what experienced Rust programmers reach for.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 11: Cow and the borrowing traits](../11-cow-and-borrowing-traits/) · Back to the [course overview](../)
