// Define the Mars Rover
struct MarsRover {
    x: i32,
    y: i32,
    direction: Direction,
}

#[derive(Debug, PartialEq)] // Derive PartialEq trait to enable equality checks
enum Direction {
    North,
    East,
    South,
    West,
}

impl MarsRover {
    // Constructor to create a new Mars Rover
    fn new(x: i32, y: i32, direction: Direction) -> Self {
        MarsRover { x, y, direction }
    }

    // Execute a command on the rover
    fn execute_command(&mut self, command: Box<dyn Command>) {
        command.execute(self);
    }

    // Show the current position and facing direction of the rover
    fn show_position(&self) {
        println!("Current position: ({}, {}), Facing: {:?}", self.x, self.y, self.direction);
    }

    // Show the movement of the rover in a 5x5 grid matrix
    fn show_movement_in_matrix(&self) {
        let mut grid = [['.'; 5]; 5];
        grid[self.y as usize][self.x as usize] = 'R';

        for row in grid.iter() {
            for cell in row.iter() {
                print!("{} ", cell);
            }
            println!();
        }
    }

    // Wrap around the grid if the rover goes beyond the boundaries
    fn wrap_around(&mut self) {
        if self.x < 0 {
            self.x = 4;
        } else if self.x > 4 {
            self.x = 0;
        }

        if self.y < 0 {
            self.y = 4;
        } else if self.y > 4 {
            self.y = 0;
        }
    }
}


// Define the Command trait
trait Command {
    fn execute(&self, rover: &mut MarsRover);
}

// Concrete MoveForwardCommand
struct MoveForwardCommand;

impl Command for MoveForwardCommand {
    fn execute(&self, rover: &mut MarsRover) {
        // Move the rover forward based on its current direction
        match rover.direction {
            Direction::North => rover.y += 1,
            Direction::East => rover.x += 1,
            Direction::South => rover.y -= 1,
            Direction::West => rover.x -= 1,
        }
        // Wrap around the grid if the rover goes beyond the boundaries
        rover.wrap_around();
    }
}

// Concrete TurnLeftCommand
struct TurnLeftCommand;

impl Command for TurnLeftCommand {
    fn execute(&self, rover: &mut MarsRover) {
        // Turn the rover left based on its current direction
        rover.direction = match rover.direction {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        };
    }
}

// Concrete TurnRightCommand
struct TurnRightCommand;

impl Command for TurnRightCommand {
    fn execute(&self, rover: &mut MarsRover) {
        // Turn the rover right based on its current direction
        rover.direction = match rover.direction {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        };
    }
}

fn main() {
    // Create a new Mars Rover
    let mut rover = MarsRover::new(0, 0, Direction::North);

    // Define a list of commands for the rover to execute
    let commands: Vec<Box<dyn Command>> = vec![
        Box::new(MoveForwardCommand),
        Box::new(TurnRightCommand),
        Box::new(MoveForwardCommand),
        Box::new(MoveForwardCommand),
        Box::new(TurnLeftCommand),
        Box::new(MoveForwardCommand),
    ];

    // Execute each command and display the rover's position and movement in a matrix
    for command in commands {
        rover.execute_command(command);
        rover.show_position();
        rover.show_movement_in_matrix();
        println!();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_forward_command() {
        let mut rover = MarsRover::new(0, 0, Direction::North);
        let move_forward = MoveForwardCommand;

        move_forward.execute(&mut rover);

        assert_eq!(rover.x, 0);
        assert_eq!(rover.y, 1);
        assert_eq!(rover.direction, Direction::North);
    }

    #[test]
    fn test_turn_left_command() {
        let mut rover = MarsRover::new(0, 0, Direction::North);
        let turn_left = TurnLeftCommand;

        turn_left.execute(&mut rover);

        assert_eq!(rover.direction, Direction::West);
    }

    #[test]
    fn test_turn_right_command() {
        let mut rover = MarsRover::new(0, 0, Direction::North);
        let turn_right = TurnRightCommand;

        turn_right.execute(&mut rover);

        assert_eq!(rover.direction, Direction::East);
    }

    #[test]
    fn test_move_forward_wrap_around() {
        let mut rover = MarsRover::new(0, 0, Direction::North);
        let move_forward = MoveForwardCommand;

        // Move the rover forward to the top boundary
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);

        assert_eq!(rover.x, 0);
        assert_eq!(rover.y, 0);

        // Move the rover forward to the bottom boundary
        rover.direction = Direction::South;
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);
        move_forward.execute(&mut rover);

        assert_eq!(rover.x, 0);
        assert_eq!(rover.y, 0);
    }

    #[test]
    fn test_turn_left_and_wrap_around() {
        let mut rover = MarsRover::new(0, 0, Direction::North);
        let turn_left = TurnLeftCommand;
        let turn_right = TurnRightCommand;

        // Turn left four times to face West
        turn_left.execute(&mut rover);
        turn_left.execute(&mut rover);
        turn_left.execute(&mut rover);
        turn_left.execute(&mut rover);

        // The rover should now wrap around to face East (opposite direction from West)
        turn_right.execute(&mut rover);

        assert_eq!(rover.direction, Direction::East);
    }

}
