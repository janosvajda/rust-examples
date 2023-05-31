pub fn calculate_speed(time_per_km: f64) -> f64 {
    // Convert time_per_km to hours
    let time_hours = time_per_km / 60.0;

    // Calculate speed in km per hour
    let speed = 1.0 / time_hours;

    speed
}
