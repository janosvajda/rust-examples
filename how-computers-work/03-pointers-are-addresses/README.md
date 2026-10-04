<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Pointers are addresses

## The idea in one sentence

A **pointer** is just an address, a mailbox number, stored like any other number; following it leads to the value, and a wrong number leads somewhere it shouldn't, which is why Rust checks pointers so strictly.

## Imagine…

Imagine writing a mailbox number on a slip of paper: "mailbox 1,204". The slip isn't the treasure. It tells you where the treasure is. You can copy the slip and hand it to a friend, and you can put the slip itself into a mailbox. Following the slip, going to mailbox 1,204 and looking inside, is called **dereferencing**.

Now imagine the slip says "mailbox 9,999,999", but that mailbox belongs to someone else, or doesn't exist, or the treasure moved out last week. Following the slip leads to the wrong place. With pointers, that's one of the most common causes of crashes and security holes. Rust's references are slips that are **guaranteed** to point somewhere valid.

## Precisely

### A reference holds an address

```text
score lives at       0x16bb99d0c
the reference holds  0x16bb99d0c
following it gives   42
the reference itself is 8 bytes: just the address
```

`&score` makes a reference: a pointer to `score`. Inside, it's the address of `score`'s first byte, an 8-byte number on a 64-bit computer. `*pointer` follows it. (The addresses differ on every run.)

### Indexing is arithmetic

```text
numbers[0]: start + 0 × 4 = 0x16bb99db0  (Rust used 0x16bb99db0)
numbers[1]: start + 1 × 4 = 0x16bb99db4  (Rust used 0x16bb99db4)
numbers[2]: start + 2 × 4 = 0x16bb99db8  (Rust used 0x16bb99db8)
```

`numbers[i]` is computed as **start + i × size**, exactly as the program calculates it by hand. That's what makes an array fast: finding element 1,000,000 is one multiplication and one addition, not a million steps.

### A pointer to a pointer

A pointer is a value, so it has an address too, and something can point to it:

```text
the outer reference holds 0x16bb99d10, where the inner one is stored
following both gives 42
```

`&&u32` is a slip of paper telling you where to find another slip. `**` follows both. Linked lists, trees and graphs are made of values that hold pointers to other values ([data structures](../../data-structures/)).

### What goes wrong with pointers, and what Rust does about it

| Mistake | In C | In Rust |
|---|---|---|
| **reading past the end** of an array (`numbers[10]` in an array of 4) | reads whatever is in the next mailboxes: other variables, passwords, anything | every index is checked: the program stops with `index out of bounds: the len is 4 but the index is 10`, or `numbers.get(10)` returns `None` |
| **a pointer to something that's gone** (a *dangling* pointer) | reads or overwrites memory that now belongs to something else | doesn't compile: `` error[E0597]: `x` does not live long enough `` |
| **a null pointer**, "no address" (`NULL`) | crashes, if you forget to check | references are never null; "maybe no value" is an `Option<&T>`, and the compiler makes you handle `None` |

Two details are worth knowing:
- **The check costs almost nothing.** It's one comparison, and the compiler removes it whenever it can prove the index is always valid. Iterating with `for n in &numbers` needs no check at all, because there's no index.
- **`Option<&u32>` is still 8 bytes.** A reference can never be address 0, so Rust uses address 0 to mean `None`. "Maybe a pointer" costs no extra space, just like C's `NULL`, but you can't forget to check it.

Rust also has **raw pointers** (`*const T`, `*mut T`), which have no guarantees, like C's. They're needed for talking to hardware and C libraries, and following one requires `unsafe`. The [Pointers and memory](../../pointers-and-memory/) course covers them, and all of Rust's pointer types, in depth.

## A bit of history

| When | What happened |
|---|---|
| **1940s–50s** | In machine code and assembly, every memory access uses an address. Using a number *as* an address, "the value stored here tells you where to look next", is called **indirect addressing**, and early machines already supported it |
| **1964** | Harold Lawson adds **pointer variables** to the PL/I language, for building linked lists. He later received the IEEE Computer Pioneer Award for inventing the pointer variable |
| **1965** | Tony Hoare adds the **null reference** to ALGOL W, "simply because it was so easy to implement". In 2009 he called it his **"billion-dollar mistake"**, for all the crashes and errors it caused |
| **1972** | **C** makes pointers a central part of the language, and pointer arithmetic (`p + 1`) everyday programming. It's powerful and fast, with no checks: the topic of [lesson 9](../09-why-rust/) |
| **2015** | **Rust 1.0**: references that are checked when the program is compiled, with no garbage collector and no cost while it runs |

## Run it

```bash
cargo run
cargo test
```

Previous: [Lesson 2: Memory and addresses](../02-memory-and-addresses/) · Next: [Lesson 4: A tiny CPU](../04-a-tiny-cpu/)
