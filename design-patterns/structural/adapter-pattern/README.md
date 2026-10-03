<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Adapter Pattern

## What is it?

The **Adapter** pattern lets two pieces of code work together even though their interfaces don't match, by putting a small translator between them.

Think of a travel plug adapter. Your laptop charger has a UK plug, the wall socket in Hungary expects a European one. You don't rewire the charger or the wall. You put an adapter in between that has the right shape on each side.

In code, it's the same idea: your program expects one interface, some existing code offers a different one, and you can't (or don't want to) change either. The adapter wraps the existing code and presents it in the shape your program expects.

## The parts

```text
  your code ──uses──► «trait» TemperatureSensor ◄──implements── LegacySensorAdapter
                       name(), celsius()                              │
                                                                      │ wraps and translates
                                                                      ▼
                                                     legacy_thermo::ThermoDevice
                                                     serial_number(), read_tenths_fahrenheit()
```

- **Target** (`TemperatureSensor` trait): the interface your program uses everywhere.
- **Adaptee** (`legacy_thermo::ThermoDevice`): the existing code with the "wrong" interface. Here it's a module standing in for a third-party library you can't edit.
- **Adapter** (`LegacySensorAdapter`): holds an adaptee and implements the target interface by calling the adaptee and converting the results.

## The example

A smart-home app reads temperatures from sensors that implement `TemperatureSensor`, reporting a name and a reading in °C.

You also own some old sensors whose library is different in every way:

| | What the app expects | What the old library offers |
|---|---|---|
| Name | `name()` returns the room | only `serial_number()` |
| Reading | `celsius()` returns °C as a decimal | `read_tenths_fahrenheit()` returns tenths of °F as a whole number (`725` = 72.5 °F) |

`LegacySensorAdapter` translates both: it builds a name from the room and serial number, and converts tenths of °F into °C.

In `main`, modern and adapted legacy sensors sit in the **same list**, and `average_celsius` works with all of them without knowing the old library exists.

## The Rust way

In Rust an adapter is usually just **a struct that wraps the other type, plus a trait implementation**. That's all `LegacySensorAdapter` is.

Rust has one rule that makes adapters especially common. You can't implement a trait from another crate for a type from another crate (the *orphan rule*). So if a library's type doesn't implement a trait you need, you wrap it in your own struct and implement the trait for that. This wrapper is often called a *newtype*, and it's the Adapter pattern in its simplest form.

## When to use it

- You want to use existing code (a library, an old module, an external API) whose interface doesn't match yours.
- You want to keep the rest of your program independent of that code, so you could swap it out later by changing only the adapter.

**When not to:** if you control both sides and can simply make them match, change the code instead of adding a translation layer.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p adapter-pattern`.
