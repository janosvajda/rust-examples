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
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
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
            println!("Player {}, enter your move (0-8):", current_player);

            let mut input = String::new();
            std::io::stdin()
                .read_line(&mut input)
                .expect("Failed to read input.");

            let position: usize = match input.trim().parse() {
                Ok(num) => num,
                Err(_) => {
                    println!("Invalid input. Please enter a number from 0 to 8.");
                    continue;
                }
            };

            if position < 9 {
                if let Some(_) = board.cells[position] {
                    println!("Invalid move. The position is already occupied.");
                    continue;
                } else {
                    board.make_move(position, current_player);

                    if board.check_win(current_player) {
                        clear_console();
                        board.display();
                        println!("Player {} wins!", current_player);
                        break;
                    }

                    current_player = Player::O;
                }
            } else {
                println!("Invalid input. Please enter a number from 0 to 8.");
                continue;
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