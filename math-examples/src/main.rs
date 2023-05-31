mod speed_calculator;

use speed_calculator::calculate_speed;

fn main() {
    // Example time per kilometer in minutes
    let time_per_km = 5.25;

    // Calculate speed in km per hour
    let speed = calculate_speed(time_per_km);

    // Print the result
    println!("Your speed is {:.2} km/h", speed);
}
