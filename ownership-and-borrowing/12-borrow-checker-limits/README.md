<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 12: Where the borrow checker is too strict

## The idea in one sentence

The borrow checker can reject some correct programs because its current analysis cannot prove their borrowing relationships. Other rejections enforce necessary safety rules; learning the difference helps you choose a good rewrite.

Imagine a careful librarian who needs clear proof before lending a book. A rejected request can mean either "this conflicts with another loan" or "I need a clearer plan".

## Why this happens

The compiler uses conservative rules for safe references. It cannot recognise every possible correct program, so sometimes clearer structure or a purpose-built API is needed.

Rust **does accept `unsafe` code**. Ordinary reference borrowing checks still apply inside an `unsafe` block, but additional operations such as raw-pointer dereferences need safety guarantees from the programmer. Successful compilation alone does not prove those operations are sound.

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

This code is correct: on the path that reaches `insert`, nothing is borrowed any more. But because the borrow from `get` is **returned** on one path, the current checker treats it as lasting until the end of the function on **every** path. Stable Rust 1.99 still rejects it. The **Polonius** borrow-checking work adds more precise analysis for cases like this; a preview called Polonius Alpha was enabled on nightly Rust in August 2026. [Rust team's announcement](https://blog.rust-lang.org/2026/08/04/enabling-polonius-alpha-on-nightly/)

**Workaround: use the entry API**, which combines lookup and optional insertion under one mutable borrow:

```rust
fn get_or_insert(map: &mut HashMap<u32, String>, key: u32) -> &String {
    map.entry(key).or_default()
}
```

It also avoids repeated lookup calls: the original missing-key path calls `get`, `insert` and `get` again. That is a useful efficiency improvement; actual timings depend on the workload. When no single-step API exists, restructure the code so that the borrow you return is only created *after* any changes, for example by checking first and borrowing at the end.

Here, `or_default()` inserts an empty `String` when the key is missing. The runnable lesson uses `or_insert_with(|| format!("item {key}"))` to name a new entry instead.

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

Adding `<'a>` and `&'a str` names a lifetime, but does not express "this reference always borrows the sibling `text` field". Constructing a self-contained parser that can be freely returned and moved while maintaining that internal borrow needs more than an ordinary lifetime parameter.

**Moving a `String` does not move its heap text**—lesson 1 shows that the buffer address stays the same. The risks include dropping or replacing the owner, editing or reallocating the buffer, and maintaining the internal relationship during construction. A reference into an inline array would have the additional problem that moving the array changes its location.

Safe Rust can create a local self-borrow after initialisation and prevent conflicting access while it is needed. This limited example works, but you cannot then freely move the parser and keep using its stored borrow:

```rust
struct BorrowingParser<'a> { text: String, word: &'a str }
let mut parser = BorrowingParser { text: String::from("hello world"), word: "" };
parser.word = &parser.text[..5];
assert_eq!(parser.word, "hello");
```

**Workaround: store positions instead of references.** Keep `current_word: Range<usize>`, the byte positions inside `text`, and create the `&str` only when you need it: `&self.text[self.current_word.clone()]`. Moving this parser preserves those byte positions; a test moves it into a `Box`. Its methods leave the original text unchanged. If you add text editing, you must maintain valid bounds and UTF-8 boundaries for the stored ranges.

This small parser treats **ASCII spaces** as separators. `"a  b "` produces `"a"`, `""`, `"b"`, `""`; a tab stays inside its segment, and empty input has one empty segment. That keeps its byte-offset logic easy to inspect. A general word parser needs an explicit whitespace or language-aware definition of a word.

Some address-sensitive types, including certain compiler-generated async futures, use **pinning**. A correctly pinned pointee that does not implement `Unpin` must remain at its address until it is dropped. The `Pin<Box<T>>` handle can still move, and `Unpin` types do not acquire that movement restriction. `Pin` alone does not express the sibling-field lifetime relationship or prevent a `String` buffer from reallocating; safe APIs must preserve the relevant invariants. [Pin documentation](https://doc.rust-lang.org/std/pin/index.html)

## 3. Holding a `&mut` into a collection while changing it

```rust
let last = items.last_mut().unwrap();   // &mut into the Vec
items.push(*last);                      // ✗
```

```text
error[E0499]: cannot borrow `items` as mutable more than once at a time
```

This expression requires overlapping mutable borrows while evaluating the call: `last` provides mutable access to an element, while `push` needs mutable access to the vector. Rust rejects that overlap. We only need a copy of the `i32`, so we can make the access order explicit.

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

This is valid: if `push` returns normally, the vector contains the added item, so the following `last_mut()` returns `Some`. Accessing the last item is constant-time; there is no search through the vector. `push_mut` expresses adding an item and borrowing it in one operation:

```rust
let new_task = tasks.push_mut(title.to_string());
new_task.push_str(" (urgent)");
```

As long as `new_task` is in use, it borrows `tasks` mutably, so you can't change `tasks` in any other way. That's the same rule as above, now working for you. `VecDeque` has `push_front_mut` and `push_back_mut`, and `Vec` also has `insert_mut`.

## 4. Data that points at itself: graphs and cycles

Graphs **can** use references. Externally owned immutable nodes can borrow other nodes, and even a cycle can use static references:

```rust
struct Node { name: &'static str, next: &'static Node }
static A: Node = Node { name: "A", next: &B };
static B: Node = Node { name: "B", next: &A };
assert_eq!(A.next.next.name, "A"); // A → B → A
```

A freely movable, owned graph with editable nodes and reference links is harder: it must maintain node lifetimes and avoid invalidating links. Indices make that ownership arrangement straightforward.

**Workaround: store the nodes in a `Vec` and link them by index.** A link is just a `usize`, so cycles are no problem and the whole graph is owned by one value. This is one form of the **arena** pattern: a collection owns nodes and identifiers link them. The demo only adds nodes; removing or reordering vector elements would require maintaining the indices, or using identifiers with generation checks.

Another approach is `Rc<RefCell<…>>` with carefully chosen `Weak` links to avoid strong ownership cycles. Reference links plus interior mutability are also possible when lifetimes permit. See the repository's [data structures](../../data-structures/) for different designs.

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

For more detail: [unsafe Rust](https://doc.rust-lang.org/book/ch20-01-unsafe-rust.html), [HashMap entry](https://doc.rust-lang.org/std/collections/struct.HashMap.html#method.entry) and [Vec::push_mut](https://doc.rust-lang.org/std/vec/struct.Vec.html#method.push_mut).

Previous: [Lesson 11: Cow and the borrowing traits](../11-cow-and-borrowing-traits/) · Back to the [course overview](../)
