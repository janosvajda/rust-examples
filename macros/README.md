<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Macros

Macros are code that writes code. You've used them from day one (`println!`, `vec!`, `#[derive(Debug)]`), and this course shows how to write your own: from simple `macro_rules!` patterns to a real `#[derive]` procedural macro.

Every compiler error quoted in these READMEs is the real output of Rust 1.99.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [`macro_rules!` basics](01-macro-rules-basics/) | patterns and templates, several rules, fragment types, why Rust macros aren't text substitution, hygiene |
| 2 | [Repetition](02-repetition/) | `$( … ),*`, the `*` `+` `?` operators, trailing commas, recursive macros |
| 3 | [Macros that generate code](03-generating-code/) | one `impl` for many types, families of types, keeping lists in sync, generated tests, when *not* to use a macro |
| 4 | [A derive macro](04-derive-macro/) | procedural macros, `syn` and `quote`, reading struct fields, good error messages |

## Macros vs functions

| | Function | Macro |
|---|---|---|
| Number of arguments | fixed | any |
| Takes | values | code: expressions, names, types, blocks |
| Runs | when the program runs | when the program is **compiled** |
| Can define new items | ✗ | ✓ functions, structs, `impl`s, tests |
| Error messages and tooling | excellent | harder to read; harder for editors to understand |

**Prefer functions and generics.** Use a macro when you need what only a macro can do: variable arguments, code as input, or generating definitions.

## The errors you'll meet

| Error | In plain words | Lesson |
|---|---|---|
| no rules expected `…` | the call doesn't match any of the macro's patterns | 1 |
| unexpected end of macro invocation | the call stopped before a pattern was complete | 1 |
| cannot find macro `…` in this scope | a `macro_rules!` macro is used above its definition | 1 |
| **E0425** cannot find value `x` in this scope | hygiene: a macro's variables aren't visible outside it | 1 |

## Run a lesson

```bash
cd macros/02-repetition
cargo run
cargo test
```

Or from the repository root: `cargo run -p macro-repetition`. The package names are in each lesson's `Cargo.toml`.
