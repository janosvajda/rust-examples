<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 3: Prompting

> Written in October 2026. Prompting changes faster than anything else in this course. Check whether what you read here still holds.

## The idea in one sentence

A prompt is a **specification**: the AI can only build what you describe, and you can only describe what you understand.

## What a prompt is

A **prompt** is what you give the AI: your request, plus everything that comes with it, like the files it can see, the error message, the examples and the rules. The AI has nothing else to go on. It can't read your mind, it doesn't know your users, and in a new conversation it doesn't remember yesterday.

When the result is wrong, the cause is usually one of two things:
- the prompt **left something out** that you knew and the AI didn't;
- the prompt **asked for the wrong thing**, because you weren't sure yourself what you wanted.

Neither is fixed with magic words. Both are fixed by **knowing what you want**.

## The parts of a good prompt

| Part | Question it answers | Example |
|---|---|---|
| **Goal** | what should exist when we're done? | "a function that checks a list for duplicates" |
| **Context** | where does it fit? | "it's called for every upload, lists have up to a million items" |
| **Constraints** | what must be true? | "no new dependencies; must not panic on bad input" |
| **Done means** | how do we know it works? | "these three tests pass, and clippy is clean" |
| **Out of scope** | what must NOT happen? | "don't change the public API; don't touch git" |

You don't need all five every time. A one-line fix needs one line. But every time an AI surprises you, check which of the five was missing.

## Vague and precise, side by side

| Vague | Precise |
|---|---|
| "make it faster" | "`has_duplicates` compares every pair. Use a `HashSet` so it's one pass, keep the signature, and add a test with 100,000 items." |
| "add error handling" | "`parse_port` panics on bad input. Return `Result<u16, String>` with a message that says what was wrong. Callers already handle `Result`." |
| "fix the tests" | "`test_discount` fails. Find out whether the test or the code is wrong according to the spec in the doc comment. Don't change the test without telling me why." |
| "write docs" | "Write a README for beginners explaining the `?` operator. Use generic examples, not machine-specific details. Keep code comments short." |

The precise prompts all contain **knowledge**: what a `HashSet` is for, what `Result` means, that a test can be wrong, what the audience needs. That's the point of [Lesson 4](../04-why-you-still-need-to-know/).

## You can only ask for what you can name

Precise words lead to precise results. Each bold word carries a whole design decision:

| Without the word | With the word | What you get |
|---|---|---|
| "store the words so lookup is fast" | "use a **trie**, I need prefix search" | a structure built for prefixes |
| "don't copy so much" | "take a **`&[T]` slice**, return **`&str`**, no `clone`" | borrowing instead of copying |
| "make it not crash" | "return **`Result`**, no **`unwrap`** outside tests" | errors the caller can handle |
| "it should work on the small chip" | "**`no_std`**, no heap allocation" | code that fits a microcontroller |
| "it's slow with big inputs" | "this is **O(n²)**, I want **O(n)**" | an algorithm that scales |

Learning the vocabulary of your field, its data structures, complexity, error handling and memory, isn't made obsolete by AI. It's what makes the AI useful.

## Habits that work

- **Ask for a plan first.** "Don't write code yet. Suggest two or three approaches with their trade-offs." Then choose. The author of this repository put it simply: *"we need to discuss the ideas first."*
- **Ask the AI to list its assumptions.** "Before you start, tell me what you're assuming about the input." Wrong assumptions are cheap to fix before the code exists.
- **Ask questions you can't answer yourself, then check the answers.** "Why does this need a lifetime?" is a great prompt. Then verify the answer against the compiler, the documentation or a test.
- **Give the error, the code and what you expected.** "It doesn't work" gives the AI nothing to go on.
- **Write rules down once.** Most AI coding tools can read a **project instruction file** that every conversation starts with. Rules like "documentation goes in README files" or "never run git commands" belong there, not repeated in every prompt. The file names and formats differ between tools and keep changing.

## A real example: one sentence changed the design

While improving the Mini compiler in this repository, the AI pinned it to one LLVM version, and it failed to build with the LLVM that was installed. The human's prompt was:

> *"please, do not specify LLVM, use that what is installed … define min version 16 and above"*

That's a perfect constraint: short, clear, and about **what** must be true, not **how**. The AI then had to find a way to meet it. The answer was to write LLVM IR as text and run the installed LLVM's own `llc`. That design was simpler and needed one dependency fewer than the original. A vaguer prompt like "fix the build" would have produced a version bump and the same problem next year.

## What may change

This lesson will age fastest. Early AI models needed tricks: special phrases, role-play ("you are an expert…"), careful word order. Most of those tricks matter far less now, and today's habits may matter less tomorrow. What won't age is the part that was never a trick: **knowing what you want, and saying it precisely.**

Previous: [Lesson 2: AI-assisted engineering](../02-ai-assisted-engineering/) · Next: [Lesson 4: Why you still need to know](../04-why-you-still-need-to-know/)
