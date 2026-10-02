//! # `edge-core`
//!
//! Core primitives and financial numerical representations for the EdgeArena engine.
//!
//! Invariants strictly enforced:
//! 1. Zero floating-point representation for money (`f32`/`f64` forbidden in public API).
//! 2. Integer minor-unit arithmetic with explicit rounding modes and currency preservation.
//! 3. Checked arithmetic on all operations to prevent silent overflow.

#![deny(unsafe_code)]

pub mod amount;
pub mod currency;
pub mod decimal;
pub mod error;
pub mod money;
pub mod rounding;

pub use amount::Amount;
pub use currency::Currency;
pub use decimal::Decimal;
pub use error::NumericError;
pub use money::Money;
pub use rounding::RoundingMode;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_usd_representation() {
        let m = Money::new(100, Currency::USD);
        assert_eq!(m.minor_units(), 100);
        assert_eq!(m.currency(), Currency::USD);
        assert_eq!(m.format_amount(), "1.00");
        assert_eq!(m.format_display(), "$1.00");
    }

    #[test]
    fn test_money_add_preserves_currency() {
        let m1 = Money::new(150, Currency::USD);
        let m2 = Money::new(250, Currency::USD);
        let sum = m1.checked_add(m2).unwrap();
        assert_eq!(sum.currency(), Currency::USD);
        assert_eq!(sum.minor_units(), 400);

        // Standard Add trait
        let op_sum = m1 + m2;
        assert_eq!(op_sum.currency(), Currency::USD);
        assert_eq!(op_sum.minor_units(), 400);
    }

    #[test]
    fn test_money_add_mismatch_fails() {
        let usd = Money::new(100, Currency::USD);
        let eur = Money::new(100, Currency::EUR);
        let res = usd.checked_add(eur);
        assert!(matches!(res, Err(NumericError::CurrencyMismatch { .. })));
    }

    #[test]
    #[should_panic(expected = "Currency mismatch")]
    fn test_money_op_add_panics_on_mismatch() {
        let usd = Money::new(100, Currency::USD);
        let eur = Money::new(100, Currency::EUR);
        let _ = usd + eur;
    }

    #[test]
    fn test_money_division_rounding_modes() {
        let m = Money::new(5, Currency::USD); // $0.05

        // 5 / 2 = 2.5
        let half_up = m.checked_div(2, RoundingMode::HalfUp).unwrap();
        assert_eq!(half_up.minor_units(), 3);

        let half_down = m.checked_div(2, RoundingMode::HalfDown).unwrap();
        assert_eq!(half_down.minor_units(), 2);

        let truncate = m.checked_div(2, RoundingMode::Truncate).unwrap();
        assert_eq!(truncate.minor_units(), 2);

        // Banker's rounding (HalfEven)
        let m_even_half = Money::new(5, Currency::USD); // 2.5 -> 2 (even)
        let m_odd_half = Money::new(7, Currency::USD); // 3.5 -> 4 (even)
        assert_eq!(
            m_even_half
                .checked_div(2, RoundingMode::HalfEven)
                .unwrap()
                .minor_units(),
            2
        );
        assert_eq!(
            m_odd_half
                .checked_div(2, RoundingMode::HalfEven)
                .unwrap()
                .minor_units(),
            4
        );
    }

    #[test]
    fn test_money_overflow_returns_err() {
        let m1 = Money::new(i64::MAX, Currency::USD);
        let m2 = Money::new(1, Currency::USD);
        assert!(matches!(m1.checked_add(m2), Err(NumericError::Overflow(_))));
    }

    #[test]
    fn test_zero_floats_in_public_api() {
        // Compile-time verification that Money methods return Amount / i64 / Result
        let m = Money::new(100, Currency::USD);
        let _: i64 = m.minor_units();
        let _: Amount = m.amount();
        let _: Currency = m.currency();
    }
}
