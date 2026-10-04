// Tic-tac-toe in the terminal, against the computer.
//
// The rules and the computer player are in game.rs. This file only does input
// and output, and it's written for any `BufRead` and `Write`, not directly for
// the keyboard and screen, so the tests can play whole games with typed text.

mod game;

use game::{Board, Difficulty, Outcome, Player, computer_move};
use std::io::{self, BufRead, IsTerminal, Write};

/// Terminal styling: colours and clearing the screen. Only used when output
/// goes to a real terminal: in a file or a pipe, escape codes are just noise.
struct Style {
    terminal: bool,
}

impl Style {
    fn clear(&self) -> &'static str {
        if self.terminal { "\x1b[2J\x1b[H" } else { "" } // clear the screen, cursor to the top
    }

    fn mark(&self, player: Player) -> String {
        match (self.terminal, player) {
            (true, Player::X) => String::from("\x1b[1;36mX\x1b[0m"), // bold cyan
            (true, Player::O) => String::from("\x1b[1;33mO\x1b[0m"), // bold yellow
            (false, p) => p.to_string(),
        }
    }
}

/// The board, with the number to type shown in every free cell.
fn render(board: &Board, style: &Style) -> String {
    let mut out = String::new();
    for row in 0..3 {
        let cells: Vec<String> = (0..3)
            .map(|col| {
                let i = row * 3 + col;
                board.cell(i).map_or((i + 1).to_string(), |p| style.mark(p))
            })
            .collect();
        out += &format!(" {} │ {} │ {}\n", cells[0], cells[1], cells[2]);
        if row < 2 {
            out += "───┼───┼───\n";
        }
    }
    out
}

/// One line of input, or `None` when the input has ended (Ctrl+D, or a closed pipe).
fn read_line(input: &mut impl BufRead) -> io::Result<Option<String>> {
    let mut line = String::new();
    if input.read_line(&mut line)? == 0 {
        return Ok(None); // end of input: stop, instead of asking forever
    }
    Ok(Some(line.trim().to_string()))
}

fn ask_difficulty(input: &mut impl BufRead, out: &mut impl Write) -> io::Result<Option<Difficulty>> {
    loop {
        write!(out, "Difficulty: 1 easy, 2 normal, 3 hard (unbeatable): ")?;
        out.flush()?;
        let Some(answer) = read_line(input)? else { return Ok(None) };
        match answer.as_str() {
            "1" => return Ok(Some(Difficulty::Easy)),
            "2" => return Ok(Some(Difficulty::Normal)),
            "3" => return Ok(Some(Difficulty::Hard)),
            _ => writeln!(out, "Please type 1, 2 or 3.")?,
        }
    }
}

/// Plays one game: the human is X and goes first. Returns `None` if the input
/// ended in the middle of the game.
fn play_game(
    input: &mut impl BufRead,
    out: &mut impl Write,
    style: &Style,
    difficulty: Difficulty,
    pick: &mut impl FnMut(usize) -> usize,
) -> io::Result<Option<Outcome>> {
    let mut board = Board::default();
    let mut message = String::new();
    loop {
        // Redraw first, THEN show the message, so it isn't wiped away by the redraw.
        write!(out, "{}{}", style.clear(), render(&board, style))?;
        if let Some(outcome) = board.outcome() {
            return Ok(Some(outcome));
        }
        if !message.is_empty() {
            writeln!(out, "{message}")?;
        }
        write!(out, "Your move ({}), 1–9: ", style.mark(Player::X))?;
        out.flush()?;

        let Some(answer) = read_line(input)? else { return Ok(None) };
        let cell = match answer.parse::<usize>() {
            Ok(n @ 1..=9) => n - 1,
            _ => {
                message = format!("`{answer}` isn't a cell: type a number from 1 to 9.");
                continue;
            }
        };
        if board.play(cell, Player::X).is_err() {
            message = format!("Cell {} is already taken.", cell + 1);
            continue;
        }
        message.clear();
        if board.outcome().is_none() {
            let reply = computer_move(&board, Player::O, difficulty, pick);
            board.play(reply, Player::O).expect("the computer only picks free cells");
            message = format!("The computer played {}.", reply + 1);
        }
    }
}

/// A random number below `n`, without a crate: the standard library's hash
/// keys are random per process. Fine for a game, never for security.
fn random_below(n: usize) -> usize {
    use std::hash::{BuildHasher, RandomState};
    (RandomState::new().hash_one(n) % n as u64) as usize
}

fn main() -> io::Result<()> {
    let style = Style { terminal: io::stdout().is_terminal() };
    let mut input = io::stdin().lock();
    let mut out = io::stdout().lock();
    writeln!(out, "Tic-tac-toe: you are X, the computer is O. Ctrl+D quits at any time.")?;

    let Some(difficulty) = ask_difficulty(&mut input, &mut out)? else { return writeln!(out) };
    loop {
        match play_game(&mut input, &mut out, &style, difficulty, &mut random_below)? {
            Some(Outcome::Win(Player::X)) => writeln!(out, "You win!")?,
            Some(Outcome::Win(Player::O)) => writeln!(out, "The computer wins.")?,
            Some(Outcome::Draw) => writeln!(out, "It's a draw.")?,
            None => break,
        }
        write!(out, "Play again? (y/n): ")?;
        out.flush()?;
        if read_line(&mut input)?.is_none_or(|answer| !answer.eq_ignore_ascii_case("y")) {
            break;
        }
    }
    writeln!(out, "\nThanks for playing!")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plays one game from typed lines, and returns the outcome and everything printed.
    fn play(typed: &str, difficulty: Difficulty) -> (Option<Outcome>, String) {
        let mut out = Vec::new();
        let style = Style { terminal: false };
        let outcome = play_game(&mut typed.as_bytes(), &mut out, &style, difficulty, &mut |_| 0).unwrap();
        (outcome, String::from_utf8(out).unwrap())
    }

    #[test]
    fn a_complete_game_against_easy() {
        // With pick = 0, easy always takes the lowest free cell: 1, then 2.
        // X plays 5, 3, 7: the diagonal from top right to bottom left.
        let (outcome, printed) = play("5\n3\n7\n", Difficulty::Easy);
        assert_eq!(outcome, Some(Outcome::Win(Player::X)));
        assert!(printed.contains("The computer played 1."));
        assert!(printed.contains("The computer played 2."));
    }

    #[test]
    fn bad_input_is_explained_and_shown_after_the_redraw() {
        let (_, printed) = play("hello\n0\n10\n5\n5\n", Difficulty::Hard);
        assert!(printed.contains("`hello` isn't a cell"));
        assert!(printed.contains("`0` isn't a cell"));
        assert!(printed.contains("`10` isn't a cell"));
        assert!(printed.contains("Cell 5 is already taken."));
    }

    #[test]
    fn end_of_input_stops_instead_of_looping_forever() {
        // The old version looped forever here: no more input, "invalid", ask again…
        assert_eq!(play("", Difficulty::Hard).0, None);
        assert_eq!(play("5\n", Difficulty::Hard).0, None);
    }

    #[test]
    fn no_escape_codes_outside_a_terminal() {
        let (_, printed) = play("5\n", Difficulty::Hard);
        assert!(!printed.contains('\x1b'));
    }

    #[test]
    fn the_board_shows_the_numbers_to_type() {
        let style = Style { terminal: false };
        assert_eq!(render(&Board::default(), &style), " 1 │ 2 │ 3\n───┼───┼───\n 4 │ 5 │ 6\n───┼───┼───\n 7 │ 8 │ 9\n");
    }
}
