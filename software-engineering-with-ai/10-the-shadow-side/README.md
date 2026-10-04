<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 10: The shadow side, and why you should still learn

> Written in October 2026. The research on how AI affects learning and work is young, and AI tools change every month. Check whether what you read here still holds.

## The idea in one sentence

AI can make you faster while making you **weaker**, unless you keep learning the basics: reading carefully, working with numbers and maths, and writing code yourself.

## A note before we start

This course was written with an AI assistant, and this lesson is that AI and the repository's author agreeing that you shouldn't depend on AI too much. That isn't a contradiction. A good tool should make you stronger, not replace you, and it's worth knowing how it can do the opposite.

## 1. Skills fade when you stop using them

Airline pilots know this problem well. Autopilot flies most of the flight, so pilots get less practice flying by hand. When the autopilot gives up in a difficult moment, the pilot must suddenly be at their best, with skills that have quietly faded. Aviation answers this with rules that make pilots keep practising by hand.

Programming has the same risk. If the AI always writes the loop, the error handling and the SQL query, you slowly lose the ability to write them, and with it **the ability to judge them**. And judging is exactly the job [lesson 2](../02-ai-assisted-engineering/) gave the human.

Early research points the same way. A 2025 study by Microsoft Research and Carnegie Mellon University surveyed knowledge workers who use AI. The more people trusted the AI, the less critical thinking they reported putting into their work.

## 2. The learning paradox

Here's the uncomfortable core of this lesson:

> **To check AI-written code, you need exactly the skills you build by writing code yourself, which is the work the AI now does for you.**

An experienced programmer learned to spot an O(n²) loop by writing slow loops, waiting for them, and fixing them. Someone who starts their career with AI writing every loop may never go through that struggle, and may never get the instinct.

The struggle isn't a waste of time. Teachers call it **productive struggle**: getting stuck, trying, being wrong and finally understanding is how knowledge gets stored in your head. When an AI removes the struggle, it can also remove the learning.

## 3. Feeling faster isn't being faster

In 2025, the research group METR ran a careful experiment with experienced open-source developers working on their own projects. Some tasks were done with AI tools, some without.
- **What happened:** with the AI tools, the developers were about **19% slower**.
- **What they believed:** afterwards, they thought the AI had made them about **20% faster**.

It was one small study, with the tools of early 2025, and newer tools may well give different results. The lesson that lasts isn't "AI makes you slower". It's: **how productive you *feel* isn't a measurement.** Measure.

## 4. Confident and wrong

AI answers sound equally sure whether they're right or wrong. People tend to trust confident, fluent answers, and to trust automated systems more than they should. This is called **automation bias**.

The defence is understanding, not suspicion of everything. If you know the subject, a wrong answer feels wrong. If you don't, it sounds just as convincing as a right one.

## 5. Teams that don't understand their own code

When much of a codebase was written by AI and only skimmed by humans, a team can end up maintaining a system that **nobody fully understands**. Everything works until something breaks in an unexpected way. Then nobody knows where to look, and the AI, which doesn't remember why it wrote anything, is guessing too. Some people call this **comprehension debt**: like technical debt, it's invisible until you have to pay it back, with interest.

## 6. Everything else in the shadow

- **Secrets and privacy.** Code, data and documents pasted into an AI tool leave your computer. Know your tool's and your employer's rules before pasting customer data, passwords or company code.
- **Where do experienced developers come from?** If the simple tasks that juniors used to learn on are all done by AI, how does anyone become senior? Teams that stop training juniors may run out of people who can check the AI.
- **Dependence on a vendor.** Prices change, services go down, tools are discontinued. Your ability to do your job shouldn't vanish when a website does.
- **Licences and ownership.** Who owns AI-generated code, and whether it may resemble code with a licence, are legal questions that are still being settled. Know your project's rules.
- **Sameness.** When everyone asks the same tools, everyone gets similar answers. Unusual, better ideas come from people who understand the problem deeply.

## So what should people still learn?

Everything that lets you **judge**, and those skills turn out to be the old basics:

| Skill | Why it matters more with AI, not less |
|---|---|
| **Mathematics** | Exact reasoning, step by step, and a feel for numbers, so you notice when a result is absurd: a price of 850 cents that should be 849, a loop that takes a second instead of milliseconds, a total that overflows. In [lesson 6](../06-clippy-and-tests-as-guardrails/), the bug was found by **working it out by hand**. |
| **Writing** | Writing forces you to understand and order your thoughts. Prompts, specifications, reviews and documentation are all writing, and a prompt is a specification ([lesson 3](../03-prompting/)): clear writing is now a programming skill. |
| **Critical thinking** | Asking "is this true, and how do I know?" about every confident answer, from an AI or anyone else. |
| **Reading and understanding text** | Specifications, error messages, documentation and the AI's own answers are all text. Misreading one word ("optimise", "every", "never") changes the result. |
| **Writing code yourself** | You can't reliably review what you couldn't have written. Writing builds the instinct that reviewing needs. |
| **Data structures, algorithms, complexity** | The difference between 4 milliseconds and 1 second in [lesson 4](../04-why-you-still-need-to-know/) is an algorithm, and so is the right prompt to fix it. |

