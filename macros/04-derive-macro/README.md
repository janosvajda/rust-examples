<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 4: A derive macro

## The idea in one sentence

A **procedural macro** is ordinary Rust code that runs **inside the compiler**: it receives your code as tokens and returns new code. `#[derive(Debug)]`, `#[derive(Serialize)]` and `#[tokio::main]` all work this way.

## `macro_rules!` vs procedural macros

| | `macro_rules!` (lessons 1–3) | Procedural macros (this lesson) |
|---|---|---|
| Written as | patterns and templates | a Rust function that transforms code |
| Can inspect the input | only by matching patterns | fully: walk the syntax tree, read field names and types |
| Lives in | any crate | **its own crate**, marked `proc-macro = true` |
| Kinds | one | **derive** `#[derive(X)]`, **attribute** `#[x]`, **function-like** `x!(…)` |
| Use when | patterns are enough | you need to look *inside* a struct or enum, e.g. its fields |

## What we build: `#[derive(Describe)]`

```rust
#[derive(Describe)]
struct User { name: String, age: u32, admin: bool }

user.describe()        // "User { name: \"Ferris\", age: 9, admin: true }"
User::field_names()    // ["name", "age", "admin"]
```

The macro reads the struct's **field names** at compile time and writes two methods. A `macro_rules!` macro can't do this: it can't look inside a struct definition it didn't create.

## Two crates

```text
04-derive-macro/
├─ describe-derive/      the macro itself     (proc-macro = true)
│  └─ src/lib.rs
└─ demo/                 a program using it   (depends on describe-derive)
   └─ src/main.rs
```

A procedural macro must be in its **own crate**, because the compiler compiles it first, loads it like a plugin, and then runs it while compiling the crates that use it.

## How it works, in four steps

All of it is in [`describe-derive/src/lib.rs`](describe-derive/src/lib.rs), about 60 lines with comments:

```text
  your struct  ──tokens──►  1. PARSE     syn turns tokens into a syntax tree (DeriveInput)
                            2. INSPECT   is it a struct with named fields? collect the names
                            3. GENERATE  quote! fills in a code template
  new methods  ◄─tokens──   4. RETURN    the compiler adds the generated code to your program
```

Two crates do the heavy lifting, and nearly every procedural macro uses them:
- **`syn`** parses tokens into a Rust syntax tree: structs, fields, types, generics;
- **`quote`** turns a Rust-like template back into tokens. `#name` inserts a value, and `#( … ),*` repeats once per item, just like `$( … ),*` in `macro_rules!`.

```rust
quote! {
    impl #name {
        pub fn field_names() -> &'static [&'static str] {
            &[ #( #field_names ),* ]
        }
    }
}
```

The macro also copies the struct's **generics** onto the generated `impl`, using `split_for_impl()`, so it works on `Labelled<T>` too. A test checks that.

## Good error messages

A macro should explain clearly when it's used wrongly. This one returns a compile error that points at the user's type, exactly as the compiler prints it:

```text
error: Describe can only be derived for structs, not enums or unions
error: Describe needs a struct with named fields, like `struct S { a: u32 }`
```

`syn::Error::new_spanned(…).to_compile_error()` is the standard way to produce these.

## The other two kinds, briefly

- **Attribute macros**, `#[my_attribute] fn …`, receive the item they're attached to and can rewrite it completely. Example: `#[tokio::main]` turns `async fn main` into a normal `main` that starts a runtime.
- **Function-like macros**, `my_macro!(…)`, look like `macro_rules!` calls but run Rust code. Example: `sqlx::query!` checks SQL against a real database at compile time.

They're written the same way as this one, with `#[proc_macro_attribute]` or `#[proc_macro]` instead of `#[proc_macro_derive]`.

## Run it

```bash
cd demo
cargo run
cargo test
```

From the repository root: `cargo run -p derive-macro-demo`.

Previous: [Lesson 3: Macros that generate code](../03-generating-code/) · Back to the [course overview](../)
