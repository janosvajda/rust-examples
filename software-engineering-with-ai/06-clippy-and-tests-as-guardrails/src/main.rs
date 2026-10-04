// Lesson 6: clippy and tests as guardrails.
//
// The specification, written by a person BEFORE any code:
//
//   Apply a percentage discount to a price given in cents.
//   - The result is rounded to the nearest cent.
//   - A discount above 100% is an error.
//
// The function below follows it. The tests check the specification, not just
// whatever the code happens to output. See the README for the AI-written version
// that a "copied output" test happily approved.
//
// Cargo.toml turns on extra clippy lints for this crate (no `unwrap`, no `todo!`,
// no `dbg!`), so `cargo clippy` rejects those shortcuts here.

/// Apply `percent` discount to `price_cents`, rounding to the nearest cent.
pub fn discounted_price(price_cents: u32, percent: u32) -> Result<u32, String> {
    if percent > 100 {
        return Err(format!("a discount of {percent}% is more than the whole price"));
    }
    // Work in u64: price × percentage can be larger than a u32 can hold.
    let hundredths_of_cents = u64::from(price_cents) * u64::from(100 - percent);
    // Round to the nearest cent: add half a cent (50 hundredths) before dividing.
    let cents = (hundredths_of_cents + 50) / 100;
    Ok(u32::try_from(cents).expect("a discounted price is never more than the original"))
}

fn main() {
    for (price, percent) in [(999, 15), (1000, 0), (1000, 100), (1, 50), (1000, 101)] {
        println!("{price:>5} cents, {percent:>3}% off → {:?}", discounted_price(price, percent));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ---- Tests that check the specification ------------------------------------------------
    // Each expected value was worked out by hand from the specification, not by running the code.

    #[test]
    fn rounds_to_the_nearest_cent() {
        // 9.99 × 0.85 = 8.4915 → 8.49
        assert_eq!(discounted_price(999, 15), Ok(849));
        // 0.01 × 0.50 = 0.005 → rounds half up to 0.01
        assert_eq!(discounted_price(1, 50), Ok(1));
        // 0.03 × 0.50 = 0.015 → 0.02
        assert_eq!(discounted_price(3, 50), Ok(2));
    }

    #[test]
    fn edge_cases_of_the_percentage() {
        assert_eq!(discounted_price(1000, 0), Ok(1000)); // no discount
        assert_eq!(discounted_price(1000, 100), Ok(0)); // free
        assert!(discounted_price(1000, 101).is_err()); // more than free: an error, not a crash
    }

    #[test]
    fn the_largest_price_does_not_overflow() {
        assert_eq!(discounted_price(u32::MAX, 0), Ok(u32::MAX));
        assert_eq!(discounted_price(u32::MAX, 50), Ok(2_147_483_648)); // 2147483647.5 rounds up
    }

    // ---- A property test: a rule that must hold for MANY inputs ----------------------------
    // Instead of a few hand-picked examples, check rules the specification implies,
    // for thousands of combinations. Mistakes in rare corners show up here.

    #[test]
    fn a_discount_never_raises_the_price_and_more_discount_never_costs_more() {
        for price in (0..=100_000).step_by(997) {
            let mut previous = price;
            for percent in 0..=100 {
                let now = discounted_price(price, percent).expect("0-100% is always valid");
                assert!(now <= price, "{price} at {percent}% became {now}");
                assert!(now <= previous, "{price}: {percent}% costs more than {}%", percent - 1);
                previous = now;
            }
        }
    }
}