What matters a little less: memorising exact syntax, function names and API details. The AI and the documentation remember those well, and you can look them up.

## Learning with AI without losing the skill

AI can be a wonderful teacher. It's patient, available at midnight, and happy to explain the same thing five ways. The difference is who does the thinking:

| AI as a tutor ✓ | AI as a ghostwriter ✗ (while you're learning) |
|---|---|
| "Explain why this needs a lifetime." | "Fix this lifetime error." |
| "Give me a hint, not the solution." | "Solve this exercise." |
| "Here's my solution. What did I miss?" | "Write the solution, I'll read it." |
| "Quiz me on ownership." | "Summarise ownership for me." |

Some habits that help:
- **Try it yourself first**, then ask the AI to review your attempt.
- **Do some work without AI on purpose**, like a pilot flying by hand: an exercise, a small project, an evening of plain coding.
- **Type, don't paste, while learning.** Typing it yourself makes you read it.
- **Explain the AI's code back to yourself.** If you can't, you haven't learned it yet. Ask until you can.
- **Check the AI with the basics:** run the code, calculate one example by hand, read the documentation.

## If people stop thinking, is the future dark?

### AI isn't a calculator

People sometimes compare AI to the calculator or the GPS: "people feared those too, and it turned out fine". That comparison is misleading.
- A **calculator** does arithmetic.
- A **GPS** finds a route.

Each is a tool for **one narrow, mechanical job**, and it does that job exactly. Neither thinks, and neither can replace a person's thinking. The human still decides what to calculate, where to go, and what the answer means.

AI is different in kind, not just in size. It produces **what thinking produces**: explanations, arguments, plans, decisions and code, in fluent language that looks like a person's thoughts. Whether it really *thinks* is debated. What matters here is that it can **stand in for your own thinking**: you can hand it the deciding, the reasoning and the judging, and still get something that looks finished.

That's why this worry is new and serious, and why it shouldn't be waved away as "just another tool". A calculator can't make you stop thinking. An AI can, if you let it.

### That's why people must not stop learning

Because AI can stand in for thinking, the skills **of** thinking must be practised on purpose. Nobody should stop learning these three:

- **Mathematics.** Maths trains exact, step-by-step reasoning: what follows from what, what's proven and what's only assumed. It's also how you notice that a number is wrong, whether the AI calculated it or not.
- **Writing.** Writing is thinking made visible. To write something clearly, you have to understand it, put it in order and decide what you mean. If an AI always writes for you, you lose more than a skill: you lose the moment where your own thoughts take shape.
- **Critical thinking.** Asking: is this true? How do I know? What's the evidence? What's missing? Who benefits if I believe it? AI answers sound sure of themselves whether they're right or wrong. Critical thinking is what lets you tell the difference, in code, in the news and in everyday life.

These aren't old-fashioned school subjects that AI made unnecessary. They're exactly what keeps a person in charge of the tool.

### It's a choice, not a fate

The same AI that lets one person avoid thinking can help another think much further, as long as **they** do the thinking. A curious person can ask "why?" ten times in a row at midnight and get patient answers every time. The tool supports both paths. Which one people take depends on:
- their habits;
- their education;
- whether anyone tells them that the difference matters.

That's why this lesson exists, and why it ends with hope rather than gloom. People who keep thinking won't become less valuable; they'll become **more** valuable. They'll be the ones who can tell when the confident machine is wrong.

### The future we'd like to see

People and AI each doing what they do best, with people firmly in charge of the judgement. This course is a small example of it:
- The AI wrote much of the text and code.
- The human kept stopping it when it went the wrong way: no unrequested scripts, real machine code instead of an interpreter, "use the LLVM that's installed".
- Every one of those corrections made the result better than either would have made alone.

That future needs people who think.

## Not conservative: realistic

It might sound old-fashioned to say "learn the basics" when AI can do so much. But it's the opposite of nostalgia. The more powerful the tool, the more it matters that the person using it understands what it does. Nobody wants a pilot who can only use the autopilot, or a doctor who can only read what the machine says.

## The course in one paragraph

AI makes writing code fast and cheap. It doesn't make **knowing what the code should do**, **checking that it does it** and **being responsible for it** any less necessary. If anything, they matter more, because there's so much more code.
- Vibe-code what you'll throw away.
- Engineer what people depend on.
- Learn the craft, so you can ask for the right thing and recognise the wrong one.
- Never stop learning maths, writing and critical thinking: they keep you in charge of the tool.
- Let Rust's compiler, clippy and tests check every change, from the first day.
- Add dependencies on purpose, never by default.

The tools in this course will change, maybe soon. These habits won't.

## What may change

Almost everything about AI tools may change, and studies done with today's tools may not describe tomorrow's. But people have relied on tools for thousands of years, and one pattern has held every time: **the tool is only as good as the judgement of the person using it.**

Previous: [Lesson 9: Reviewing AI-written code](../09-reviewing-ai-code/) · Back to the [course overview](../)
