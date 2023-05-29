use std::fmt;
use std::io;

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
}

fn main() {
    let mut board = Board::new();
    let mut current_player = Player::X;

    loop {
        board.display();

        println!("Player {}, enter your move (0-8):", current_player);

        let mut input = String::new();
        io::stdin()
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

            current_player = match current_player {
                Player::X => Player::O,
                Player::O => Player::X,
            };
        } else {
            println!("Invalid input. Please enter a number from 0 to 8.");
            continue;
        }
    }
}
