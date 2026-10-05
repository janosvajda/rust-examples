<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 6: Raw pointers and `unsafe`

## The idea in one sentence

A raw pointer is an address with no owner and no guarantees, so the compiler can't check it. Reading through one is `unsafe`, which doesn't switch the rules off: it means **you** promise they hold, and a good `unsafe` block says exactly why.

## References vs raw pointers

| | `&T` / `&mut T` | `*const T` / `*mut T` |
|---|---|---|
| can be null | never | yes |
| can dangle (point at freed memory) | never: the borrow checker prevents it | yes |
| aliasing rules (one `&mut` or many `&`) | enforced | not enforced |
| making one | safe | **safe** |
| reading or writing through it | safe | **`unsafe`** |

Creating a raw pointer is harmless: it's just a number. The danger is in **using** it, so only that needs `unsafe`:

```rust
let pointer: *const u32 = &raw const value;          // safe
let read = unsafe { read_raw(pointer) };              // unsafe: you vouch for it
```

## When you actually need raw pointers

Rarely, and usually behind a safe interface:

| Situation | Example in this repository |
|---|---|
| calling C libraries, which only know raw pointers | [calling the C library](../../no-std-and-bare-metal/01-calling-the-c-library/) |
| talking to hardware at fixed addresses | [memory-mapped registers](../../no-std-and-bare-metal/04-memory-mapped-registers/) |
| data structures the borrow checker can't express, like a doubly linked list or a pointer type of your own | the homemade `Rc` below |
| squeezing out the last bit of speed, measured, after the safe version | (rarely worth it) |

## The original example was unsound

This repository's first raw-pointer example was:

```rust
fn get_value_from_raw_pointer(raw_p: *const u32) -> u32 {
    unsafe { *raw_p }
}
```

It's called "safe": there's no `unsafe` in its signature. So any code may call it with any pointer, for example `get_value_from_raw_pointer(std::ptr::null())`. That's **undefined behaviour from safe code**, which is the one thing Rust's safety promise says can't happen. A function like this is called **unsound**. Made `pub`, Clippy refuses it:

```text
error: this public function might dereference a raw pointer but is not marked `unsafe`
```

(It only passed Clippy because it was private.) Three ways to fix it:

**1. Use a reference.** This is almost always the right answer:

```rust
fn read_value(value: &u32) -> u32 { *value }
```

**2. Make it an `unsafe fn` and write down the contract:**

```rust
/// # Safety
/// `pointer` must be non-null, properly aligned, and point to an initialised
/// `u32` that stays valid for the duration of the call.
unsafe fn read_raw(pointer: *const u32) -> u32 {
    // SAFETY: the caller guarantees `pointer` is valid (see above).
    unsafe { *pointer }
}
```

The `unsafe` on the function moves the responsibility to the **caller**, and the `# Safety` section tells them exactly what they're promising.

**3. A tempting half-fix: check for null.** It isn't enough:

```rust
unsafe fn read_if_not_null(pointer: *const u32) -> Option<u32> {
    unsafe { pointer.as_ref() }.copied()
}
```

Null is only **one** way a pointer can be bad. A non-null pointer can still dangle, or be misaligned, or point at the wrong type. So even with the check, the function stays `unsafe`, and `as_ref` is `unsafe` for the same reason.

## Writing `unsafe` responsibly

- **Keep it small.** An `unsafe` block should contain only the operation that needs it.
- **Explain every block.** A `// SAFETY:` comment says why this particular use is fine: which guarantee makes it valid. If you can't write the comment, the code probably isn't sound. Clippy's `undocumented_unsafe_blocks` lint can require these comments.
- **Wrap it.** Put `unsafe` inside a function with a safe signature, which checks or guarantees everything the `unsafe` code needs. Then the rest of the program can't misuse it. `Vec`, `String` and `Rc` are all built this way.
- **Test it with Miri.** Miri is an interpreter that runs your tests and detects undefined behaviour, such as dangling pointers, out-of-bounds reads and aliasing violations, which normal tests can't see:

  ```bash
  rustup +nightly component add miri
  cargo +nightly miri test
  ```

  (It needs the nightly toolchain. It's worth setting up for any project with `unsafe` code.)

## Pointer arithmetic: what a slice does inside

```rust
let start = numbers.as_ptr();
for i in 0..numbers.len() {
    // SAFETY: `i < numbers.len()`, so `start.add(i)` stays inside the slice.
    total += unsafe { *start.add(i) };
}
```

`start.add(i)` moves the pointer forward by `i` elements, not `i` bytes. This is roughly what iterating over a slice does inside, except that the safe version guarantees you never step outside. The `// SAFETY:` comment states the condition that makes it correct: the loop bound.

## How `Rc` works inside: building one

[Lesson 3](../03-rc-and-weak/) used `Rc`, and [lesson 5](../05-your-own-smart-pointer/) built a pointer with `Deref` and `Drop`. Here they come together: a minimal `Rc`, built from a raw pointer.

```rust
struct Inner<T> {
    count: Cell<usize>,
    value: T,
}

pub struct MyRc<T> {
    pointer: NonNull<Inner<T>>,       // a raw pointer that's never null
    _owns: PhantomData<Inner<T>>,     // "this struct owns an Inner<T>"
}
```

| Step | What happens |
|---|---|
| `MyRc::new(value)` | `Box::new(Inner { count: 1, value })` puts it on the heap; `Box::leak` hands over the raw pointer. From now on, our code must free it |
| `clone()` | checked count + 1; aborts before overflow. The pointer is copied, the value isn't |
| `*my_rc` (`Deref`) | reads the value through the pointer |
| `drop()` | count − 1; at **0**, `Box::from_raw` turns the pointer back into a `Box`, and dropping that Box frees it |

```rust
impl<T> Drop for MyRc<T> {
    fn drop(&mut self) {
        let count = &self.inner().count;
        count.set(count.get() - 1);
        if count.get() == 0 {
            // SAFETY: the count is 0, so this was the last owner: nothing else can
            // use the Inner any more. It was created by `Box::new`, so turning it
            // back into a Box and dropping that frees it correctly, exactly once.
            unsafe { drop(Box::from_raw(self.pointer.as_ptr())) };
        }
    }
}
```

Every `unsafe` block rests on one **invariant**, a rule the type always keeps true: *while any `MyRc` exists, the count is at least 1, so the `Inner` hasn't been freed.* `new`, `clone` and `drop` all maintain it, and the struct's fields are private, so no other code can break it. That's what makes the outside of `MyRc` safe, even though its inside isn't.

A test proves the value is freed **exactly once**, and only after the last owner is gone. And `MyRc`, like the real `Rc`, can't be sent to another thread: `NonNull` is neither `Send` nor `Sync`, so the compiler refuses, which is right for a counter that isn't atomic ([lesson 4](../04-arc/)).

The real `Rc` adds a lot on top: `Weak` support, unsized types such as `Rc<str>`, and many optimisations.

## A reference count must never wrap around

Safe code can call `clone` and then `mem::forget` again and again, so the count can grow without limit. If it ever wrapped around from its maximum back to 0, one handle could free the value while other handles still point at it: a safe-looking API with a use-after-free inside. So `MyRc::clone` uses checked addition and **aborts** the program if the count is full, just like the standard `Rc`. A test runs this in a separate process, so that the abort doesn't end the test run itself. ([The Rustonomicon explains why.](https://doc.rust-lang.org/nomicon/arc-mutex/arc-clone.html))

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 5: Your own smart pointer](../05-your-own-smart-pointer/) · Back to the [course overview](../)
