# Calorie Calculator

The Calorie Calculator is a simple Rust console application that helps you estimate your Basal Metabolic Rate (BMR) and Total Daily Energy Expenditure (TDEE). It takes your weight, height, age, and activity level as inputs and calculates the calories your body needs for maintaining its current weight.

## How It Works

1. When you run the application, it will prompt you to input your weight in kilograms, height in centimeters, age in years, and activity level.
2. Based on your input, the application will use the Mifflin-St Jeor Equation to calculate your BMR.
3. The BMR is then multiplied by your chosen activity level to estimate your TDEE.
4. The calculated BMR and TDEE are displayed on the console.

## Mathematical Functions Demonstrated

In the provided example of a Calorie Calculator console application, various mathematical functions and formulas are demonstrated:

### Basic Arithmetic Operations

The example utilizes basic arithmetic operations for performing calculations. These operations include:

- **Addition (`+`):** Used to add values together, such as adding terms in the BMR calculation.
- **Subtraction (`-`):** Used to subtract values, such as subtracting the age term in the BMR calculation.
- **Multiplication (`*`):** Used to multiply values, such as applying the weight and height terms in the BMR calculation.

### Mathematical Formulas

The example showcases the application of specific mathematical formulas to calculate BMR and TDEE:

- **Mifflin-St Jeor Equation:** This equation estimates the Basal Metabolic Rate (BMR) by combining weight, height, age, and gender-specific constants. The formula used in the example is:

```rust
BMR = 10.0 * weight + 6.25 * height - 5.0 * age + 5.0
```

Total Daily Energy Expenditure (TDEE): TDEE is calculated by multiplying the calculated BMR by the chosen activity level. This accounts for the calories burned through different levels of physical activity.

These mathematical concepts are translated into code within the example, showcasing how programming can implement mathematical operations to solve practical problems, such as estimating daily calorie needs.

## Usage

1. Run the application by executing the following command:
   ```bash
   cargo run
   ```