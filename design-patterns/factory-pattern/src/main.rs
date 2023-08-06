mod robot;
mod robot_factory;

use robot_factory::RobotFactory;

fn main() {
    run();
}

pub fn run() {
    let factory = RobotFactory::new();

    let robot_a = factory.create_robot_a();
    let robot_b = factory.create_robot_b();
    let robot_c = factory.create_robot_c();

    println!("Robot A: {:?}", robot_a.get_name());
    println!("Robot B: {:?}", robot_b.get_name());
    println!("Robot C: {:?}", robot_c.get_name());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run() {
        let factory = RobotFactory::new();
        let robot_a = factory.create_robot_a();
        let robot_b = factory.create_robot_b();
        let robot_c = factory.create_robot_c();

        assert_eq!(robot_a.get_name(), "Dumbly");
        assert_eq!(robot_b.get_name(), "Pumbly");
        assert_eq!(robot_c.get_name(), "Wumbly");
    }
}
