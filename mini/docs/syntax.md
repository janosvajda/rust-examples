<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# The Mini language

Mini is deliberately small. It has **two statements** (`let` and `print`), **four types of values** (whole numbers, decimals, booleans and text), and operators to calculate, compare and combine them. That's the whole language, and this page describes all of it.

## A first program

```
// My first Mini program
let apples = 3;
let pears = 4;
let fruit = apples + pears;
let label = "Pieces of fruit:";
print label;
print fruit;
```

Save it as `fruit.mini`, then compile and run it:

```bash
mini fruit.mini ./fruit
./fruit
```

```text
Pieces of fruit:
7
```

`mini` turns your program into a real executable, `./fruit`, which runs without Mini. It also leaves `fruit.ll` next to it: your program translated into LLVM IR, which you can open and read.

## How a program is laid out

- The computer runs the statements **from top to bottom**, one after another.
- **One statement per line**, and every statement **ends with `;`**.
- Spaces around words and symbols don't matter: `let x=5;` and `let  x  =  5 ;` are the same.
- Empty lines are ignored.
- A line that starts with `//` is a **comment**: a note for people, ignored by Mini. A comment must be on its own line, not after a statement.

```
// ✓ a comment on its own line
let x = 5;

let y = 6;  // ✗ error: unrecognized syntax
```

## Names

A variable's name starts with a letter or `_`, followed by letters, digits or `_`:

| ✓ allowed | ✗ not allowed |
|---|---|
| `x`, `total`, `max_speed`, `_tmp`, `player2` | `2player` (starts with a digit), `max-speed` (contains `-`), `my name` (contains a space) |

Upper and lower case are different: `Total` and `total` are two different variables.

These **keywords** have a special meaning, so they can't be used as names: `let`, `print`, `true`, `false`, `and`, `or`, `not`. Keywords are always written in lower case.

## Values and their types

Every value in Mini has a **type**. The type decides what you can do with the value.

| Type | Examples | What it's for |
|---|---|---|
| whole number | `42`, `-7`, `0` | counting things |
| decimal | `2.5`, `19.99`, `-0.05` | amounts with a fractional part: prices, measurements |
| boolean | `true`, `false` | answers to yes/no questions |
| text | `"Hello"` | messages |

### Whole numbers

```
let year = 2025;
let below_zero = -15;
```

- Whole numbers have no fractional part.
- They are **32-bit**, so they range from **−2,147,483,648 to 2,147,483,647**. A positive literal larger than 2,147,483,647 is an error; the special negative literal `-2147483648` is accepted.
- If a calculation goes past the largest value, it **wraps around** to the smallest one: `2147483647 + 1` gives `-2147483648`. This is how the processor's 32-bit arithmetic works.

### Decimals

```
let price = 19.99;
let temperature = -3.5;
let ratio = 0.125;
```

- A decimal has a point **with digits on both sides**: `2.5` and `0.5` are decimals. `2.` and `.5` are errors, and `2.0` is the decimal two.
- Decimals are **exact**, with up to **6 digits after the point**: `0.1 + 0.2` is exactly `0.3`. A decimal written with more than 6 digits after the point is an error.
- They range up to about **±9,223,372,036,854** (nine trillion).
- `print` writes no unnecessary zeros: `2.50` prints as `2.5`, and the decimal two prints as `2.0`.

> **Why exact?** Many languages store fractional numbers as binary *floating point*, where `0.1 + 0.2` is `0.30000000000000004`, because 0.1 can't be written exactly in binary. Mini stores a decimal as a whole number of **millionths** instead: `2.5` is stored as 2,500,000 millionths. Representable millionths are exact, but multiplication/division may discard fractional millionths and overflow can wrap. Banking and accounting software uses the same trick.

### Booleans

```
let raining = true;
let finished = false;
```

A boolean is one of exactly two values: `true` or `false`. You usually get booleans from comparisons (see below), like `let adult = age >= 18;`.

### Text

```
let greeting = "Hello, world!";
let quote = "She said \"hi\"";
```

