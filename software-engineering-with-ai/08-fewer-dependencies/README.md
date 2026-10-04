<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 8: Fewer dependencies

> Written in October 2026. AI tools change every month. Check whether what you read here still holds.

## The idea in one sentence

Every external package is code you run but didn't write and probably won't read, so add one **on purpose**, never by default. AI makes that more important, not less.

## Our opinion: fewer, but not zero

**Reducing dependencies is good practice.** But "zero dependencies" isn't the goal:
- **Don't write your own** cryptography, TLS, date and time zone handling, Unicode processing, an async runtime or a serialization framework. Well-known crates like `rustls`, `chrono`, `tokio` and `serde` have years of expert work and real-world testing behind them. Your version won't.
- **Do write your own** small things: a 20-line helper, one simple parser, one error type. A package for those costs more than it saves.

The question for every dependency is: **is what it saves me worth what it costs me?**

## What a dependency really costs

Adding one line to `Cargo.toml` is easy. Here's what comes with it:

- **Code you trust without reading.** A dependency runs with all the permissions of your program. If it's malicious or buggy, so is your program.
- **Its dependencies too.** One line can pull in dozens of crates. In this repository, the [Lambda example](../../aws-lambda-example-db/) lists **15** dependencies in its `Cargo.toml`, and builds **226** crates. That's normal for the AWS SDK, which does a lot, but each of those 226 is code someone trusted.
- **Supply-chain attacks.** Attackers target packages, because one compromised package reaches thousands of programs. In 2024, a backdoor was found in `xz`, a compression library used by many Linux systems. It had been planted over years by a contributor who had gained the maintainers' trust. Package registries, crates.io included, regularly remove packages whose names imitate popular ones.
- **Abandonment.** Maintainers move on. `atty`, once used by a huge number of Rust programs to check "is this a terminal?", became unmaintained. Today the standard library does the same job.
- **Build time, binary size, licences and breaking updates**, multiplied by every crate in the tree.

## What AI changes

AI changes the balance in two directions at once:

**1. AI adds dependencies casually.** "Just add crate X" is an easy answer for an AI, and it rarely weighs the cost. Worse, AI sometimes **invents package names** that sound right but don't exist. Attackers have noticed: they register those invented names and fill them with malicious code. This even has a name, **slopsquatting**. Always check that a suggested crate exists, is the one you think, and is well maintained.

**2. AI makes writing small things yourself cheap.** Writing a small helper with tests used to cost an hour, so pulling in a crate was tempting. Now an AI writes it in a minute, and you review it in five. **The cost of "write it yourself" dropped; the cost of a dependency didn't.** For small things, that tips the balance towards your own code.

## A real example: Mini went from 5 dependencies to 2

The [Mini compiler](../../mini/) in this repository used to depend on five crates: `inkwell`, `anyhow`, `thiserror`, `regex` and `which`.

- **`inkwell`**, the Rust bindings for LLVM, was the expensive one. It didn't just add code: it tied Mini to **one exact LLVM version**, which had to be installed to build Mini at all. Replacing it with plain text output (LLVM IR, which the installed LLVM compiles) removed that requirement entirely. Mini now builds on any machine and works with any LLVM from version 16 up.
- **`thiserror`** wasn't used at all anymore. Unused dependencies happen easily and cost build time for nothing.
- **`which`** was used once, to check whether `gcc` exists. Trying to run `gcc --version` does the same job with the standard library.

Now `cargo tree` shows 2 direct dependencies and 6 crates in total. And honestly, `regex` could go too: it's used for two simple line patterns.

## The standard library already does it

Some crates were essential years ago, and the standard library has since caught up. AI suggestions, trained on older code, often still recommend the old crate:

| Old habit | Standard library today |
|---|---|
| `lazy_static` or `once_cell` for a global computed once | `std::sync::LazyLock` and `OnceLock` |
| `atty` to check for a terminal | `std::io::IsTerminal` |
| `num_cpus` to count processor cores | `std::thread::available_parallelism` |
| `cfg-if` to choose code per platform | `cfg_select!` (see the [asm example](../../asm/hello_asm/)) |
| `fs2` or `fs4` for file locks | `File::lock`, `File::try_lock` (see [concurrency lesson 3](../../concurrency/03-locks-and-condvars/)) |
| `thiserror` for one or two error types | implement `Display` and `Error` yourself (see the [error handling course](../../error-handling/03-custom-error-types/)) |

`thiserror` is still a fine crate, and [lesson 4 of the error handling course](../../error-handling/04-thiserror-and-anyhow/) shows when it pays off. The point is to **choose**, not to add it out of habit.

## Before adding a dependency

| Question | Why it matters |
|---|---|
| Is it hard to get right (crypto, TLS, time zones, Unicode, async)? | then **use** a well-known crate; don't write your own |
| Is it 20–50 lines I can test? | then consider writing it yourself |
| Does it really exist, and is the name exactly right? | invented and look-alike names are an attack |
| Is it maintained? Who maintains it, and how many people use it? | abandoned code doesn't get security fixes |
| How many crates does it pull in? (`cargo tree`) | you trust all of them |
| Can I turn off what I don't need? (`default-features = false`) | many crates have optional parts |
| Is its licence compatible with my project? | some licences have obligations |

## Tools that help

| Tool | What it does |
|---|---|
| `cargo tree` | shows every crate you depend on, and why; built into Cargo |
| `cargo tree --duplicates` | shows crates included in several versions |
| `cargo audit` | checks your `Cargo.lock` against the RustSec database of known vulnerabilities (`cargo install cargo-audit`) |
| `cargo deny` | enforces rules: allowed licences, banned crates, trusted sources (`cargo install cargo-deny`) |
| `Cargo.lock` | records exact versions; commit it for applications, so every build uses the same, checked code |

And tell your AI tools. Put a rule like this in the project instruction file: **"Don't add a new dependency without asking. Prefer the standard library."**

## What may change

Package registries are adding better defences: signing, verified publishers, and automatic scanning for malicious code. AI tools may learn to check packages before suggesting them. But a dependency will always be code you didn't write, and the decision to trust it will always be yours.

Previous: [Lesson 7: From day one, or later?](../07-from-day-one-or-later/) · Next: [Lesson 9: Reviewing AI-written code](../09-reviewing-ai-code/)
