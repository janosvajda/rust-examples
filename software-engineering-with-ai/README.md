<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Software Engineering with AI

AI can now write a lot of code. This course is about what that changes, and what it doesn't. It covers vibe coding and what isn't vibe coding, prompting, why a programmer still has to know their craft, and why clippy and tests matter more than ever.

## ⚠️ Read this first: this course will age

**The AI industry changes every day.** New models, new tools, new prices and new ways of working appear every month. Something that's true while you read this may be false tomorrow:
- the things AI does badly may soon be things it does well;
- today's best tools and prompting tricks may be forgotten next year;
- even the words, like "vibe coding" or "agent", may change their meaning.

So:
- **This course was written in October 2026.** Each lesson repeats the date, to remind you.
- **Read it critically.** If a claim here doesn't match what you see, trust what you can check, and check it.
- **Some parts last longer than others.** We think these will stay true for a long time:
  - someone has to know what "correct" means;
  - someone has to check the code;
  - someone is responsible for it.

  The tools that help with all three will keep changing.

## These lessons contain opinions

This course is less about facts like "this is how `?` works" and more about **judgement**. The lessons give opinions and say so.

They were written with an AI assistant (Claude), in a conversation with the author of this repository. Many examples are **real moments from that conversation**:
- times the AI got something wrong and the human corrected it;
- times the AI found real bugs.

These examples show both the benefits and the risks of using AI in software development.

## The lessons

| # | Lesson | You'll learn |
|---|---|---|
| 1 | [What is vibe coding?](01-what-is-vibe-coding/) | where the term comes from, when it's fine, when it's dangerous |
| 2 | [AI-assisted engineering](02-ai-assisted-engineering/) | what isn't vibe coding: the human sets the direction and checks the work; real examples |
| 3 | [Prompting](03-prompting/) | a prompt is a specification; you can only ask for what you can name |
| 4 | [Why you still need to know](04-why-you-still-need-to-know/) | **Rust code:** plausible AI-style code next to good code, and the tests that tell them apart |
| 5 | [Why Rust fits AI-assisted development](05-why-rust-fits-ai/) | the compiler as a tireless reviewer: types, ownership, exhaustive `match`; and the honest downsides |
| 6 | [Clippy and tests as guardrails](06-clippy-and-tests-as-guardrails/) | **Rust code:** what clippy catches, stricter lints, tests that check intent instead of copying output |
| 7 | [From day one, or later?](07-from-day-one-or-later/) | why clippy and tests belong in the first commit; the prototype exception; adding them to old code |
| 8 | [Fewer dependencies](08-fewer-dependencies/) | why every external package is a decision; what AI changes; how to check what you depend on |
| 9 | [Reviewing AI-written code](09-reviewing-ai-code/) | a checklist for reading code you didn't write |
| 10 | [The shadow side](10-the-shadow-side/) | fading skills, the learning paradox, feeling faster vs being faster; why reading, maths and coding still matter; learning with AI as a tutor |

## The course on one page

| Idea | In one sentence |
|---|---|
| Vibe coding | Accepting AI code without reading it: fine for throwaway code, dangerous for anything that matters. |
| AI-assisted engineering | The AI can write most of the code; the human understands, checks and owns all of it. |
| Prompting | A good prompt is a good specification, and writing one takes knowledge. |
| Knowledge | AI writes code that *looks* right; only someone who knows better notices when it isn't. |
| Rust | Its compiler rejects whole classes of mistakes before the program runs, which is exactly what AI-written code needs. |
| Clippy and tests | Tireless automatic reviewers, for you and for the AI. |
| When to add them | From the first commit: almost free then, expensive later. |
| Dependencies | Every package is code you trust without reading it; add one on purpose, never by default. |
| The shadow side | AI can stand in for thinking, unlike a calculator; never stop learning maths, writing and critical thinking. |
| What lasts | Knowing what "correct" means, checking it, and taking responsibility. |

## Run the code lessons

```bash
cd software-engineering-with-ai/04-why-you-still-need-to-know
cargo run
cargo test
```

Or from the repository root: `cargo run -p why-you-still-need-to-know`, `cargo run -p clippy-and-tests-as-guardrails`.