Text goes between double quotes. To put special characters inside, write an **escape**, a backslash followed by a letter:

| Write | To get |
|---|---|
| `\n` | a new line |
| `\t` | a tab |
| `\"` | a double quote |
| `\\` | a backslash |

Any other letter after a backslash, like `\q`, is an error.

## Types don't mix

The two sides of an operator must have **the same type**. Mini never converts one type into another for you:

```
let a = 1 + 2.5;      // ✗ type error: cannot use `+` on a number and a decimal
let b = 1.0 + 2.5;    // ✓ 3.5
```

This rule (the same one Rust has) means a value never quietly changes its type.

## `let`: create a variable

```
let name = value;
```

`let` gives a value a name, so you can use it later. The value can be any expression, or a piece of text:

```
let width = 8;
let height = 5;
let area = width * height;
let unit = "square metres";
let is_big = area > 30;
```

The variable gets the type of its value: `area` is a whole number, `unit` is text, `is_big` is a boolean.

**Using `let` again with the same name** replaces the old variable. The right side is calculated first, using the old value, so counting up works:

```
let score = 10;
let score = score + 5;
print score;            // prints 15
```

A variable must be created **before** it's used. Any variable can be copied into a new one: `let backup = score;`.

## Arithmetic

Arithmetic works on **whole numbers** and on **decimals**:

| Operator | Meaning | Whole numbers | Decimals |
|---|---|---|---|
| `+` | add | `7 + 2` is `9` | `0.1 + 0.2` is `0.3` |
| `-` | subtract | `7 - 2` is `5` | `5.0 - 0.25` is `4.75` |
| `*` | multiply | `7 * 2` is `14` | `19.99 * 3.0` is `59.97` |
| `/` | divide | `7 / 2` is `3` | `7.0 / 2.0` is `3.5` |
| `-` in front | negate | `-x` | `-x` |

**Division** never rounds up:
- Whole numbers keep only the whole part, rounding towards zero: `7 / 2` is `3`, and `-7 / 2` is `-3`.
- Decimals keep 6 digits after the point and cut off the rest: `1.0 / 3.0` is `0.333333`, and `2.0 / 3.0` is `0.666666`.

**Invalid division traps.** A zero divisor terminates the generated program. Integer `-2147483648 / -1` also traps because its result does not fit `i32`. These checks run before LLVM division; the language does not expose a recoverable runtime error.

## Comparisons

A comparison asks a question about two values. The answer is a **boolean**:

| Operator | Question | Example | Result |
|---|---|---|---|
| `==` | equal? | `3 == 3` | `true` |
| `!=` | not equal? | `3 != 3` | `false` |
| `<` | less than? | `2.5 < 3.0` | `true` |
| `<=` | less than or equal? | `4 <= 4` | `true` |
| `>` | greater than? | `1 > 2` | `false` |
| `>=` | greater than or equal? | `age >= 18` | depends on `age` |

- Whole numbers and decimals can use all six operators.
- Booleans can use `==` and `!=`.
- Text can't be compared yet.
- Note the double `==`: a single `=` belongs to `let`. Writing `1 = 2` in an expression is an error that reminds you to write `==`.

Comparisons can't be chained like in maths. `1 < 2 < 3` is an error, because `1 < 2` is a boolean and a boolean can't be compared with `3` using `<`. Write `1 < 2 and 2 < 3` instead.

## Logic: `and`, `or`, `not`

These combine booleans:

| Expression | Is `true` when… |
|---|---|
| `a and b` | **both** `a` and `b` are true |
| `a or b` | **at least one** of `a`, `b` is true |
| `not a` | `a` is false |

```
let age = 20;
let has_ticket = true;
let can_enter = age >= 18 and has_ticket;
print can_enter;        // prints true
```

## The order of operations

When an expression has several operators, Mini works them out in this order, from first to last:

| Order | Operators |
|---|---|
| 1 | parentheses `( … )` |
| 2 | negation `-x` |
| 3 | `*` `/` |
| 4 | `+` `-` |
| 5 | comparisons `==` `!=` `<` `<=` `>` `>=` |
| 6 | `not` |
| 7 | `and` |
| 8 | `or` |

