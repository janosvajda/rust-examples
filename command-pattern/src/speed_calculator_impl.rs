pub trait SpeedCalculator {
    fn calculate_speed(&self, time_per_kilometer: f64);
}

pub struct SpeedCalculatorImpl;

impl SpeedCalculator for SpeedCalculatorImpl {
    fn calculate_speed(&self, time_per_kilometer: f64) {
        let speed = 60.0 / time_per_kilometer; // Calculate speed in km/h
        println!("Speed: {:.2} km/h", speed);
    }
}
