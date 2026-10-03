use std::fmt;

pub trait Robot: fmt::Debug {
    fn do_work(&self);
    fn get_name(&self) -> &'static str;
}

#[derive(Debug)]
pub struct RobotA {
    name: &'static str,
}

impl RobotA {
    pub fn new() -> Self {
        RobotA { name: "Dumbly" }
    }
}

impl Robot for RobotA {
    fn do_work(&self) {
        println!("{} is sitting and moving hands", self.name);
    }

    fn get_name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug)]
pub struct RobotB {
    name: &'static str,
}

impl RobotB {
    pub fn new() -> Self {
        RobotB { name: "Pumbly" }
    }
}

impl Robot for RobotB {
    fn do_work(&self) {
        println!("{} is moving hands and walking", self.name);
    }

    fn get_name(&self) -> &'static str {
        self.name
    }
}

#[derive(Debug)]
pub struct RobotC {
    name: &'static str,
}

impl RobotC {
    pub fn new() -> Self {
        RobotC { name: "Wumbly" }
    }
}

impl Robot for RobotC {
    fn do_work(&self) {
        println!("{} is moving hands, walking, and speaking", self.name);
    }

    fn get_name(&self) -> &'static str {
        self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_robot_a() {
        let robot = RobotA::new();
        assert_eq!(robot.get_name(), "Dumbly");
    }

    #[test]
    fn test_robot_b() {
        let robot = RobotB::new();
        assert_eq!(robot.get_name(), "Pumbly");
    }

    #[test]
    fn test_robot_c() {
        let robot = RobotC::new();
        assert_eq!(robot.get_name(), "Wumbly");
    }
}
