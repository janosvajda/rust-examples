use std::fmt;
use std::process::Command;
use rand::Rng;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Player {
    X,
    O,
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::X => write!(f, "\x1b[36mX\x1b[0m"), // Cyan
            Player::O => write!(f, "O"), // Default color
        }
    }
}

#[derive(Clone)]
struct Board {
    cells: [Option<Player>; 9],
}

impl Board {
    fn new() -> Self {
        Board { cells: [None; 9] }
    }

    fn make_move(&mut self, position: usize, player: Player) {
        self.cells[position] = Some(player);
    }

    fn display(&self) {
        println!("  1   2   3"); // Horizontal indices
        println!("-------------");

        for (i, cell) in self.cells.iter().enumerate() {

            if let Some(player) = cell {
                print!("| {} ", player);
            } else {
                print!("|   ");
            }

            if i % 3 == 2 {
                println!("|");
                if i < 8 {
                    println!("-------------");
                }
            }
        }
    }


    fn check_win(&self, player: Player) -> bool {
        let win_conditions = [
            [0, 1, 2],
            [3, 4, 5],
            [6, 7, 8],
            [0, 3, 6],
            [1, 4, 7],
            [2, 5, 8],
            [0, 4, 8],
            [2, 4, 6],
        ];

        win_conditions.iter().any(|&[a, b, c]| {
            self.cells[a] == Some(player) && self.cells[b] == Some(player) && self.cells[c] == Some(player)
        })
    }

    fn get_available_moves(&self) -> Vec<usize> {
        self.cells
            .iter()
            .enumerate()
            .filter_map(|(index, &cell)| if cell.is_none() { Some(index) } else { None })
            .collect()
    }

    fn is_full(&self) -> bool {
        self.cells.iter().all(|&cell| cell.is_some())
    }

    fn predict_win(&self, player: Player) -> Option<usize> {
        for &move_ in self.get_available_moves().iter() {
            let mut temp_board = self.clone();
            temp_board.make_move(move_, player);
            if temp_board.check_win(player) {
                return Some(move_);
            }
        }
        None
    }
}

fn clear_console() {
    // This will clear the console by running a command in the shell
    // It's best to do this right before you need to redraw the game board
    let _ = Command::new("clear") // try 'clear' command first
        .status()
        .or_else(|_| Command::new("cls") // if 'clear' failed, try 'cls' command
            .status());
}

fn main() {
    let mut board = Board::new();
    let mut current_player = Player::X;

    while !board.is_full() {
        clear_console();
        board.display();

        if current_player == Player::X {
            println!("Player {}, enter your move (1-9):", current_player);

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input.");

            let position: usize = match input.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Invalid input. Please enter a number from 1 to 9.");
                    continue;
                }
            };

            if position < 1 || position > 9 {
                println!("Invalid input. Please enter a number from 1 to 9.");
                continue;
            }

            let index = position - 1;

            if let Some(_) = board.cells[index] {
                println!("Invalid move. The position is already occupied.");
                continue;
            } else {
                board.make_move(index, current_player);
                println!("\x07"); // Beep sound

                if board.check_win(current_player) {
                    clear_console();
                    board.display();
                    println!("Player {} wins!", current_player);
                    break;
                }

                current_player = Player::O;
            }
        } else {
            println!("Computer's turn (Player {}).", current_player);

            let computer_move = if let Some(win_move) = board.predict_win(Player::O) {
                win_move
            } else if let Some(block_move) = board.predict_win(Player::X) {
                block_move
            } else {
                let available_moves = board.get_available_moves();
                let random_index = rand::thread_rng().gen_range(0..available_moves.len());
                available_moves[random_index]
            };

            board.make_move(computer_move, current_player);

            if board.check_win(current_player) {
                clear_console();
                board.display();
                println!("Player {} wins!", current_player);
                break;
            }

            current_player = Player::X;
        }
    }

    // Display the final board state at the end of the game
    if !board.check_win(Player::X) && !board.check_win(Player::O) {
        clear_console();
        board.display();
        println!("It's a tie!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board_new() {
        let board = Board::new();
        assert_eq!(board.cells, [None; 9]);
    }

    #[test]
    fn test_board_make_move() {
        let mut board = Board::new();
        board.make_move(0, Player::X);
        board.make_move(4, Player::O);
        board.make_move(8, Player::X);

        assert_eq!(board.cells, [
            Some(Player::X), None, None,
            None, Some(Player::O), None,
            None, None, Some(Player::X),
        ]);
    }

    #[test]
    fn test_board_check_win() {
        let mut board = Board::new();
        board.make_move(0, Player::X);
        board.make_move(1, Player::X);
        board.make_move(2, Player::X);

        assert_eq!(board.check_win(Player::X), true);
        assert_eq!(board.check_win(Player::O), false);

        let mut board = Board::new();
        board.make_move(0, Player::O);
        board.make_move(3, Player::O);
        board.make_move(6, Player::O);

        assert_eq!(board.check_win(Player::X), false);
        assert_eq!(board.check_win(Player::O), true);
    }

    #[test]
    fn test_board_is_full() {
        let mut board = Board::new();
        assert_eq!(board.is_full(), false);

        for i in 0..9 {
            board.make_move(i, Player::X);
        }
        assert_eq!(board.is_full(), true);
    }
    #[test]
    fn test_board_predict_win() {
        let mut board = Board::new();
        board.make_move(0, Player::X);
        board.make_move(1, Player::X);
        assert_eq!(board.predict_win(Player::X), Some(2)); // Winning move is 2
    }
    
    #[test]
    fn test_board_predict_loosing() {
        // Test losing move
        let mut board = Board::new();
        board.make_move(0, Player::O);
        board.make_move(1, Player::O);
        assert_eq!(board.predict_win(Player::O), Some(2)); // Losing move is 2
    }
    
    #[test]
    fn test_board_predict_tie() {
        // Test tie scenario
        let mut board = Board::new();
        board.make_move(0, Player::X);
        board.make_move(1, Player::O);
        board.make_move(2, Player::X);
        board.make_move(3, Player::O);
        board.make_move(4, Player::O);
        board.make_move(5, Player::X);
        board.make_move(6, Player::O);
        board.make_move(7, Player::X);
        board.make_move(8, Player::O);
        assert_eq!(board.predict_win(Player::X), None); // Tie scenario, no winning move
    }
}
