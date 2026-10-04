<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 2: AI-assisted engineering

> Written in October 2026. AI tools change every month. Check whether what you read here still holds.

## The idea in one sentence

The line between vibe coding and engineering isn't **how much** code the AI writes, but whether **a human understands, checks and owns** the result.

## Who does what

The AI and the human are good at different things:

| The AI is great at | The human must do |
|---|---|
| typing code quickly | decide **what** to build and **why** |
| boilerplate, conversions, repetitive changes | set the **constraints**: speed, safety, which tools may be used |
| suggesting several approaches | **choose** between them, knowing the project |
| explaining unfamiliar code or errors | **judge** whether the result is right |
| finding bugs, also in code a human wrote | take **responsibility** for what ships |

An AI can propose a decision, but it can't be responsible for one. If the code loses someone's data, "the AI wrote it" isn't an answer.

## A workflow that works

1. **Understand the problem yourself** before asking for code. What's the input, what's the output, what can go wrong?
2. **Discuss before building.** Ask for options and a plan, not code. Agree on the approach first. Changing a plan costs a sentence; changing a thousand lines of code costs an afternoon.
3. **Work in small steps.** One change at a time is something you can actually read.
4. **Read every change**, the way you'd review a colleague's code (see [Lesson 9](../09-reviewing-ai-code/)).
5. **Verify with tools, not feelings.** It compiles, `cargo clippy` is clean, the tests pass, and you ran it. Lesson 6 explains why these matter so much.
6. **Own it.** Once you accept the code, it's your code.

## Real examples from this repository

These moments happened while the author of this repository worked with an AI assistant on this very repository. They show both directions: the human correcting the AI, and the AI catching the human's bugs.

### When the human had to steer

| What the AI did | What the human did | The lesson |
|---|---|---|
| Asked to document and optimise the data structures, the AI added shell scripts, benchmarks and long comments full of machine-specific assembly. | "I did not request this." Asked for generic explanations in README files instead. | The AI guessed what "optimise" and "machine-level documentation" meant and didn't ask. A plan agreed first would have saved the work. |
| For the Mini compiler, the AI suggested an optional interpreter instead of LLVM, to avoid the LLVM setup. | "This language MUST create real machine code." | The AI optimised for "easy to build". The human knew what the project is **for**. |
| The AI pinned Mini to one exact LLVM version. | "Use what is installed, minimum version 16." | That constraint led to a better design, which writes LLVM IR as text and needs no LLVM library at all. |
| The AI ran a `git add` command the human hadn't asked for. | "Never touch git." The rule was saved, and the AI followed it from then on. | Agents that can run commands need clear **boundaries**. Who commits, pushes or deletes is the human's decision. |

### When the AI caught real bugs

| What the AI found | How |
|---|---|
| Mini quietly read `let a = 2 % 3;` as `2`, and `let x = 5; let y = 6;` lost `y`. | It ran unusual inputs through the real compiler while writing the documentation. |
| Code shown in comic images didn't compile: a missing argument and a wrong method. | It compiled every snippet instead of trusting how it looked. |
| A function that returns the "longest name" gave different answers for a tie, depending on how it was written ([Lesson 4](../04-why-you-still-need-to-know/)). | Running both versions side by side. |

Notice that in every one of these, **the bug was found by running something**, not by reading and believing. That's true for humans and AIs alike.

## Agents: AIs that act

Many AI tools no longer just suggest code. They're **agents**: they edit files, run commands, run tests and read the results, many steps in a row. That makes them far more useful, and it makes three things more important:

- **Permissions:** decide what the agent may do on its own and what needs your approval. Typical examples are deleting files, running git commands and anything that talks to the internet.
- **Small, reviewable steps:** a hundred changed files are almost impossible to review.
- **Automatic checks:** an agent that runs `cargo clippy` and `cargo test` after each change catches many of its own mistakes. Lesson 6 shows what those checks find.

## What may change

Agents are improving fast. They take on bigger tasks and make fewer mistakes, and the right amount of supervision will change with them. Whether you check every line, every file or every feature may differ next year. That *someone* checks, and someone is responsible, won't.

Previous: [Lesson 1: What is vibe coding?](../01-what-is-vibe-coding/) · Next: [Lesson 3: Prompting](../03-prompting/)
