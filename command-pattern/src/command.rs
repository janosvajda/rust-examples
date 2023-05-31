use crate::speed_calculator_impl::SpeedCalculator;

pub trait Command {
    fn execute(&self);
}

pub struct CalculateSpeedCommand {
    pub receiver: Box<dyn SpeedCalculator>,
    pub time_per_kilometer: f64,
}

impl Command for CalculateSpeedCommand {
    fn execute(&self) {
        self.receiver.calculate_speed(self.time_per_kilometer);
    }
}
