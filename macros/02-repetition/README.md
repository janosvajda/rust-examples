<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: Repetition

## The idea in one sentence

`$( … ),*` matches something **repeated** in a macro call, and the same `$( … )*` in the template **repeats the output** once for each match. That's how `vec![1, 2, 3]` takes any number of items.

## How a repetition works

```rust
macro_rules! my_vec {
    ( $( $item:expr ),* ) => {{
        let mut v = Vec::new();
        $( v.push($item); )*
        v
    }};
}
```

```text
  my_vec![1, 2, 3]

  pattern   $( $item:expr ),*      matches 1 , 2 , 3      → $item = [1, 2, 3]
  template  $( v.push($item); )*   repeats once per item  → v.push(1); v.push(2); v.push(3);
```

The parts of a repetition:

```text
  $(  …  )  ,  *
   └─┬─┘    │  └── how many: * zero or more, + one or more, ? zero or one
     │      └───── separator between repetitions (optional: , or ; are common)
     └──────────── what repeats
```

## The three repetition operators

| Operator | Means | Example in the demo |
|---|---|---|
| `*` | zero or more | `my_vec![]` and `my_vec![1, 2, 3]` both work |
| `+` | one or more | `sum!(…)` needs at least one number, so `sum!()` doesn't compile |
| `?` | zero or one: optional | `greeting!("Ferris")` or `greeting!("Ferris", with "!!!")` |

## Repeating more than one piece

A repetition can contain several fragments. They repeat together:

```rust
macro_rules! hashmap {
    ( $( $key:expr => $value:expr ),* $(,)? ) => {{
        let mut map = HashMap::new();
        $( map.insert($key, $value); )*
        map
    }};
}

let capitals = hashmap! {
    "Hungary" => "Budapest",
    "Austria" => "Vienna",      // ← trailing comma
};
```

`$(,)?` at the end is a common idiom: it accepts an optional **trailing comma**, so a list can have one entry per line with every line ending in a comma, just like `vec!` and struct literals allow.

## Recursion: a macro calling itself

```rust
macro_rules! max {
    ( $x:expr ) => { $x };                       // one value: that's the max
    ( $x:expr, $( $rest:expr ),+ ) => {{         // several: compare the first with the max of the rest
        let rest = max!( $( $rest ),+ );
        if $x > rest { $x } else { rest }
    }};
}
```

`max!(3, 9, 4, 1)` expands into `max!(9, 4, 1)`, then `max!(4, 1)`, then `max!(1)`, which matches the first rule and stops. Like a recursive function, a recursive macro needs a **base case** that comes first.

The same technique counts arguments: `count!(a b c d e)` becomes `1 + 1 + 1 + 1 + 1 + 0`. It's all worked out **at compile time**, so the result can even initialise a `const`. The tests do that.

## Where you've already seen this

| Macro | Repetition inside |
|---|---|
| `vec![a, b, c]` | `$( $x:expr ),*` |
| `println!("{} {}", a, b)` | the arguments after the format string |
| `assert_eq!(a, b, "msg {}", x)` | an optional message with its own arguments |
| `matches!(value, A \| B)` | one or more patterns |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 1: macro_rules! basics](../01-macro-rules-basics/) · Next: [Lesson 3: Macros that generate code](../03-generating-code/)
