<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 11: Cow and the borrowing traits

## The idea in one sentence

The standard library provides tools for accepting **borrowed and owned data** and choosing when to make an owned copy.

Ferris calls `Cow` the lazy photocopier: use the original text when possible, and request a copy when owned or mutable access is needed.

## `Cow`: borrow when you can, own when you must

`Cow` stands for **clone on write**. Its shape is shown below; in your program, import `std::borrow::Cow` rather than defining this enum:

```rust,ignore
enum Cow<'a, B: ToOwned + ?Sized> {
    Borrowed(&'a B),    // e.g. a &str
    Owned(B::Owned),    // e.g. a String
}
```

`ToOwned` links a borrowed form to its owned form: `str` to `String`, for example. `?Sized` allows unsized borrowed forms such as `str`.

Use `Cow` when a function usually returns its input unchanged but sometimes creates a changed result:

```rust
use std::borrow::Cow;

fn normalise_tabs(text: &str) -> Cow<'_, str> {
    if text.contains('\t') {
        Cow::Owned(text.replace('\t', "    "))   // changed: a new String
    } else {
        Cow::Borrowed(text)                      // unchanged: no copy at all
    }
}
```

Returning a `String` for every input requires owned text even when no tab changes. `Cow` expresses both outcomes in one return type. In this helper, text without tabs is returned as-is: a test checks that it points to the original memory.

Each tab becomes exactly four spaces here; this helper does not calculate column-dependent tab stops.

`Cow<str>` dereferences to `str`, so methods such as `len()` work directly. `.into_owned()` returns its existing owned `String`, or creates one from the borrowed text. **`to_mut()`** clones borrowed contents before giving mutable access; it cannot predict whether you will actually change them:

```rust
use std::borrow::Cow;
let mut label: Cow<'_, str> = Cow::Borrowed("Ferris");
label.to_mut().push('!'); // makes an owned String before allowing mutation
assert_eq!(label, "Ferris!");
```

Once owned, later `to_mut()` calls can use the existing owned value.

## `Borrow`: why `HashMap<String, _>` accepts `&str`

```rust
use std::collections::HashMap;

let mut stock: HashMap<String, u32> = HashMap::new();
stock.insert(String::from("apples"), 12);
stock.get("apples");          // ✓ search with a &str, no String needed
```

The keys are `String`s, yet you search with a plain `&str`. That works because `String` implements **`Borrow<str>`**, which is a promise that a `String` and the `&str` it contains compare and hash **exactly the same way**. The map can therefore compare your `&str` against its stored keys directly. This borrowed-key API avoids creating a temporary owned key for each lookup.

## `AsRef`: one function, many kinds of input

```rust
use std::path::Path;
fn file_extension<P: AsRef<Path>>(path: P) -> Option<String> {
    path.as_ref().extension().map(|ext| ext.to_string_lossy().into_owned())
}
let owned_path = String::from("report.pdf");
file_extension("photo.jpg");          // &str
file_extension(&owned_path);          // &String
file_extension(Path::new("notes"));   // &Path
```

`AsRef<T>` means *"can be cheaply viewed as a `&T`"*. Taking `P: AsRef<Path>` lets callers pass whatever kind of path or string they already have. The standard library does this everywhere: `File::open`, `fs::read_to_string` and friends all take `AsRef<Path>`.

**`AsRef` or `Borrow`?** `AsRef` offers a cheap reference conversion. `Borrow` additionally requires matching equality, ordering and hashing behaviour between the owned and borrowed forms. Use `AsRef` when you only need the view, and `Borrow` when those key semantics matter.

A generic argument taken by value still follows move rules: `file_extension(owned_path)` consumes that `String`; `file_extension(&owned_path)` borrows it. The helper uses `to_string_lossy()`, so invalid Unicode in a platform's path extension is replaced rather than preserved exactly.

## `Deref`: a wrapper that behaves like what it wraps

`String` derefs to `str`, `Vec<T>` to `[T]`, and `Box<T>` to `T`. That's why you can call slice methods on a `Vec`, or `str` methods on a `String`. You can do the same for your own wrapper types:

```rust
use std::ops::Deref;
struct SortedNames(Vec<String>);
impl SortedNames {
    fn new(mut names: Vec<String>) -> Self {
        names.sort();
        Self(names)
    }
}
impl Deref for SortedNames {
    type Target = [String];
    fn deref(&self) -> &[String] { &self.0 }
}
let names = SortedNames::new(vec![String::from("Zoe"), String::from("Ana")]);
assert_eq!(names.len(), 2);
assert_eq!(names.first().unwrap(), "Ana");
assert!(names.contains(&String::from("Zoe")));
assert_eq!(&names[0], "Ana");
```

`SortedNames` does not expose `DerefMut<Target = [String]>`: mutable slice access would permit reordering or replacing names and break its sorted invariant. A slice has no `Vec::push` method, so this target would not expose pushing even with `DerefMut`.

`Deref` exposes the target's methods as part of the wrapper's interface. Use that deliberately, particularly for smart-pointer-like types; it is not necessary for every wrapper.

## Deref coercion

When you pass a reference to a function, Rust follows `Deref` as many times as needed to match the parameter type:

```rust
fn count_chars(text: &str) -> usize { text.chars().count() }

let boxed: Box<String> = Box::new(String::from("…"));
count_chars(&boxed);     // &Box<String> → &String → &str, automatically
```

A `&str` parameter accepts suitable **references to** `String`, `Box<String>` and `Cow<str>` through deref coercion. A `&[T]` parameter accepts borrowed vectors through deref coercion and borrowed arrays through array-to-slice coercion. You still pass a borrow rather than handing over the owned value.

`count_chars` counts Unicode scalar values, which can differ from visible letters or grapheme clusters (lesson 4).

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

Library details: [Cow](https://doc.rust-lang.org/std/borrow/enum.Cow.html), [Borrow](https://doc.rust-lang.org/std/borrow/trait.Borrow.html) and [Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html).

Previous: [Lesson 10: Advanced lifetimes](../10-advanced-lifetimes/) · Next: [Lesson 12: Where the borrow checker is too strict](../12-borrow-checker-limits/)
