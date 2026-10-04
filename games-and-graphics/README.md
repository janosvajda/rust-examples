<img src="../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Games and graphics

Small, playable programs. Games are a great way to learn: you see the result immediately, they're fun to improve, and underneath they use the same ideas as serious software, like state, input, algorithms and testing.

These are **examples**, not a course: each one stands on its own.

| Example | What it is | What it teaches |
|---|---|---|
| [Tic-tac-toe](tic-tac-toe/) | the classic game in the terminal, with an unbeatable computer player | the **minimax** algorithm and recursion, enums for game state, and testing a game by playing it with typed text |
| [Bouncing face](bouncing-face/) | a face in a window that follows your mouse, with fireworks | how **graphics** work: a framebuffer of pixels, colours as numbers, drawing shapes, and the game loop |

Each README ends with a list of what was fixed while modernising it, from bugs that are easy to make: colours in the wrong format, an unsigned number going below zero, and a loop that never ends when the input does.

```bash
cd games-and-graphics/tic-tac-toe
cargo run
```

Want to build a bigger language-shaped project? See [Mini](../mini/), a tiny programming language that compiles to real machine code.
