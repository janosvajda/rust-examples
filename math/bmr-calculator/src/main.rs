use std::io;
use std::str::FromStr;

#[derive(Clone, Copy, Debug)]
enum Sex {
    Male,
    Female,
}

impl FromStr for Sex {
    type Err = &'static str;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "1" => Ok(Self::Male),
            "2" => Ok(Self::Female),
            _ => Err("choose 1 or 2"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum Activity {
    Sedentary,
    Light,
    Moderate,
    VeryActive,
    ExtraActive,
}

impl Activity {
    fn multiplier(self) -> f64 {
        match self {
            Self::Sedentary => 1.2,
            Self::Light => 1.375,
            Self::Moderate => 1.55,
            Self::VeryActive => 1.725,
            Self::ExtraActive => 1.9,
        }
    }
}

impl FromStr for Activity {
    type Err = &'static str;
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        match text {
            "1" => Ok(Self::Sedentary),
            "2" => Ok(Self::Light),
            "3" => Ok(Self::Moderate),
            "4" => Ok(Self::VeryActive),
            "5" => Ok(Self::ExtraActive),
            _ => Err("choose 1 to 5"),
        }
    }
}

// These are estimates from an adult equation, with rough activity multipliers.
fn calculate_calories(
    weight: f64,
    height: f64,
    age: u32,
    sex: Sex,
    activity: Activity,
) -> Result<(f64, f64), &'static str> {
    if !weight.is_finite() || weight <= 0.0 || !height.is_finite() || height <= 0.0 {
        return Err("weight and height must be positive finite numbers");
    }
    if !(19..=78).contains(&age) {
        return Err("this example covers the original study's adult age range, 19 to 78");
    }
    let offset = match sex {
        Sex::Male => 5.0,
        Sex::Female => -161.0,
    };
    let resting = 10.0 * weight + 6.25 * height - 5.0 * f64::from(age) + offset;
    let daily = resting * activity.multiplier();
    if !resting.is_finite() || !daily.is_finite() || resting <= 0.0 {
        return Err("these inputs do not produce a valid positive estimate");
    }
    Ok((resting, daily))
}

fn read_value<T: FromStr>(prompt: &str, valid: impl Fn(&T) -> bool) -> io::Result<T> {
    loop {
        println!("{prompt}");
        let mut input = String::new();
        if io::stdin().read_line(&mut input)? == 0 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "input ended"));
        }
        if let Ok(value) = input.trim().parse::<T>() {
            if valid(&value) {
                return Ok(value);
            }
        }
        println!("Please enter a valid value from the range shown.");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Adult energy estimate: Mifflin–St Jeor. Results are estimates in kcal/day.");
    let weight = read_value("Weight in kilograms (positive):", |n: &f64| {
        n.is_finite() && *n > 0.0
    })?;
    let height = read_value("Height in centimetres (positive):", |n: &f64| {
        n.is_finite() && *n > 0.0
    })?;
    let age = read_value("Age in years (19–78, the original study's range):", |n| {
        (19..=78).contains(n)
    })?;
    let sex = read_value(
        "Sex coefficient from the original equation: 1 male, 2 female:",
        |_| true,
    )?;
    let activity = read_value(
        "Activity: 1 sedentary, 2 light, 3 moderate, 4 very active, 5 extra active:",
        |_| true,
    )?;
    let (resting, daily) = calculate_calories(weight, height, age, sex, activity)?;
    println!("Estimated resting energy: {resting:.2} kcal/day");
    println!("Estimated total daily energy: {daily:.2} kcal/day");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_choices_map_to_factors_and_both_equation_variants_work() {
        let activity = "3".parse().unwrap();
        assert_eq!(
            calculate_calories(70.0, 175.0, 30, Sex::Male, activity),
            Ok((1648.75, 2555.5625))
        );
        let (resting, daily) = calculate_calories(70.0, 175.0, 30, Sex::Female, activity).unwrap();
        assert_eq!(resting, 1482.75);
        assert!((daily - 2298.2625).abs() < 1e-9);
    }
    #[test]
    fn invalid_inputs_are_errors() {
        for weight in [f64::NAN, f64::INFINITY, -1.0, 0.0, f64::MAX] {
            assert!(calculate_calories(weight, 175.0, 30, Sex::Male, Activity::Light).is_err());
        }
        assert!(calculate_calories(70.0, 175.0, 10, Sex::Female, Activity::Light).is_err());
        assert!("6".parse::<Activity>().is_err());
    }
}
