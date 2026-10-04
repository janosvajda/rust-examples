//! The rules of the game and the computer player. No input, no output, no
//! randomness of its own, so every part of it can be tested exactly.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Player {
    pub fn other(self) -> Player {
        match self {
            Player::X => Player::O,
            Player::O => Player::X,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Player::X => "X",
            Player::O => "O",
        })
    }
}

/// How a finished game ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Win(Player),
    Draw,
}

/// Why a move isn't allowed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveError {
    OutOfRange,
    Occupied,
}

/// The 8 lines that win: 3 rows, 3 columns, 2 diagonals. Cells are numbered
/// 0–8, left to right, top to bottom.
const LINES: [[usize; 3]; 8] = [[0, 1, 2], [3, 4, 5], [6, 7, 8], [0, 3, 6], [1, 4, 7], [2, 5, 8], [0, 4, 8], [2, 4, 6]];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Board {
    cells: [Option<Player>; 9],
}

impl Board {
    pub fn cell(&self, index: usize) -> Option<Player> {
        self.cells[index]
    }

    /// Places a mark. Refuses moves outside the board or onto a taken cell,
    /// so the board can never get into an impossible state.
    pub fn play(&mut self, index: usize, player: Player) -> Result<(), MoveError> {
        match self.cells.get(index) {
            None => Err(MoveError::OutOfRange),
            Some(Some(_)) => Err(MoveError::Occupied),
            Some(None) => {
                self.cells[index] = Some(player);
                Ok(())
            }
        }
    }

    pub fn free_cells(&self) -> impl Iterator<Item = usize> + '_ {
        (0..9).filter(|&i| self.cells[i].is_none())
    }

    pub fn winner(&self) -> Option<Player> {
        LINES.iter().find_map(|&[a, b, c]| match self.cells[a] {
            Some(p) if self.cells[b] == Some(p) && self.cells[c] == Some(p) => Some(p),
            _ => None,
        })
    }

    /// `None` while the game is still going.
    pub fn outcome(&self) -> Option<Outcome> {
        match self.winner() {
            Some(player) => Some(Outcome::Win(player)),
            None if self.free_cells().next().is_none() => Some(Outcome::Draw),
            None => None,
        }
    }
}

// ---- The computer player ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    /// Picks any free cell.
    Easy,
    /// Wins when it can, blocks when it must, otherwise picks any free cell.
    Normal,
    /// Looks at every possible future (minimax): it can't be beaten.
    Hard,
}

/// Chooses the computer's move. `pick(n)` must return a number below `n`; it's
/// passed in so the game itself has no randomness, and tests can control it.
pub fn computer_move(board: &Board, me: Player, difficulty: Difficulty, pick: &mut impl FnMut(usize) -> usize) -> usize {
    let free: Vec<usize> = board.free_cells().collect();
    let any = |pick: &mut dyn FnMut(usize) -> usize| free[pick(free.len()) % free.len()];
    match difficulty {
        Difficulty::Easy => any(pick),
        Difficulty::Normal => winning_move(board, me).or_else(|| winning_move(board, me.other())).unwrap_or_else(|| any(pick)),
        Difficulty::Hard => best_move(board, me),
    }
}

/// A cell that would complete a line for `player` right now, if there is one.
fn winning_move(board: &Board, player: Player) -> Option<usize> {
    board.free_cells().find(|&cell| {
        let mut after = *board;
        after.cells[cell] = Some(player);
        after.winner() == Some(player)
    })
}

/// The move with the best minimax score. Ties go to the lowest cell number,
/// so the choice is always the same for the same board.
fn best_move(board: &Board, me: Player) -> usize {
    let mut best = (i32::MIN, 0);
    for cell in board.free_cells() {
        let mut after = *board;
        after.cells[cell] = Some(me);
        let score = minimax(&after, me.other(), me, 1);
        if score > best.0 {
            best = (score, cell);
        }
    }
    best.1
}

/// Minimax: how good is this board for `me`, if both sides play perfectly
/// from here? `to_move` plays next. Positive is good for `me`, negative bad.
///
/// It tries every move, and for each, every reply, all the way to the end of
/// the game. On my turn I take the best score; on the opponent's turn I assume
/// they take the worst one for me. Faster wins (smaller `depth`) score higher,
/// so the computer doesn't dawdle when it can win now.
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
        minimax(&after, to_move.other(), me, depth + 1)
    });
    if to_move == me { scores.max() } else { scores.min() }.unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(layout: &str) -> Board {
        let mut b = Board::default();
        for (i, c) in layout.chars().filter(|c| !c.is_whitespace()).enumerate() {
            b.cells[i] = match c {
                'X' => Some(Player::X),
                'O' => Some(Player::O),
                _ => None,
            };
        }
        b
    }

    #[test]
    fn wins_draws_and_unfinished_games() {
        assert_eq!(board("XXX ... ...").outcome(), Some(Outcome::Win(Player::X)));
        assert_eq!(board("O.. .O. ..O").outcome(), Some(Outcome::Win(Player::O)));
        assert_eq!(board("XOX XOO OXX").outcome(), Some(Outcome::Draw));
        assert_eq!(board("XO. ... ...").outcome(), None);
    }

    #[test]
    fn illegal_moves_are_refused() {
        let mut b = board("X.. ... ...");
        assert_eq!(b.play(0, Player::O), Err(MoveError::Occupied));
        assert_eq!(b.play(9, Player::O), Err(MoveError::OutOfRange));
        assert_eq!(b.play(4, Player::O), Ok(()));
    }

    #[test]
    fn normal_wins_first_then_blocks() {
        let mut never = |_| unreachable!("no random move needed");
        // O can win at 2, and X threatens 6: winning comes first
        assert_eq!(computer_move(&board("OO. X.. .X."), Player::O, Difficulty::Normal, &mut never), 2);
        // no win for O, so block X's row at 2
        assert_eq!(computer_move(&board("XX. .O. ..."), Player::O, Difficulty::Normal, &mut never), 2);
    }

    #[test]
    fn hard_sees_a_fork_coming() {
        // X has opposite corners. Taking a corner now loses to a fork;
        // the only safe move is an edge.
        let reply = computer_move(&board("X.. .O. ..X"), Player::O, Difficulty::Hard, &mut |_| 0);
        assert!([1, 3, 5, 7].contains(&reply), "chose {reply}");
    }

    #[test]
    fn easy_only_picks_free_cells() {
        let b = board("XOX OX. ...");
        for i in 0..20 {
            let cell = computer_move(&b, Player::O, Difficulty::Easy, &mut |n| i % n);
            assert!(b.cell(cell).is_none());
        }
    }

    /// Plays EVERY possible game against the hard computer: every move the
    /// human could make, at every point. The computer must never lose one.
    #[test]
    fn hard_can_never_be_beaten() {
        fn explore(board: Board, games: &mut u32) {
            for cell in board.free_cells() {
                let mut b = board;
                b.play(cell, Player::X).unwrap(); // the human's move
                match b.outcome() {
                    Some(Outcome::Win(Player::X)) => panic!("the human won: {b:?}"),
                    Some(_) => *games += 1,
                    None => {
                        let reply = computer_move(&b, Player::O, Difficulty::Hard, &mut |_| 0);
                        b.play(reply, Player::O).unwrap();
                        if b.outcome().is_some() {
                            *games += 1;
                        } else {
                            explore(b, games);
                        }
                    }
                }
            }
        }
        let mut games = 0;
        explore(Board::default(), &mut games);
        assert!(games > 100, "explored {games} complete games");
    }
}
