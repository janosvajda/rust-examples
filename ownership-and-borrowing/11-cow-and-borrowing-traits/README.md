<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 11: Cow and the borrowing traits

## The idea in one sentence

The standard library has a few tools that let the same code work with **borrowed and owned data interchangeably**, so you only copy when you really have to.

## `Cow`: borrow when you can, own when you must

`Cow` stands for **clone on write**. It's an enum that holds either a borrowed value or an owned one:

```rust
enum Cow<'a, B> {
    Borrowed(&'a B),    // e.g. a &str
    Owned(B::Owned),    // e.g. a String
}
```

Use it when a function **usually** returns its input unchanged, but **sometimes** has to change it:

```rust
fn normalise_tabs(text: &str) -> Cow<'_, str> {
    if text.contains('\t') {
        Cow::Owned(text.replace('\t', "    "))   // changed: a new String
    } else {
        Cow::Borrowed(text)                      // unchanged: no copy at all
    }
}
```

Without `Cow`, the function would have to return a `String` every time, allocating a copy even when nothing changed. With `Cow`, text without tabs is returned as-is: a test checks it's the very same memory as the input.

You use a `Cow<str>` just like a `&str` (it derefs to one). `.into_owned()` gives you a `String` either way, copying only if it was borrowed.

## `Borrow`: why `HashMap<String, _>` accepts `&str`

```rust
let mut stock: HashMap<String, u32> = HashMap::new();
stock.insert(String::from("apples"), 12);
stock.get("apples");          // ✓ search with a &str, no String needed
```

The keys are `String`s, yet you search with a plain `&str`. That works because `String` implements **`Borrow<str>`**, which is a promise that a `String` and the `&str` it contains compare and hash **exactly the same way**. The map can therefore compare your `&str` against its stored keys directly. Without that, every lookup would have to create a `String` first.

## `AsRef`: one function, many kinds of input

```rust
fn file_extension<P: AsRef<Path>>(path: P) -> Option<String>

file_extension("photo.jpg");          // &str
file_extension(&owned_path);          // &String
file_extension(Path::new("notes"));   // &Path
```

`AsRef<T>` means *"can be cheaply viewed as a `&T`"*. Taking `P: AsRef<Path>` lets callers pass whatever kind of path or string they already have. The standard library does this everywhere: `File::open`, `fs::read_to_string` and friends all take `AsRef<Path>`.

**`AsRef` or `Borrow`?** They look alike. `AsRef` just means "can give a reference". `Borrow` adds the promise that equality and hashing behave identically, which `HashMap` needs. For your own function parameters, use `AsRef`.

## `Deref`: a wrapper that behaves like what it wraps

`String` derefs to `str`, `Vec<T>` to `[T]`, and `Box<T>` to `T`. That's why you can call slice methods on a `Vec`, or `str` methods on a `String`. You can do the same for your own wrapper types:

```rust
struct SortedNames(Vec<String>);

impl Deref for SortedNames {
    type Target = [String];
    fn deref(&self) -> &[String] { &self.0 }
}

names.len();  names.first();  names.contains(…);  names[0];   // all from [String]
```

`SortedNames` deliberately **doesn't** implement `DerefMut`: that would let callers push or reorder items and break its "always sorted" promise. Read-only access through `Deref` is safe; mutable access is up to you.

## Deref coercion

When you pass a reference to a function, Rust follows `Deref` as many times as needed to match the parameter type:

```rust
fn count_chars(text: &str) -> usize

let boxed: Box<String> = Box::new(String::from("…"));
count_chars(&boxed);     // &Box<String> → &String → &str, automatically
```

This is why functions should take `&str` and `&[T]` (lesson 4): through deref coercion, they accept `String`, `Box<String>`, `Cow<str>`, `Vec`, arrays and more.

## Summary

| Tool | Use it when |
|---|---|
| `Cow<str>`, `Cow<[T]>` | a function sometimes returns its input unchanged and sometimes a modified copy |
| `Borrow` | you build a collection that should be searchable by a borrowed form of its keys |
| `AsRef<T>` | a function should accept many types that can all be viewed as `&T` |
| `Deref` | your type wraps another and should be usable like it (smart pointers, wrappers) |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 10: Advanced lifetimes](../10-advanced-lifetimes/) · Next: [Lesson 12: Where the borrow checker is too strict](../12-borrow-checker-limits/)
