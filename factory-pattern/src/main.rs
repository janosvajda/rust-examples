trait Robot {
    fn name(&self) -> &str;
    fn move_hands(&self);
    fn walk(&self);
    fn speak(&self);
}

struct RobotA;

impl Robot for RobotA {
    fn name(&self) -> &str {
        "Dumbly"
    }

    fn move_hands(&self) {
        println!("{} is moving hands", self.name());
    }

    fn walk(&self) {
        // Robot A cannot walk, so this method does nothing
    }

    fn speak(&self) {
        // Robot A cannot speak, so this method does nothing
    }
}

struct RobotB;

impl Robot for RobotB {
    fn name(&self) -> &str {
        "Pumbly"
    }

    fn move_hands(&self) {
        println!("{} is moving hands", self.name());
    }

    fn walk(&self) {
        println!("{} is walking", self.name());
    }

    fn speak(&self) {
        // Robot B cannot speak, so this method does nothing
    }
}

struct RobotC;

impl Robot for RobotC {
    fn name(&self) -> &str {
        "Wumbly"
    }

    fn move_hands(&self) {
        println!("{} is moving hands", self.name());
    }

    fn walk(&self) {
        println!("{} is walking", self.name());
    }

    fn speak(&self) {
        println!("{} is speaking", self.name());
    }
}

enum RobotType {
    RobotA,
    RobotB,
    RobotC,
}

struct RobotFactory;

impl RobotFactory {
    fn create_robot(&self, robot_type: RobotType) -> Box<dyn Robot> {
        match robot_type {
            RobotType::RobotA => Box::new(RobotA),
            RobotType::RobotB => Box::new(RobotB),
            RobotType::RobotC => Box::new(RobotC),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_robot_a() {
        let factory = RobotFactory;
        let robot = factory.create_robot(RobotType::RobotA);
        robot.move_hands();
        robot.walk();
        robot.speak();
        assert_eq!(robot.name(), "Dumbly");
    }

    #[test]
    fn test_create_robot_b() {
        let factory = RobotFactory;
        let robot = factory.create_robot(RobotType::RobotB);
        robot.move_hands();
        robot.walk();
        robot.speak();
        assert_eq!(robot.name(), "Pumbly");
    }

    #[test]
    fn test_create_robot_c() {
        let factory = RobotFactory;
        let robot = factory.create_robot(RobotType::RobotC);
        robot.move_hands();
        robot.walk();
        robot.speak();
        assert_eq!(robot.name(), "Wumbly");
    }
}

fn main() {
    let factory = RobotFactory;
    let robot_a = factory.create_robot(RobotType::RobotA);
    let robot_b = factory.create_robot(RobotType::RobotB);
    let robot_c = factory.create_robot(RobotType::RobotC);

    robot_a.move_hands();
    robot_a.walk();
    robot_a.speak();
    println!("Robot name: {}", robot_a.name());

    robot_b.move_hands();
    robot_b.walk();
    robot_b.speak();
    println!("Robot name: {}", robot_b.name());

    robot_c.move_hands();
    robot_c.walk();
    robot_c.speak();
    println!("Robot name: {}", robot_c.name());
}
