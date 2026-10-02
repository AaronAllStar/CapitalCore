use crate::amount::Amount;
use crate::error::NumericError;
use crate::rounding::RoundingMode;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Exact fixed-point decimal backed by an `i128` mantissa and a `u32` scale (fractional digits).
/// Completely eliminates IEEE-754 floating-point inaccuracy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Decimal {
    mantissa: i128,
    scale: u32,
}

impl Decimal {
    pub const ZERO: Self = Self {
        mantissa: 0,
        scale: 0,
    };
    pub const ONE: Self = Self {
        mantissa: 1,
        scale: 0,
    };

    /// Creates a Decimal from raw mantissa and scale.
    #[must_use]
    pub const fn from_parts(mantissa: i128, scale: u32) -> Self {
        Self { mantissa, scale }
    }

    #[must_use]
    pub const fn mantissa(&self) -> i128 {
        self.mantissa
    }

    #[must_use]
    pub const fn scale(&self) -> u32 {
        self.scale
    }

    /// Converts an `Amount` with known precision into a `Decimal`.
    #[must_use]
    pub fn from_amount(amount: Amount, precision: u32) -> Self {
        Self {
            mantissa: amount.as_i64() as i128,
            scale: precision,
        }
    }

    /// Converts this `Decimal` into an `Amount` with the target precision, applying rounding.
    pub fn to_amount(
        self,
        target_precision: u32,
        mode: RoundingMode,
    ) -> Result<Amount, NumericError> {
        let res = self.rescale(target_precision, mode)?;
        if res.mantissa > i64::MAX as i128 || res.mantissa < i64::MIN as i128 {
            return Err(NumericError::Overflow("to_amount conversion"));
        }
        Ok(Amount::from_minor(res.mantissa as i64))
    }

    /// Rescales this Decimal to a new target scale with explicit rounding.
    pub fn rescale(self, target_scale: u32, mode: RoundingMode) -> Result<Self, NumericError> {
        if self.scale == target_scale {
            return Ok(self);
        }

        if target_scale > self.scale {
            let diff = target_scale - self.scale;
            let factor = 10_i128
                .checked_pow(diff)
                .ok_or(NumericError::Overflow("rescale factor"))?;
            let new_mantissa = self
                .mantissa
                .checked_mul(factor)
                .ok_or(NumericError::Overflow("rescale mantissa"))?;
            Ok(Self {
                mantissa: new_mantissa,
                scale: target_scale,
            })
        } else {
            let diff = self.scale - target_scale;
            let factor = 10_i128
                .checked_pow(diff)
                .ok_or(NumericError::Overflow("rescale factor"))?;
            // Divide mantissa by factor with RoundingMode
            if factor > i64::MAX as i128
                || self.mantissa > i64::MAX as i128
                || self.mantissa < i64::MIN as i128
            {
                // High precision 128-bit division
                let q = self.mantissa / factor;
                let r = self.mantissa % factor;
                let same_sign =
                    (self.mantissa > 0 && factor > 0) || (self.mantissa < 0 && factor < 0);
                let rem_abs = r.abs();
                let den_abs = factor.abs();
                let double_rem = rem_abs * 2;
                let adjust = match mode {
                    RoundingMode::Truncate => 0,
                    RoundingMode::Floor => {
                        if same_sign {
                            0
                        } else {
                            -1
                        }
                    }
                    RoundingMode::Ceiling => {
                        if same_sign {
                            1
                        } else {
                            0
                        }
                    }
                    RoundingMode::HalfUp => {
                        if double_rem >= den_abs {
                            if same_sign {
                                1
                            } else {
                                -1
                            }
                        } else {
                            0
                        }
                    }
                    RoundingMode::HalfDown => {
                        if double_rem > den_abs {
                            if same_sign {
                                1
                            } else {
                                -1
                            }
                        } else {
                            0
                        }
                    }
                    RoundingMode::HalfEven => {
                        if double_rem > den_abs {
                            if same_sign {
                                1
                            } else {
                                -1
                            }
                        } else if double_rem == den_abs {
                            if q % 2 != 0 {
                                if same_sign {
                                    1
                                } else {
                                    -1
                                }
                            } else {
                                0
                            }
                        } else {
                            0
                        }
                    }
                };
                let final_mantissa = q
                    .checked_add(adjust)
                    .ok_or(NumericError::Overflow("rescale"))?;
                Ok(Self {
                    mantissa: final_mantissa,
                    scale: target_scale,
                })
            } else {
                let div_res = RoundingMode::round_div(self.mantissa as i64, factor as i64, mode)?;
                Ok(Self {
                    mantissa: div_res as i128,
                    scale: target_scale,
                })
            }
        }
    }

