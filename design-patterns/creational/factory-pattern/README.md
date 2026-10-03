<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Factory Pattern

## What is it?

The **Factory** pattern puts the decision *"which concrete type should I create?"* in one place, so the rest of the program only asks for what it needs and never names the concrete types.

Think of ordering a taxi through an app. You say "I need a ride for 6 people", and the company decides whether to send a minivan or two cars. You don't pick the vehicle model, and if the company adds electric cars next year, nothing changes for you.

## The parts

```text
  order: "guard"                 ┌─────────────────────┐
  ─────────────► Job::Guard ───► │ RobotFactory::build │ ──► Box<dyn Robot>
                                 └──────────┬──────────┘     (the caller only sees "a Robot")
                                            │ decides
                         ┌──────────────────┼──────────────────┐
                         ▼                  ▼                  ▼
                     HelperBot          GuardBot            ScoutBot
```

- **Product** (`Robot` trait): what every robot can do. The caller uses only this.
- **Concrete products** (`HelperBot`, `GuardBot`, `ScoutBot`): the actual models. The caller never names them.
- **Factory** (`RobotFactory::build`): the one function that maps an order to a model, and sets it up with the factory's own settings (shift length, scouting range).

## The example

You order a robot by the **job** it should do:

| Order | The factory builds | Set up with |
|---|---|---|
| `"help"` | `HelperBot` | nothing extra |
| `"guard"` | `GuardBot` | the factory's shift length |
| `"explore"` | `ScoutBot` | the factory's scouting range |
| `"dance"` | nothing: the order is refused | |

The orders arrive as text, like they would from a user or a config file, and are parsed into a `Job` first. An unknown order becomes an error, not a crash.

Adding a fourth model, say a `CleanerBot`, means adding one enum variant and one `match` arm in `build`. No calling code changes.

## The Rust way

- **The order is an enum** (`Job`). The list of valid orders is explicit, and because `match` must handle every variant, the compiler won't let you add a job and forget to build a robot for it.
- **The result is a trait object** (`Box<dyn Robot>`): "some type that implements `Robot`, decided at runtime". This is what lets one function return different types.
- **Parsing with `FromStr`** lets you write `"guard".parse::<Job>()`, the standard Rust way to turn text into a value that can fail.

If the set of products is fixed and you don't need trait objects, a factory can also return an enum (`enum AnyRobot { Helper(HelperBot), … }`). The idea is the same.

## When to use it

- The type to create depends on runtime information: user input, configuration, file contents.
- Creating an object needs setup the caller shouldn't have to know about.
- You want to add new types later without touching the code that uses them.

**When not to:** if there's only one type and it's created in one place, call its constructor directly.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p factory-pattern`.
