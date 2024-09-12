use std::io;

fn main() {
    // Welcome message
    println!("Welcome to the Calorie Calculator!");

    // Gather user input
    let (weight, height, age, activity_level) = get_user_input();

    // Calculate BMR and TDEE
    let (bmr, tdee) = calculate_calories(weight, height, age, activity_level);

    // Display results
    println!("Your Basal Metabolic Rate (BMR) is: {:.2} calories", bmr);
    println!("Your Total Daily Energy Expenditure (TDEE) is: {:.2} calories", tdee);
}

fn get_user_input() -> (f64, f64, u32, f64) {
    // Prompt for weight
    println!("Please enter your weight in kilograms:");
    let weight = read_f64_input();

    // Prompt for height
    println!("Please enter your height in centimeters:");
    let height = read_f64_input();

    // Prompt for age
    println!("Please enter your age:");
    let age = read_u32_input();

    // Prompt for activity level
    println!("Select your activity level:");
    println!("1. Sedentary (little to no exercise)");
    println!("2. Lightly active (light exercise/sports 1-3 days a week)");
    println!("3. Moderately active (moderate exercise/sports 3-5 days a week)");
    println!("4. Very active (hard exercise/sports 6-7 days a week)");
    println!("5. Super active (very hard exercise/sports, physical job, or training twice a day)");
    let activity_level = read_f64_input();

    (weight, height, age, activity_level)
}

fn calculate_calories(weight: f64, height: f64, age: u32, activity_level: f64) -> (f64, f64) {
    // Calculate BMR using the Mifflin-St Jeor Equation
    let bmr = 10.0 * weight + 6.25 * height - 5.0 * f64::from(age) + 5.0;

    // Apply activity level to calculate TDEE
    let tdee = bmr * activity_level;

    (bmr, tdee)
}

fn read_f64_input() -> f64 {
    // Read a f64 input from the user
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().parse().expect("Invalid input")
}

fn read_u32_input() -> u32 {
    // Read a u32 input from the user
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().parse().expect("Invalid input")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_calories() {
        // Test case with moderate values
        let weight = 70.0;
        let height = 175.0;
        let age = 30;
        let activity_level = 1.5;

        let (bmr, tdee) = calculate_calories(weight, height, age, activity_level);

        // Calculate expected values based on the formula
        let expected_bmr = 10.0 * weight + 6.25 * height - 5.0 * f64::from(age) + 5.0;
        let expected_tdee = expected_bmr * activity_level;

        // Compare calculated values to expected values with a small delta
        assert!((bmr - expected_bmr).abs() < 0.01);
        assert!((tdee - expected_tdee).abs() < 0.01);
    }

    #[test]
    fn test_lower_weight() {
        // Test case with lower weight and moderate activity level
        let weight = 55.0;
        let height = 160.0;
        let age = 25;
        let activity_level = 1.3;

        let (bmr, tdee) = calculate_calories(weight, height, age, activity_level);

        // Calculate expected values based on the formula
        let expected_bmr = 10.0 * weight + 6.25 * height - 5.0 * f64::from(age) + 5.0;
        let expected_tdee = expected_bmr * activity_level;

        // Compare calculated values to expected values with a small delta
        assert!((bmr - expected_bmr).abs() < 0.01);
        assert!((tdee - expected_tdee).abs() < 0.01);
    }

    #[test]
    fn test_high_activity_level() {
        // Test case with higher weight and very high activity level
        let weight = 80.0;
        let height = 180.0;
        let age = 35;
        let activity_level = 2.0;

        let (bmr, tdee) = calculate_calories(weight, height, age, activity_level);

        // Calculate expected values based on the formula
        let expected_bmr = 10.0 * weight + 6.25 * height - 5.0 * f64::from(age) + 5.0;
        let expected_tdee = expected_bmr * activity_level;

        // Compare calculated values to expected values with a small delta
        assert!((bmr - expected_bmr).abs() < 0.01);
        assert!((tdee - expected_tdee).abs() < 0.01);
    }
}