    /// Checked addition of two Decimals. Rescales to the maximum scale.
    pub fn checked_add(self, other: Self) -> Result<Self, NumericError> {
        let target_scale = self.scale.max(other.scale);
        let a = self.rescale(target_scale, RoundingMode::HalfEven)?;
        let b = other.rescale(target_scale, RoundingMode::HalfEven)?;
        let mantissa = a
            .mantissa
            .checked_add(b.mantissa)
            .ok_or(NumericError::Overflow("Decimal::add"))?;
        Ok(Self {
            mantissa,
            scale: target_scale,
        })
    }

    /// Checked subtraction of two Decimals. Rescales to the maximum scale.
    pub fn checked_sub(self, other: Self) -> Result<Self, NumericError> {
        let target_scale = self.scale.max(other.scale);
        let a = self.rescale(target_scale, RoundingMode::HalfEven)?;
        let b = other.rescale(target_scale, RoundingMode::HalfEven)?;
        let mantissa = a
            .mantissa
            .checked_sub(b.mantissa)
            .ok_or(NumericError::Overflow("Decimal::sub"))?;
        Ok(Self {
            mantissa,
            scale: target_scale,
        })
    }

    /// Checked multiplication of two Decimals.
    pub fn checked_mul(self, other: Self) -> Result<Self, NumericError> {
        let mantissa = self
            .mantissa
            .checked_mul(other.mantissa)
            .ok_or(NumericError::Overflow("Decimal::mul"))?;
        let scale = self
            .scale
            .checked_add(other.scale)
            .ok_or(NumericError::Overflow("Decimal::mul scale"))?;
        Ok(Self { mantissa, scale })
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.scale == 0 {
            return write!(f, "{}", self.mantissa);
        }
        let is_neg = self.mantissa < 0;
        let abs_val = self.mantissa.abs();
        let factor = 10_i128.pow(self.scale);
        let whole = abs_val / factor;
        let frac = abs_val % factor;
        let frac_str = format!("{:0width$}", frac, width = self.scale as usize);
        if is_neg {
            write!(f, "-{whole}.{frac_str}")
        } else {
            write!(f, "{whole}.{frac_str}")
        }
    }
}

impl FromStr for Decimal {
    type Err = NumericError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();
        if s.is_empty() {
            return Err(NumericError::ParseError("empty string".to_string()));
        }

        let (is_neg, num_str) = if let Some(stripped) = s.strip_prefix('-') {
            (true, stripped)
        } else if let Some(stripped) = s.strip_prefix('+') {
            (false, stripped)
        } else {
            (false, s)
        };

        let parts: Vec<&str> = num_str.split('.').collect();
        match parts.len() {
            1 => {
                let whole: i128 = parts[0]
                    .parse()
                    .map_err(|_| NumericError::ParseError(s.to_string()))?;
                let mantissa = if is_neg { -whole } else { whole };
                Ok(Self { mantissa, scale: 0 })
            }
            2 => {
                let whole_str = parts[0];
                let frac_str = parts[1];
                let scale = frac_str.len() as u32;

                let combined_str = format!("{whole_str}{frac_str}");
                let raw_mantissa: i128 = combined_str
                    .parse()
                    .map_err(|_| NumericError::ParseError(s.to_string()))?;
                let mantissa = if is_neg { -raw_mantissa } else { raw_mantissa };
                Ok(Self { mantissa, scale })
            }
            _ => Err(NumericError::ParseError(format!(
                "multiple decimal points: '{s}'"
            ))),
        }
    }
}
