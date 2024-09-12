// mars_rover.rs

#[derive(Debug, PartialEq)] // Derive PartialEq and Debug traits
struct Rover {
    x: i32,
    y: i32,
    direction: Direction,
}

// Define the Direction enum
#[derive(Clone, Copy, Debug, PartialEq)]
enum Direction {
    North,
    East,
    South,
    West,
}

// Define a Command trait for the Rover commands
trait Command {
    fn execute(&self, rover: &mut Rover);
}

// Define MoveForward command
struct MoveForward;

impl Command for MoveForward {
    fn execute(&self, rover: &mut Rover) {
        match rover.direction {
            Direction::North => rover.y += 1,
            Direction::East => rover.x += 1,
            Direction::South => rover.y -= 1,
            Direction::West => rover.x -= 1,
        }
    }
}

// Define TurnLeft command
struct TurnLeft;

impl Command for TurnLeft {
    fn execute(&self, rover: &mut Rover) {
        match rover.direction {
            Direction::North => rover.direction = Direction::West,
            Direction::East => rover.direction = Direction::North,
            Direction::South => rover.direction = Direction::East,
            Direction::West => rover.direction = Direction::South,
        }
    }
}

// Define TurnRight command
struct TurnRight;

impl Command for TurnRight {
    fn execute(&self, rover: &mut Rover) {
        match rover.direction {
            Direction::North => rover.direction = Direction::East,
            Direction::East => rover.direction = Direction::South,
            Direction::South => rover.direction = Direction::West,
            Direction::West => rover.direction = Direction::North,
        }
    }
}

// Define the Invoker struct
struct Invoker {
    commands: Vec<Box<dyn Command>>,
}

impl Invoker {
    fn new() -> Self {
        Invoker {
            commands: Vec::new(),
        }
    }

    fn add_command(&mut self, command: Box<dyn Command>) {
        self.commands.push(command);
    }

    fn execute_commands(&mut self, rover: &mut Rover) {
        for command in &self.commands {
            command.execute(rover);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_pattern() {
        let mut rover = Rover {
            x: 0,
            y: 0,
            direction: Direction::North,
        };
        let mut invoker = Invoker::new();

        // Add commands to the invoker
        invoker.add_command(Box::new(MoveForward));
        invoker.add_command(Box::new(MoveForward));
        invoker.add_command(Box::new(TurnRight));
        invoker.add_command(Box::new(MoveForward));
        invoker.add_command(Box::new(MoveForward));
        invoker.add_command(Box::new(TurnLeft));
        invoker.add_command(Box::new(MoveForward));

        // Execute the commands
        invoker.execute_commands(&mut rover);

        assert_eq!(rover, Rover { x: 2, y: 2, direction: Direction::South });
    }
}

fn main() {
    let mut rover = Rover {
        x: 0,
        y: 0,
        direction: Direction::North,
    };
    let mut invoker = Invoker::new();

    // Add commands to the invoker
    invoker.add_command(Box::new(MoveForward));
    invoker.add_command(Box::new(MoveForward));
    invoker.add_command(Box::new(TurnRight));
    invoker.add_command(Box::new(MoveForward));
    invoker.add_command(Box::new(MoveForward));
    invoker.add_command(Box::new(TurnLeft));
    invoker.add_command(Box::new(MoveForward));

    // Execute the commands
    invoker.execute_commands(&mut rover);

    println!("Rover Position: ({}, {}), Direction: {:?}", rover.x, rover.y, rover.direction);
}
