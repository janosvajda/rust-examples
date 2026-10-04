<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# How Mini turns your program into a real program

## The computer doesn't understand your words

When you write a Mini program, you write words that people can read:

```
let apples = 3;
let pears = 4;
let fruit = apples + pears;
print fruit;
```

But the computer's brain, the **processor**, can't read words. It only understands **numbers**: long lists of tiny, simple commands, each one written as a number. These are commands like "add these two numbers" or "remember this number". We call this list **machine code**.

Machine code is very hard for people to read. A few commands look something like this:

```text
82 00 80 52   E1 03 00 AA   40 00 00 94   C0 03 5F D6
```

So we have a problem:
- **people** like to write words;
- **computers** only understand numbers.

We need a **translator** between them. In programming, that translator is called a **compiler**.

## What a compiler is

A **compiler** is a program that reads your program and translates it into machine code.

Think of a pen pal in Japan. You write a letter in English, but your friend only reads Japanese. A translator reads your letter and writes the same message in Japanese. Your friend never sees your English. They only read the translation.

```
   your Mini program   ──►   COMPILER   ──►   machine code
   (words for people)                         (numbers for the processor)
```

**Mini is a compiler.** When you type

```bash
mini fruit.mini ./fruit
```

Mini reads `fruit.mini` and makes `fruit`, a real program that your computer can run by itself.

## A compiler works in small steps

Translating a whole program in one jump would be really hard. So Mini works like a factory with an **assembly line**: each worker does one small job and passes the result to the next one.

Here's what happens to the line `let x = 2 + 3;`.

### Step 1: chop it into words (the lexer)

First, Mini cuts the line into small pieces called **tokens**, like cutting a sentence into words:

```text
let   x   =   2   +   3   ;
```

Now Mini knows: "this is the word `let`, this is the name `x`, this is the number `2`…"

### Step 2: understand the sentence (the parser)

Next, Mini works out what the pieces **mean together**, the way you understand a sentence and not just its words. It builds a **tree**:

```text
        let x
          │
          +
         / \
        2   3
```

The tree says: "make a variable called `x`, and its value is 2 plus 3".

This is also when Mini notices mistakes. If you forget the `;`, Mini tells you which line is wrong, like a teacher marking a spelling mistake.

### Step 3: check that it makes sense (the type check)

Some sentences have correct grammar but still make no sense, like "the banana drove to school". Mini checks for that too. Adding two numbers is fine, but adding a number and a piece of text isn't:

```
let n = 5 + "hello";      // ✗ Mini says: that doesn't make sense!
```

### Step 4: write it in the in-between language (LLVM IR)

Now Mini writes your program again, in a language called **LLVM IR**. It sits halfway between your words and machine code: it's still text, but the steps are already as small as the processor's.

```text
%t0 = add i32 2, 3
```

This says: "add 2 and 3, and call the answer `%t0`". After you build a program, Mini leaves this file next to it (for example `fruit.ll`), so you can open it and see what your program turned into!

### Step 5: turn it into machine code (LLVM)

Last, a big helper program called **LLVM** turns the in-between language into machine code: the numbers the processor understands.

Why doesn't Mini do this last step itself? Because **different computers speak different machine code**. A Mac with an Apple chip, a Windows laptop and a Raspberry Pi each have a different kind of processor, and each one understands different numbers. LLVM already knows how to speak to all of them. Mini only has to speak LLVM's language, and LLVM does the rest. Many real languages use the same trick, including Rust, Swift and C.

```text
             your program
                  │
            Mini (steps 1–4)
                  │
               LLVM IR
                  │
                LLVM
         ┌────────┼────────┐
         ▼        ▼        ▼
     Apple chip  Intel   Raspberry Pi
```

## What a linker is

After step 5, the machine code is ready, but there's a piece missing.

Your program says `print fruit;`. But showing something on the screen is actually a lot of work, and Mini didn't write that part. Your computer **already has** a ready-made "print" program piece, in a collection of useful pieces called a **library**. Every program on your computer can use it.

So now there are two pieces:

1. **your machine code**, which says "print this number here";
2. **the library**, where the "print" piece really lives.

The **linker** joins them together into one program you can run.

Think of building with LEGO. You built your own model, and the instructions say "now attach the wheels". You didn't make the wheels yourself: they come from the box. The linker is the step where you **click the wheels onto your model**, so it can actually drive.

```text
   your machine code  ──┐
                        ├──►  LINKER  ──►  ./fruit   (your finished program!)
   the "print" piece  ──┘
   from the library
```

Mini doesn't need its own linker. Every computer that's set up for programming already has one, so Mini just asks it to do the joining.

## Why do we need all of this?

| Without it… | With it… |
|---|---|
| You would have to write long lists of numbers by hand. | You write `let fruit = apples + pears;` |
| You would have to write a different program for every kind of computer. | LLVM translates for all of them. |
| Every program would have to build its own "print" from scratch. | The linker connects your program to pieces that already exist. |

## The whole journey

```text
 fruit.mini          you write words
     │
     │  Mini: chop into words, understand them, check them
     ▼
 fruit.ll            the in-between language (you can read it!)
     │
     │  LLVM: translate for your computer's processor
     ▼
 fruit.o             machine code, but the "print" piece is still missing
     │
     │  linker: click the library pieces on
     ▼
 fruit               a finished, real program 🎉
```

Then you run it:

```bash
./fruit
```

```text
7
```

## Words to remember

| Word | What it means |
|---|---|
| **processor** | the computer's brain, which follows the commands |
| **machine code** | the commands written as numbers, the only thing the processor understands |
| **compiler** | a translator from a programming language to machine code |
| **token** | one small piece of your program, like a word in a sentence |
| **parser** | the part that works out what the tokens mean together |
| **LLVM IR** | the in-between language, halfway from your words to machine code |
| **LLVM** | the helper that turns LLVM IR into machine code for many kinds of computers |
| **library** | a collection of ready-made program pieces, like "print" |
| **linker** | joins your machine code and the library pieces into one finished program |

## Try it yourself

1. Write a small program, build it, and open the `.ll` file next to it. Can you find your numbers in it?
2. Change `+` to `*` in your program, build it again, and look at the `.ll` file once more. What changed?
3. Make a mistake on purpose, like forgetting a `;`. What does Mini tell you?

When you want to learn all the things you can write in Mini, read [The Mini language](syntax.md).