So:
- `2 + 3 * 5` is `2 + 15`, which is `17`.
- `price * 2.0 > 10.0` compares the product with `10.0`.
- `not age < 18` means `not (age < 18)`.
- `a or b and c` means `a or (b and c)`.

Operators of the same level are worked out **from left to right**: `10 - 3 - 2` is `(10 - 3) - 2`, which is `5`. When in doubt, add parentheses: they always work and make the meaning clear to readers.

## `print`: show a value

```
print name;
```

`print` writes the value of a variable and then moves to a new line. Booleans print as `true` or `false`. It prints **variables only**. To print a number or a piece of text, put it in a variable first:

```
print "hi";             // ✗ error: unrecognized syntax

let message = "hi";
print message;          // ✓
```

## When something is wrong

Mini stops at the first mistake and tells you what's wrong. Mistakes in how a line is written also give its line number.

| You wrote | Mini says | What's wrong |
|---|---|---|
| `let x = 5` | `line 1: unrecognized syntax` | the `;` is missing |
| `print "hi";` | `line 1: unrecognized syntax` | `print` takes a variable name |
| `let x = (1 + 2;` | ``line 1: bad expression `(1 + 2` `` | a `(` is never closed |
| `let x = 1 2;` | ``line 1: bad expression `1 2` `` | two numbers with no operator between them |
| `let a = 2 % 3;` | ``unexpected character `%` `` | Mini has no `%` operator |
| `let p = 1.1234567;` | ``decimal `1.1234567` has more than 6 digits after the point`` | decimals keep 6 places |
| `let b = 1 = 2;` | ``unexpected `=`: to compare two values, write `==` `` | comparison needs `==` |
| `let or = 1;` | ``line 1: `or` is a keyword and can't be a variable name`` | pick another name |
| `let s = "bad \q";` | `line 1 string literal` | `\q` isn't a known escape |
| `print total;` (never created) | ``undefined variable `total` `` | create it with `let` first |
| `let n = 1 + 2.5;` | ``type error: cannot use `+` on a number and a decimal`` | write `1.0 + 2.5` |
| `let b = true + true;` | ``type error: cannot use `+` on booleans`` | arithmetic needs numbers |
| `let b = not 5;` | ``type error: `not` needs a boolean, found a number`` | `not` works on booleans |

## The whole grammar

This is the complete set of rules for Mini, in a common notation: `|` means "or", `( … )*` means "repeated zero or more times", `[ … ]` means "optional", and words in quotes are written literally. The rules are listed from the loosest operator (`or`) to the tightest, which is how the order of operations is built into the grammar.

```text
program     = ( line "\n" )*
line        = statement | comment | empty
statement   = "let" name "=" ( text | expression ) ";"
            | "print" name ";"
comment     = "//" any text

expression  = and_expr ( "or" and_expr )*
and_expr    = not_expr ( "and" not_expr )*
not_expr    = "not" not_expr | comparison
comparison  = sum ( ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) sum )*
sum         = product ( ( "+" | "-" ) product )*
product     = negation ( ( "*" | "/" ) negation )*
negation    = "-" negation | atom
atom        = number | decimal | "true" | "false" | name | "(" expression ")"

name        = letter_or_underscore ( letter_or_underscore | digit )*   (but not a keyword)
number      = digit digit*
decimal     = digit digit* "." digit [digit] [digit] [digit] [digit] [digit]
text        = '"' ( any character except '"' and '\' | escape )* '"'
escape      = "\n" | "\t" | "\"" | "\\"
```

## What Mini doesn't have yet

Mini has no `if`, no loops and no functions yet. See the roadmap in the [Mini README](../README.md).

An expression may contain at most 256 tokens. This bounds recursive parsing and code generation; longer expressions return an error. Boolean `and` and `or` evaluate both operands from left to right. They do **not** short-circuit, so `false and (1 / 0 == 0)` still traps.
