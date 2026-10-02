use crate::amount::Amount;
use crate::currency::Currency;
use crate::error::NumericError;
use crate::rounding::RoundingMode;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Sub};

/// Strongly-typed monetary value pairing an integer minor-unit amount with an explicit currency.
/// Guaranteed zero floating-point representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Money {
    amount: Amount,
    currency: Currency,
}

impl Money {
    /// Constructs a new `Money` instance from raw integer minor units (e.g. 100 minor units of USD = $1.00).
    #[must_use]
    pub const fn new(minor_units: i64, currency: Currency) -> Self {
        Self {
            amount: Amount::from_minor(minor_units),
            currency,
        }
    }

    /// Constructs `Money` from an existing `Amount` and currency.
    #[must_use]
    pub const fn from_amount(amount: Amount, currency: Currency) -> Self {
        Self { amount, currency }
    }

    /// Constructs `Money` from major units (e.g. 10 USD -> 1000 minor units).
    pub fn from_major(major_units: i64, currency: Currency) -> Result<Self, NumericError> {
        let amount = Amount::from_major(major_units, currency.precision())?;
        Ok(Self { amount, currency })
    }

    /// Returns a zero-value `Money` for the given currency.
    #[must_use]
    pub const fn zero(currency: Currency) -> Self {
        Self {
            amount: Amount::ZERO,
            currency,
        }
    }

    /// Returns the strongly-typed `Amount`.
    #[must_use]
    pub const fn amount(&self) -> Amount {
        self.amount
    }

    /// Returns the raw integer minor units.
    #[must_use]
    pub const fn minor_units(&self) -> i64 {
        self.amount.as_i64()
    }

    /// Returns the associated `Currency`.
    #[must_use]
    pub const fn currency(&self) -> Currency {
        self.currency
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.amount.is_zero()
    }

    #[must_use]
    pub const fn is_positive(&self) -> bool {
        self.amount.is_positive()
    }

    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.amount.is_negative()
    }

    /// Checked absolute value.
    pub fn checked_abs(&self) -> Result<Self, NumericError> {
        let amount = self.amount.checked_abs()?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Checked addition. Returns `Err` if currencies differ or on integer overflow.
    pub fn checked_add(&self, other: Self) -> Result<Self, NumericError> {
        if self.currency != other.currency {
            return Err(NumericError::CurrencyMismatch {
                expected: self.currency,
                found: other.currency,
            });
        }
        let amount = self.amount.checked_add(other.amount)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Checked subtraction. Returns `Err` if currencies differ or on integer overflow/underflow.
    pub fn checked_sub(&self, other: Self) -> Result<Self, NumericError> {
        if self.currency != other.currency {
            return Err(NumericError::CurrencyMismatch {
                expected: self.currency,
                found: other.currency,
            });
        }
        let amount = self.amount.checked_sub(other.amount)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Checked scalar multiplication.
    pub fn checked_mul(&self, factor: i64) -> Result<Self, NumericError> {
        let amount = self.amount.checked_mul(factor)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Checked division by an integer with explicit rounding mode.
    pub fn checked_div(&self, divisor: i64, mode: RoundingMode) -> Result<Self, NumericError> {
        let amount = self.amount.checked_div_round(divisor, mode)?;
        Ok(Self {
            amount,
            currency: self.currency,
        })
    }

    /// Formatted decimal amount string (e.g. "1.00").
    #[must_use]
    pub fn format_amount(&self) -> String {
        self.amount.format_with_precision(self.currency.precision())
    }

    /// Formatted string with currency symbol (e.g. "$1.00" or "¥100").
    #[must_use]
    pub fn format_display(&self) -> String {
        let amt_str = self.format_amount();
        let sym = self.currency.symbol();
        if self.amount.is_negative() {
            let abs_amt = self.amount.format_with_precision(self.currency.precision());
            let trimmed = abs_amt.strip_prefix('-').unwrap_or(&abs_amt);
            format!("-{sym}{trimmed}")
        } else {
            format!("{sym}{amt_str}")
        }
    }
}

impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.currency != other.currency {
            None
        } else {
            self.amount.partial_cmp(&other.amount)
        }
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.format_amount(), self.currency.code())
    }
}

impl Add for Money {
    type Output = Money;

    fn add(self, rhs: Money) -> Self::Output {
        self.checked_add(rhs).unwrap_or_else(|err| match err {
            NumericError::CurrencyMismatch { expected, found } => {
                panic!("Currency mismatch in Money::add: cannot add {found} to {expected}");
            }
            NumericError::Overflow(op) => {
                panic!("Overflow in Money::add: {op}");
            }
            other => panic!("Error in Money::add: {other}"),
        })
    }
}

impl Sub for Money {
    type Output = Money;

    fn sub(self, rhs: Money) -> Self::Output {
        self.checked_sub(rhs).unwrap_or_else(|err| match err {
            NumericError::CurrencyMismatch { expected, found } => {
                panic!("Currency mismatch in Money::sub: cannot subtract {found} from {expected}");
            }
            NumericError::Overflow(op) => {
                panic!("Overflow in Money::sub: {op}");
            }
            other => panic!("Error in Money::sub: {other}"),
        })
    }
}
