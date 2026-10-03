use crate::robot::{Robot, RobotA, RobotB, RobotC};

pub struct RobotFactory {}

impl RobotFactory {
    pub fn new() -> Self {
        RobotFactory {}
    }

    pub fn create_robot_a(&self) -> Box<dyn Robot> {
        Box::new(RobotA::new())
    }

    pub fn create_robot_b(&self) -> Box<dyn Robot> {
        Box::new(RobotB::new())
    }

    pub fn create_robot_c(&self) -> Box<dyn Robot> {
        Box::new(RobotC::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_factory() {
        let factory = RobotFactory::new();
        let robot_a = factory.create_robot_a();
        let robot_b = factory.create_robot_b();
        let robot_c = factory.create_robot_c();

        assert_eq!(robot_a.get_name(), "Dumbly");
        assert_eq!(robot_b.get_name(), "Pumbly");
        assert_eq!(robot_c.get_name(), "Wumbly");
    }
}
