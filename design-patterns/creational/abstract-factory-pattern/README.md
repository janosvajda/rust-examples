<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Abstract Factory Pattern

## What is it?

The **Abstract Factory** pattern creates **families of related objects** that are meant to be used together, through one interface, without the caller naming any concrete type.

Think of buying furniture from a shop that sells complete styles: a *modern* set or a *rustic* set. You pick a style once, and every piece you get (chair, table, lamp) matches. You can't accidentally end up with a modern chair and a rustic table, because each set comes from the same workshop.

**Factory or Abstract Factory?** A Factory (see the [factory example](../factory-pattern/)) decides which *one* product to make. An Abstract Factory makes a *matching set* of several different products.

## The parts

```text
                        «trait» Farm
                    create_fruit(), create_vegetable()
                     ▲                          ▲
          ┌──────────┴─────────┐     ┌──────────┴──────────┐
          │    TropicalFarm    │     │    TemperateFarm    │
          │  fruit: Mango      │     │  fruit: Apple       │
          │  vegetable: Okra   │     │  vegetable: Carrot  │
          └────────────────────┘     └─────────────────────┘

  pack_basket(farm: &dyn Farm)  ──►  always gets a fruit AND a vegetable from the same farm
```

- **Abstract factory** (`Farm` trait): one method per product in the family, `create_fruit()` and `create_vegetable()`.
- **Concrete factories** (`TropicalFarm`, `TemperateFarm`): each makes one complete, matching family.
- **Products** (`Fruit` and `Vegetable` traits, implemented by `Mango`, `Okra`, `Apple`, `Carrot`).
- **Client** (`pack_basket`): uses only the `Farm` trait, so it works with any farm and never mixes families.

## The example

A shop packs a basket for each customer, with one fruit and one vegetable. Which farm supplies it depends on the customer's region, decided at runtime by `farm_for(region)`:

| Region | Farm | Basket |
|---|---|---|
| India, Brazil, Kenya | `TropicalFarm` | mango + okra |
| anywhere else | `TemperateFarm` | apple + carrot |

`pack_basket` only ever sees `&dyn Farm`. Adding a third family, say a `MediterraneanFarm` with figs and artichokes, means writing one new type that implements `Farm`. `pack_basket` and `describe` don't change.

## The Rust way

- **The abstract factory is a trait** with one method per product. Each family is a type implementing it.
- **Products come back as trait objects** (`Box<dyn Fruit>`), so each farm can return its own concrete type.
- **The farm itself is chosen at runtime** as a `Box<dyn Farm>`. If the family is known at compile time, you can use generics instead (`fn pack_basket<F: Farm>(farm: &F)`), and the compiler generates a separate, slightly faster version for each farm.

## When to use it

- Your program creates several kinds of objects that must **match each other**: UI widgets for one theme, database connection and query builder for one database, parsers and formatters for one file format.
- You want to switch the whole family at once, in one place.

**When not to:** if there's only one product, or the products don't need to match, a plain Factory is simpler.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p abstract-factory-pattern`.
