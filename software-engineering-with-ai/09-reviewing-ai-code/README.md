<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Lesson 9: Reviewing AI-written code

> Written in October 2026. AI tools change every month. Check whether what you read here still holds.

## The idea in one sentence

Review AI-written code as if a fast, well-read, very confident new colleague wrote it: the code is often good, it's sometimes wrong in ways that look right, and it's always your responsibility once you accept it.

## Before reading the code

- **Did it do what was asked, and only that?** AIs like to "help" beyond the request: an extra script, a refactored file nobody asked for, a renamed function. Each unrequested change is code you now have to review and maintain. In this repository, a request for documentation came back with shell scripts and benchmarks that weren't wanted. Scope is the first thing to check.
- **Do the automatic checks pass?** `cargo build`, `cargo clippy --all-targets -- -D warnings`, `cargo test`. Don't start reading before the machines are happy.
- **Did it really run what it says it ran?** "All tests pass" is a claim. Run them yourself, or look at the actual output.

## The checklist

### Correctness

- [ ] Does it do what the **specification** says, not just what the example shows?
- [ ] Empty input, one item, huge input, invalid input: what happens? (Lesson 4)
- [ ] Ties, duplicates, negative numbers, zero: did anyone decide what should happen?
- [ ] Are new tests written from the specification, or copied from the output? (Lesson 6)
- [ ] Were existing tests changed? If yes, **why**?

### Errors

- [ ] Any `.unwrap()` or `.expect()` on input that can really fail?
- [ ] Any `let _ = …` or `.ok()` that silently throws an error away?
- [ ] Do error messages say what went wrong, so someone can fix it?

### Rust-specific warning signs

| You see | Ask |
|---|---|
| `.clone()` in many places | was it needed, or did it just silence the borrow checker? |
| `Arc<Mutex<…>>` around everything | is the data really shared between threads? |
| `'static` added to fix a lifetime error | does the value really live forever, or was the real problem elsewhere? |
| `unsafe` | why is it needed, and is the safety argument written down? |
| `as` casts between number types | can the value be too big or negative for the target type? |
| `#[allow(…)]` | why is the warning wrong here? Is the reason written next to it? |

### Performance

- [ ] Loops inside loops over the same data: is it O(n²) where O(n) is possible?
- [ ] Work repeated inside a loop that could be done once outside?
- [ ] Any claim like "this is faster": was it **measured**?

### Security and dependencies

- [ ] Does any user input reach a file path, a shell command or an SQL query without checks?
- [ ] Are there secrets, passwords or keys in the code?
- [ ] **Does every new dependency really exist, and is it the right one?** AIs sometimes suggest crates that don't exist, or that have a name close to a popular one. Attackers register those names. Check on crates.io: does it exist, who maintains it, how many people use it?
- [ ] Is a new dependency needed at all? Fewer dependencies mean less code you have to trust ([Lesson 8](../08-fewer-dependencies/)).

### Readability

- [ ] Does it match the style of the code around it?
- [ ] Are the comments true? AIs write convincing comments that sometimes describe what the code *should* do instead of what it does.
- [ ] Could you explain every part to a colleague? If not, ask the AI to explain it, then check the explanation against the code.

## When you find a problem

- **Fix the cause, not the symptom.** "Make the error go away" leads to the clone, the unwrap and the deleted test.
- **Say what's wrong and why**, the way you'd tell a colleague. The AI's next attempt is only as good as your feedback.
- **Add a test** that would have caught the problem, so it can't come back.
- **Write a lasting rule** if the same mistake keeps happening. Put it in the project instruction file, or add a clippy lint.

Previous: [Lesson 8: Fewer dependencies](../08-fewer-dependencies/) · Next: [Lesson 10: The shadow side, and why you should still learn](../10-the-shadow-side/)
