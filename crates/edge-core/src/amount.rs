use crate::error::NumericError;
use crate::rounding::RoundingMode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::Neg;

/// Strongly typed integer minor units for monetary quantities.
/// Internally stored as an `i64` count of minor units (e.g., cents for USD).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct Amount(i64);

impl Amount {
    pub const ZERO: Self = Self(0);
    pub const MIN: Self = Self(i64::MIN);
    pub const MAX: Self = Self(i64::MAX);

    /// Constructs an `Amount` directly from integer minor units.
    #[must_use]
    pub const fn from_minor(minor: i64) -> Self {
        Self(minor)
    }

    /// Constructs an `Amount` from major units and a decimal precision.
    /// e.g. from_major(10, 2) -> 1000 minor units ($10.00).
    pub fn from_major(major: i64, precision: u32) -> Result<Self, NumericError> {
        if precision > 18 {
            return Err(NumericError::InvalidPrecision(format!(
                "precision {precision} exceeds max 18"
            )));
        }
        let factor = 10_i64
            .checked_pow(precision)
            .ok_or(NumericError::Overflow("from_major factor"))?;
        let minor = major
            .checked_mul(factor)
            .ok_or(NumericError::Overflow("from_major multiplication"))?;
        Ok(Self(minor))
    }

    /// Access the underlying raw `i64` minor unit value.
    #[must_use]
    pub const fn as_i64(&self) -> i64 {
        self.0
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_positive(&self) -> bool {
        self.0 > 0
    }

    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.0 < 0
    }

    /// Checked absolute value. Returns `Err` on `i64::MIN`.
    pub fn checked_abs(self) -> Result<Self, NumericError> {
        self.0
            .checked_abs()
            .map(Self)
            .ok_or(NumericError::Overflow("checked_abs"))
    }

    /// Checked addition of two amounts.
    pub fn checked_add(self, other: Self) -> Result<Self, NumericError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or(NumericError::Overflow("checked_add"))
    }

    /// Checked subtraction of two amounts.
    pub fn checked_sub(self, other: Self) -> Result<Self, NumericError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or(NumericError::Overflow("checked_sub"))
    }

    /// Checked multiplication by an integer scalar.
    pub fn checked_mul(self, factor: i64) -> Result<Self, NumericError> {
        self.0
            .checked_mul(factor)
            .map(Self)
            .ok_or(NumericError::Overflow("checked_mul"))
    }

    /// Checked division by an integer divisor with explicit rounding mode.
    pub fn checked_div_round(self, divisor: i64, mode: RoundingMode) -> Result<Self, NumericError> {
        RoundingMode::round_div(self.0, divisor, mode).map(Self)
    }

    /// Formats the minor units into a human-readable decimal string given currency precision.
    /// e.g. Amount(100) with precision 2 -> "1.00", Amount(-50) with precision 2 -> "-0.50".
    #[must_use]
    pub fn format_with_precision(&self, precision: u32) -> String {
        if precision == 0 {
            return self.0.to_string();
        }

        let is_neg = self.0 < 0;
        let abs_val = (self.0 as i128).abs();
        let factor = 10_i128.pow(precision);
        let whole = abs_val / factor;
        let frac = abs_val % factor;

        let frac_str = format!("{:0width$}", frac, width = precision as usize);
        if is_neg {
            format!("-{whole}.{frac_str}")
        } else {
            format!("{whole}.{frac_str}")
        }
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Neg for Amount {
    type Output = Result<Amount, NumericError>;

    fn neg(self) -> Self::Output {
        self.0
            .checked_neg()
            .map(Amount)
            .ok_or(NumericError::Overflow("neg"))
    }
}
