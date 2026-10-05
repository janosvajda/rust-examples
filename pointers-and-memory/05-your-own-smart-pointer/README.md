<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 5: Your own smart pointer

## The idea in one sentence

A smart pointer is just a struct that implements two traits: **`Deref`**, so it can be used like the value it points to, and **`Drop`**, so it can clean up when it goes away. With `Drop`, any resource can clean up after itself, when the owning value is dropped.

## `Deref`: behave like the value inside

This lesson's `Tracked<T>` owns a value on the heap, like a `Box`, and counts how often it's read and written:

```rust
impl<T> Deref for Tracked<T> {
    type Target = T;
    fn deref(&self) -> &T {
        self.reads.set(self.reads.get().saturating_add(1));
        &self.value
    }
}
```

With `Deref` (and `DerefMut` for changes), a `Tracked<String>` can be used as if it **were** a `String`:

```rust
let mut name = Tracked::new(String::from("ferris"));
name.len();                     // a String method: Rust calls deref() for you
name.push_str(" the crab");     // through DerefMut
shout(&name);                   // shout takes &str
```

```text
length: 6
shouted: FERRIS THE CRAB
used: 2 reads, 1 write
```

### Deref coercion

`shout` takes a `&str`, and we passed a `&Tracked<String>`. Rust follows the `Deref` chain by itself: `&Tracked<String>` → `&String` → `&str`. This is **deref coercion**. It's why a `&Box<String>`, a `&Rc<String>` or a `&String` can all be passed where a `&str` is expected.

Implement `Deref` only for types that really **are** a kind of pointer to their target. Using it to fake inheritance ("my struct derefs to another struct, so it gets its methods") makes code confusing, and the Rust API guidelines advise against it.

## `Drop`: clean-up that can't be forgotten

```rust
impl Drop for Noisy<'_> {
    fn drop(&mut self) {
        self.log.borrow_mut().push(self.name);
    }
}
```

`drop` runs automatically when a value goes away. The order is predictable:

```text
["early", "— end of scope —", "second", "first"]
```

- At the end of a scope, values are dropped in the **reverse** order of declaration within that scope: `second` before `first`. Later values may depend on earlier ones, so they go first.
- `drop(value)` ends a value early, as `early` shows. Calling the method directly isn't allowed:

```text
error[E0040]: explicit use of destructor method
```

  That rule makes running `drop` twice on one value impossible.

This is how `Box` frees its heap memory, and how `Rc` decreases its count. The standard library's smart pointers are built on exactly these two traits.

## RAII: resources that clean themselves up

The most useful thing `Drop` does is tie a **resource** to a **value**: the resource is released when the value goes away. The name comes from C++: *Resource Acquisition Is Initialisation*. In short: if you own it, you clean it up, automatically.

```rust
pub struct TempFile {
    path: PathBuf,
}

impl TempFile {
    pub fn create(contents: &str) -> std::io::Result<Self> {
        let path = /* a new, unique name in the system's temporary folder */;
        let mut file = OpenOptions::new().write(true).create_new(true).open(&path)?;
        file.write_all(contents.as_bytes())?;
        Ok(TempFile { path })
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);   // the clean-up, in one place
    }
}
```

Creating a `TempFile` creates the file; dropping it deletes the file. `create_new(true)` refuses to open a file that already exists, so a `TempFile` never takes over, and later deletes, someone else's file. The demo gives each file a unique name made of the process's id and a counter.

The file is deleted whenever the `TempFile` value goes away:

| How the function ends | File deleted? | Test |
|---|---|---|
| normally | yes | `temporary_files_are_distinct_and_removed_on_drop` |
| an early `return` (or `?`) | yes | same |
| an **unwinding panic** | yes: destructors run during unwinding | `the_temp_file_is_gone_even_after_a_panic` |

The destructor runs on every normal way out, and while a panic unwinds. It doesn't run if the process is killed or aborts, or if the value is deliberately leaked with `mem::forget`. Rust uses this everywhere:

| Type | What its `Drop` releases |
|---|---|
| `Box`, `Vec`, `String` | heap memory |
| `File`, `TcpStream` | the operating system's file or connection |
| `MutexGuard` | the lock: that's why there's no `unlock()` |
| `JoinHandle` | (nothing: dropping it detaches the thread, which keeps running) |

One limit: `drop` can't return an error, so a failure while cleaning up can only be ignored or logged. When the outcome of the clean-up matters, like making sure data really reached the disk, add an explicit method that returns a `Result` (such as `File::sync_all`) and call it before the value goes away.

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 4: Arc](../04-arc/) · Next: [Lesson 6: Raw pointers and unsafe](../06-raw-pointers-and-unsafe/)
