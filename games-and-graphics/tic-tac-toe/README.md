<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Tic-tac-toe

Play tic-tac-toe in the terminal against the computer, at three levels: **easy**, **normal**, and **hard**, which can't be beaten.

```text
Tic-tac-toe: you are X, the computer is O. Ctrl+D quits at any time.
Difficulty: 1 easy, 2 normal, 3 hard (unbeatable): 3
 O │ 2 │ O
───┼───┼───
 4 │ X │ 6
───┼───┼───
 7 │ 8 │ X
The computer played 3.
Your move (X), 1–9:
```

Free cells show the number to type.

## Run it

```bash
cargo run
cargo test
```

## How the computer plays

| Level | Strategy |
|---|---|
| easy | any free cell |
| normal | win if it can, block you if it must, otherwise any free cell |
| hard | **minimax**: it looks at every possible future of the game |

### Minimax: thinking every move ahead

For each free cell, the hard computer imagines playing there. Then it imagines every reply you could make, every answer to that, and so on, all the way to the end of every possible game. Each finished game gets a score:

| End of the game | Score for the computer |
|---|---|
| the computer wins | **+10 − moves taken**: winning sooner is better |
| a draw | **0** |
| you win | **moves taken − 10**: losing later is less bad |

Then it works backwards. On **its** turn, it assumes it will choose the highest score. On **your** turn, it assumes you'll choose the lowest one, the best move for you. The move whose score survives this back-and-forth is the one it plays.

```rust
fn minimax(board: &Board, to_move: Player, me: Player, depth: i32) -> i32 {
    match board.outcome() {
        Some(Outcome::Win(p)) if p == me => return 10 - depth,
        Some(Outcome::Win(_)) => return depth - 10,
        Some(Outcome::Draw) => return 0,
        None => {}
    }
    let scores = board.free_cells().map(|cell| {
        let mut after = *board;
        after.cells[cell] = Some(to_move);
        minimax(&after, to_move.other(), me, depth + 1)    // the same question, one move later
    });
    if to_move == me { scores.max() } else { scores.min() }.unwrap_or(0)
}
```

It's a **recursive** function: to score a board, it scores the boards one move later. Tic-tac-toe is small enough to search completely, at most 9 × 8 × 7 × … moves. Chess isn't: there, programs search only a few moves deep and estimate the rest.

The "normal" level is the strategy the original version of this game used. It's easy to beat with a **fork**: two threats at once, where blocking one lets the other win. Minimax sees forks coming, and a test proves it.

### Proving it can't be beaten

`hard_can_never_be_beaten` doesn't try a few games. It plays **every possible game**: every move you could make, at every point, against the hard computer's reply, and checks that you never win. Tic-tac-toe is small enough to make that kind of complete proof cheap.

## How the code is organised

| File | Contains | Tested how |
|---|---|---|
| `game.rs` | the board, the rules, the three levels of computer player; no input, output or randomness | exact tests of wins, draws, illegal moves, forks, and every possible game |
| `main.rs` | asking for moves and drawing the board | whole games played with typed text |

`main.rs` is written for **any** `BufRead` (input) and `Write` (output), not directly for the keyboard and screen. The real program passes `stdin` and `stdout`. The tests pass text like `"5\n3\n7\n"` and collect what was printed. That's how a terminal game gets tested without anyone typing.

The computer's randomness is also passed in, as a function `pick(n)` that returns a number below `n`. The real game uses a random one. The tests use `|_| 0`, so every game is repeatable. The random numbers come from the standard library's `RandomState`, whose hash keys are random for every run of the program: enough for a game, with no `rand` crate needed.

## What was fixed

This game was first written in 2023. While modernising it, these problems turned up:

| Problem | Fix |
|---|---|
| **Ctrl+D made it loop forever.** At the end of input, `read_line` returns 0 bytes, the empty answer was "invalid", and the loop asked again, endlessly | the end of input now ends the game cleanly; a test checks it |
| **Error messages were never seen**: the screen was cleared right after printing them | the board is redrawn first, then the message |
| the screen was cleared by starting the `clear` program (or `cls`), a new process every turn | a terminal escape code, `\x1b[2J`, written directly |
| colour codes and screen-clearing were also written into pipes and files | used only when the output is a real terminal (`IsTerminal`) |
| the computer was easy to beat with a fork | three levels, with an unbeatable one |
| the game logic was mixed into `main`, so only parts could be tested | rules in `game.rs`; whole games tested |
| the `rand` crate, for one random number | the standard library's `RandomState` |

Back to [Games and graphics](../)
