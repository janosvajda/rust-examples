<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Builder pattern: assemble a spacecraft

Imagine ordering a spacecraft: choose its type, equipment and assembler, then check the finished order. Named builder methods make those choices visible at the call site.

```rust
let ship = SpacecraftBuilder::new("Research Shuttle")
    .equipment("Deep space telescope")
    .assembler(Assembler::Nanorobots)
    .build()?;
```

`new` starts an order with a spacecraft type, no equipment, and a human assembler. Each configuration method takes ownership of the builder and returns it, so methods chain naturally. `build(self)` validates required fields and **moves** the strings into the spacecraft. It returns `BuildError` for a blank type or missing/blank equipment.

The same builder offers three presets: `shuttle()`, `battleship()` and `dreadnought()`. You can override a preset before building:

```rust
let ship = SpacecraftBuilder::shuttle()
    .equipment("Radar and a telescope")
    .build()?;
```

## Where the pattern comes from

Builder was among the patterns described in the 1994 [*Design Patterns* book](https://www.informit.com/store/design-patterns-elements-of-reusable-object-oriented-software-9780201633610). A classic design can use a **director** to run a reusable sequence of construction steps against different builders. A director is optional. This Rust example uses a consuming builder with presets; its three spacecraft configurations share one implementation.

`Assembler` is an enum, so an unsupported assembler cannot slip through as a misspelled string. The strings describe the fictional spacecraft; the program does not manufacture hardware.

```bash
cargo run
cargo test
```
