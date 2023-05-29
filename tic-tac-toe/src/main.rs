use std::fmt;
use rand::Rng;

#[derive(Debug, PartialEq, Copy, Clone)]
enum Player {
    X,
    O,
}

// Implement the Display trait for Player enum to customize its string representation
impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

struct Board {
    cells: [Option<Player>; 9],
}

impl Board {
    // Create a new instance of the board with empty cells
    fn new() -> Self {
        Board { cells: [None; 9] }
    }

    // Make a move on the board at the specified position with the given player
    fn make_move(&mut self, position: usize, player: Player) {
        self.cells[position] = Some(player);
    }

    // Display the current state of the board
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

    // Check if the specified player has won the game
    fn check_win(&self, player: Player) -> bool {
        let win_conditions = [
            [0, 1, 2], [3, 4, 5], [6, 7, 8], // Rows
            [0, 3, 6], [1, 4, 7], [2, 5, 8], // Columns
            [0, 4, 8], [2, 4, 6], // Diagonals
        ];

        win_conditions.iter().any(|&[a, b, c]| {
            self.cells[a] == Some(player) && self.cells[b] == Some(player) && self.cells[c] == Some(player)
        })
    }

    // Get a vector of available moves (empty positions) on the board
    fn get_available_moves(&self) -> Vec<usize> {
        self.cells.iter().enumerate()
            .filter_map(|(index, &cell)| if cell.is_none() { Some(index) } else { None })
            .collect()
    }

    // Check if the board is full
    fn is_full(&self) -> bool {
        self.cells.iter().all(|&cell| cell.is_some())
    }
}

fn main() {
    let mut board = Board::new();
    let mut current_player = Player::X;

    loop {
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
                }

                board.make_move(position, current_player);

                if board.check_win(current_player) {
                    println!("Player {} wins!", current_player);
                    break;
                }

                if board.is_full() {
                    println!("It's a tie!");
                    break;
                }

                current_player = Player::O;
            } else {
                println!("Invalid input. Please enter a number from 0 to 8.");
                continue;
            }
        } else {
            println!("Computer's turn (Player {}).", current_player);
            let available_moves = board.get_available_moves();
            let random_index = rand::thread_rng().gen_range(0..available_moves.len());
            let computer_move = available_moves[random_index];
            board.make_move(computer_move, current_player);

            if board.check_win(current_player) {
                println!("Player {} wins!", current_player);
                break;
            }

            if board.is_full() {
                println!("It's a tie!");
                break;
            }

            current_player = Player::X;
        }
    }
}
