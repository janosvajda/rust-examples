<img src="../../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

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

- **Template method** (`export`): runs the steps in a fixed order and does the shared work, like looping over the sales and adding up the total. All three formats reuse its default implementation.
- **Required steps** (`header`, `row`): every format must provide these.
- **Optional steps, or "hooks"** (`title`, `footer`): have a sensible default that a format can replace if it wants to.

## The example

A café exports its daily sales report in three formats. They all use the same `export` recipe, but fill in the steps differently:

| Format | `title` | `header` + `row` | `footer` |
|---|---|---|---|
| `CsvExporter` | overridden: no title (spreadsheets don't want one) | comma-separated values | default: nothing |
| `MarkdownExporter` | overridden: a `##` heading | a Markdown table | overridden: a bold total row |
| `PlainTextExporter` | default: a plain line | fixed-width columns | overridden: a separator and total |

Adding a fourth format, say HTML, means writing `header` and `row`, plus any optional steps it wants. Reusing `export` keeps the loop, order and total calculation in one place. Rust permits overriding any default trait method, including `export`; this design shares a recipe but does not enforce that every future implementor follows it.

## The Rust way: default methods in traits

Many languages build this pattern with an abstract base class. In Rust it's a **trait with default methods**:

- A trait method **with a body** is a default that implementors get automatically. `export` is one, and it's the template.
- A trait method **without a body** must be written by every implementor (`header`, `row`).
- A default method can call other trait methods, including ones the implementor provides. That's what lets `export` call `row` without knowing which format it's working with.

The standard library also uses default trait methods extensively. `Iterator` has one required method, `next`, and many default methods such as `map`, `filter` and `sum` that build on the iterator's behavior. Implementors can override defaults when a specialized implementation is useful.

## When to use it

- Several variations of a task share the same overall steps and differ in the details.
- You want the order of the steps and the shared work to be written once and reused by several implementations.

**Compared with Strategy:** Strategy swaps out a whole algorithm. Template Method keeps the algorithm and swaps out individual steps inside it.

## Every format has its own special characters

- **CSV:** a product name containing a comma or a quote, like `Tea, "iced"`, is put in quotes, with every inner quote doubled, so it stays one field.
- **Markdown:** a `|` would end a table cell, so it's written as `&#124;`; a line break becomes `<br>`; and HTML characters like `<` are escaped, so a name can't inject HTML.

Prices are whole cents, multiplied and added up in `u64` with checked arithmetic, so the total is exact and can't silently overflow.

## Run it

```bash
cargo run
cargo test
```

From the repository root: `cargo run -p template-method-pattern`.
