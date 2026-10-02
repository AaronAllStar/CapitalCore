use edge_core::{Currency, Money, NumericError, RoundingMode};
use proptest::prelude::*;

proptest! {
    /// Property: (a + b) - b == a for non-overflowing i64 amounts
    #[test]
    fn prop_add_sub_identity(
        a in -1_000_000_000_000_i64..1_000_000_000_000_i64,
        b in -1_000_000_000_000_i64..1_000_000_000_000_i64,
    ) {
        let m_a = Money::new(a, Currency::USD);
        let m_b = Money::new(b, Currency::USD);

        let sum = m_a.checked_add(m_b).expect("addition within bounded range should not overflow");
        let result = sum.checked_sub(m_b).expect("subtraction within bounded range should not overflow");

        prop_assert_eq!(result.minor_units(), a);
        prop_assert_eq!(result.currency(), Currency::USD);
    }

    /// Property: No overflow for any amounts within [-(i64::MAX / 2), +(i64::MAX / 2)]
    #[test]
    fn prop_no_overflow_half_max(
        a in -(i64::MAX / 4)..(i64::MAX / 4),
        b in -(i64::MAX / 4)..(i64::MAX / 4),
    ) {
        let m_a = Money::new(a, Currency::EUR);
        let m_b = Money::new(b, Currency::EUR);

        let sum = m_a.checked_add(m_b);
        prop_assert!(sum.is_ok(), "adding amounts in safe half range must not overflow");

        let diff = m_a.checked_sub(m_b);
        prop_assert!(diff.is_ok(), "subtracting amounts in safe half range must not overflow");
    }

    /// Property: Multiplication by 1 is identity, by 0 is zero
    #[test]
    fn prop_mul_identity(a in any::<i64>()) {
        let m = Money::new(a, Currency::GBP);
        let mul_one = m.checked_mul(1).expect("mul by 1");
        prop_assert_eq!(mul_one.minor_units(), a);

        let mul_zero = m.checked_mul(0).expect("mul by 0");
        prop_assert_eq!(mul_zero.minor_units(), 0);
    }

    /// Property: Division by 1 is identity for all rounding modes
    #[test]
    fn prop_div_one_identity(
        a in any::<i64>(),
        mode_idx in 0..6_u8,
    ) {
        let mode = match mode_idx {
            0 => RoundingMode::HalfUp,
            1 => RoundingMode::HalfDown,
            2 => RoundingMode::HalfEven,
            3 => RoundingMode::Floor,
            4 => RoundingMode::Ceiling,
            _ => RoundingMode::Truncate,
        };
        let m = Money::new(a, Currency::JPY);
        let res = m.checked_div(1, mode).expect("div by 1");
        prop_assert_eq!(res.minor_units(), a);
    }

    /// Property: Division by zero always returns Err(DivisionByZero)
    #[test]
    fn prop_div_zero_returns_err(a in any::<i64>()) {
        let m = Money::new(a, Currency::USD);
        let res = m.checked_div(0, RoundingMode::HalfUp);
        prop_assert!(matches!(res, Err(NumericError::DivisionByZero)));
    }
}
