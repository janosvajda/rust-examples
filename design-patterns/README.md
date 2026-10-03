<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Design Patterns

Design patterns are proven, reusable solutions to problems that come up again and again when designing software. Each example here explains its pattern in plain language with an everyday analogy, shows a small runnable program with tests, and points out what's different when you write it in Rust.

The patterns are grouped the classic way, into three kinds. Click a comic to see it full size.

## Creational: how objects get created

<p align="center">
  <a href="creational-patterns-comic.png">
    <img src="creational-patterns-comic.png" alt="Comic: the four creational patterns" width="100%">
  </a>
</p>

| Pattern | Example | In one line |
|---|---|---|
| [Factory](creational/factory-pattern/) | a robot factory: order by job, it picks the model | one place decides which concrete type to build |
| [Abstract Factory](creational/abstract-factory-pattern/) | tropical and temperate farms making matching fruit and vegetables | create families of related objects through one interface |
| [Builder](creational/builder-pattern/) | assembling spacecraft step by step | build a complex object in steps |
| [Singleton](creational/singleton-pattern/) | one shared app config, a global ID counter | exactly one shared instance, safely, with `OnceLock` and atomics |

## Structural: how objects fit together

<p align="center">
  <a href="structural-patterns-comic.png">
    <img src="structural-patterns-comic.png" alt="Comic: the five structural patterns" width="100%">
  </a>
</p>

| Pattern | Example | In one line |
|---|---|---|
| [Adapter](structural/adapter-pattern/) | an old Fahrenheit sensor in a Celsius app | make one interface fit another |
| [Decorator](structural/decorator-pattern/) | coffee with milk, sugar and cream | add behaviour by wrapping, layer by layer |
| [Facade](structural/facade-pattern/) | one "leave home" button for the whole house | one simple entry point to a complicated system |
| [Proxy](structural/proxy-pattern/) | a cache in front of a slow exchange-rate service | a stand-in that controls access to the real object |
| [Composite](structural/composite-pattern/) | files and folders | treat single items and groups the same way |

## Behavioral: how objects work together

<p align="center">
  <a href="behavioral-patterns-comic.png">
    <img src="behavioral-patterns-comic.png" alt="Comic: the eight behavioral patterns" width="100%">
  </a>
</p>

| Pattern | Example | In one line |
|---|---|---|
| [Command](behavioral/command-pattern/) | Mars Rover instructions | turn a request into an object |
| [Strategy](behavioral/strategy-pattern/) | travel time by car, bike or on foot | swap the algorithm at runtime |
| [Observer](behavioral/observer-pattern/) | a weather station and its displays | notify every subscriber when something changes |
| [State](behavioral/state-pattern/) | an online order's lifecycle | behaviour depends on the current state |
| [Iterator](behavioral/iterator-pattern/) | a playlist and the Fibonacci sequence | walk through items without seeing how they're stored |
| [Template Method](behavioral/template-method-pattern/) | one report, three export formats | fixed steps, customisable details |
| [Chain of Responsibility](behavioral/chain-of-responsibility-pattern/) | expense approval up the management chain | pass a request along until someone handles it |
| [Memento](behavioral/memento-pattern/) | undo and redo in a text editor | save snapshots to restore later |

## Run a pattern

```bash
cd design-patterns/behavioral/state-pattern
cargo run
cargo test
```

Or from the repository root, with the folder name as the package name: `cargo run -p state-pattern`.
