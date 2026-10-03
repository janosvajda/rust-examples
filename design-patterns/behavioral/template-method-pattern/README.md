<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="48">

# Template Method Pattern

## What is it?

The **Template Method** pattern defines the steps of a task once, in a fixed order, and lets different variations fill in some of those steps their own way.

Think of a recipe for making a hot drink: boil water, brew, pour into a cup, add extras. The recipe is the same whether you're making tea or coffee. Only some steps differ: you *brew* tea by steeping a tea bag and coffee by filtering ground beans, and the extras might be lemon or milk. Nobody rewrites the whole recipe for each drink. You follow the same template and change the steps that differ.

## The parts

```text
  export(title, sales)          ◄── the template method: the fixed recipe
  │
  ├── 1. title(title)           ◄── optional step: has a default, can be overridden
  ├── 2. header()               ◄── required step: every format must write its own
  ├── 3. row(sale)  × each sale ◄── required step
  └── 4. footer(total)          ◄── optional step: the default writes nothing
```

- **Template method** (`export`): runs the steps in a fixed order and does the shared work, like looping over the sales and adding up the total. Every format gets it for free and doesn't override it.
- **Required steps** (`header`, `row`): every format must provide these.
- **Optional steps, or "hooks"** (`title`, `footer`): have a sensible default that a format can replace if it wants to.

## The example

A café exports its daily sales report in three formats. They all use the same `export` recipe, but fill in the steps differently:

| Format | `title` | `header` + `row` | `footer` |
|---|---|---|---|
| `CsvExporter` | overridden: no title (spreadsheets don't want one) | comma-separated values | default: nothing |
| `MarkdownExporter` | overridden: a `##` heading | a Markdown table | overridden: a bold total row |
| `PlainTextExporter` | default: a plain line | fixed-width columns | overridden: a separator and total |

Adding a fourth format, say HTML, means writing just `header` and `row`, plus any optional steps it wants. The loop, the order and the total calculation are already done, and can't be got wrong.

## The Rust way: default methods in traits

Many languages build this pattern with an abstract base class. In Rust it's a **trait with default methods**:

- A trait method **with a body** is a default that implementors get automatically. `export` is one, and it's the template.
- A trait method **without a body** must be written by every implementor (`header`, `row`).
- A default method can call other trait methods, including ones the implementor provides. That's what lets `export` call `row` without knowing which format it's working with.

The standard library uses this everywhere. `Iterator` has one required method, `next`, and dozens of default ones like `map`, `filter` and `sum`. Each is a template built on top of `next`.

## When to use it

- Several variations of a task share the same overall steps and differ in the details.
- You want the order of the steps, and the shared work, to be written once and impossible to get wrong.

**Compared with Strategy:** Strategy swaps out a whole algorithm. Template Method keeps the algorithm and swaps out individual steps inside it.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p template-method-pattern`.
