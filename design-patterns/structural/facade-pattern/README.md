<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Facade Pattern

## What is it?

The **Facade** pattern puts one simple interface in front of a complicated system, so callers don't need to know how the system works inside.

Think of a hotel reception desk. You say "I'd like a taxi at 8, and a wake-up call at 7". You don't phone the taxi company, program the room's alarm clock and update the booking system yourself. The receptionist knows who to talk to and in what order. The receptionist is the facade.

A facade doesn't hide the system completely. The parts are still there if someone needs them directly. It just gives the common tasks a shortcut.

## The parts

```text
              leave_home()   arrive_home(pin)   movie_night()
                       │             │                │
                ┌──────▼─────────────▼────────────────▼──────┐
  your code ──► │                SmartHome                    │  (the facade)
                └──────┬────────────┬───────────┬────────┬───┘
                       ▼            ▼           ▼        ▼
                    Lights     Thermostat    DoorLock   Alarm      (the subsystems)
```

- **Facade** (`SmartHome`): a few high-level actions that coordinate everything.
- **Subsystems** (`Lights`, `Thermostat`, `DoorLock`, `Alarm`): the real work, each with its own detailed controls. They don't know the facade exists.

## The example

A smart home has four devices. The facade offers three everyday actions:

| Action | What the facade does, in order |
|---|---|
| `leave_home()` | all lights off → heating down to 16 °C → lock the door → arm the alarm |
| `arrive_home(pin)` | unlock with the PIN → disarm the alarm → heating up to 21 °C → hallway light on |
| `movie_night()` | all lights off → living room light on → heating to 22 °C |

The facade also knows the **rules** about order. If the PIN is wrong, `arrive_home` stops before disarming the alarm, so the house stays secure. A test checks this. Without the facade, every piece of code that lets someone in would have to remember that rule itself.

## The Rust way

A facade in Rust is just a struct that owns (or borrows) the subsystems and has a few methods. Nothing special is needed.

**Modules** are where Rust takes the idea further: a module or crate can keep its many internal types private (`pub(crate)` or no `pub` at all) and export only a small, simple public API. Many popular crates work this way. For example, `reqwest::get(url)` is a one-line facade over connection pools, TLS, redirects and HTTP parsing.

## When to use it

- A system has many parts, and most callers only need a few common tasks.
- Several places in the code repeat the same sequence of steps across several components.
- You want to keep the rest of your program from depending on the details of a complicated library.

**Watch out for:** a facade that keeps growing until it does everything. That's a "god object". Keep it focused on the common tasks.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p facade-pattern`.
