<img src="../../rust-exampleslogo.png" alt="Rust Examples logo" width="96">

# Estimate resting energy with typed inputs

This console example turns menu choices into enums, checks numeric inputs, and calculates an **estimate of adult resting energy expenditure (REE)**. Food energy here uses kilocalories (`kcal`), often called Calories on food labels.

## The equation and its history

Mifflin and colleagues published their equation in 1990, using measurements from 498 adults aged 19–78. The simplified equations use weight in kilograms, height in centimetres and age in years:

```text
male coefficient:   REE = 10 × weight + 6.25 × height − 5 × age + 5
female coefficient: REE = 10 × weight + 6.25 × height − 5 × age − 161
```

These are the two sex coefficients from the original study. REE estimates energy used at rest; BMR refers to measurement under stricter basal conditions. Neither equation measures an individual's energy use directly. [Original paper and abstract](https://pubmed.ncbi.nlm.nih.gov/2305711/)

This program restricts age to 19–78 to keep its example within that study's age range. It is not a children's calorie calculator. Weight and height must be positive and finite, and calculated results must also be positive and finite.

## Menu choices are labels

Choosing activity `3` selects `Activity::Moderate`, whose factor is **1.55**. It does not multiply the result by three.

| Choice | Activity label | Factor |
|---|---|---|
| 1 | sedentary | 1.2 |
| 2 | light | 1.375 |
| 3 | moderate | 1.55 |
| 4 | very active | 1.725 |
| 5 | extra active | 1.9 |

These factors are rough assumptions used by this example to estimate total daily energy from REE; they are not part of the original resting-energy equation or precise measurements of activity.

For the male coefficient, 70 kg, 175 cm and age 30 give `1648.75 kcal/day` at rest. Selecting moderate activity gives `1648.75 × 1.55 = 2555.5625 kcal/day`, displayed to two decimal places. Displaying more digits would not make the estimate more accurate.

Invalid menu choices and inputs prompt another attempt. End-of-input returns an error rather than panicking or looping forever. Tests check both equation coefficients, activity mapping, invalid numbers and overflow.

```bash
cargo run
cargo test
```
