use crate::error::NumericError;
use serde::{Deserialize, Serialize};

/// Explicit rounding mode for financial division.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoundingMode {
    /// Half round away from zero (standard commercial / financial rounding).
    /// e.g. 1.5 -> 2, -1.5 -> -2, 1.4 -> 1, -1.4 -> -1
    HalfUp,
    /// Half round towards zero.
    /// e.g. 1.5 -> 1, -1.5 -> -1, 1.6 -> 2, -1.6 -> -2
    HalfDown,
    /// Banker's rounding: round half to the nearest even integer.
    /// e.g. 2.5 -> 2, 3.5 -> 4, -2.5 -> -2, -3.5 -> -4
    HalfEven,
    /// Round towards negative infinity.
    /// e.g. 1.1 -> 1, -1.1 -> -2
    Floor,
    /// Round towards positive infinity.
    /// e.g. 1.1 -> 2, -1.1 -> -1
    Ceiling,
    /// Truncate towards zero (standard integer division).
    /// e.g. 1.9 -> 1, -1.9 -> -1
    Truncate,
}

impl RoundingMode {
    /// Divides two i64 integers with explicit rounding mode.
    /// Returns Result<i64, NumericError>.
    pub fn round_div(
        numerator: i64,
        denominator: i64,
        mode: RoundingMode,
    ) -> Result<i64, NumericError> {
        if denominator == 0 {
            return Err(NumericError::DivisionByZero);
        }

        // Handle overflow case: i64::MIN / -1
        if numerator == i64::MIN && denominator == -1 {
            return Err(NumericError::Overflow("round_div"));
        }

        let quotient = numerator / denominator;
        let remainder = numerator % denominator;

        if remainder == 0 {
            return Ok(quotient);
        }

        let same_sign = (numerator > 0 && denominator > 0) || (numerator < 0 && denominator < 0);
        let rem_abs = (remainder as i128).abs();
        let den_abs = (denominator as i128).abs();
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
                    if quotient % 2 != 0 {
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

        quotient
            .checked_add(adjust)
            .ok_or(NumericError::Overflow("round_div"))
    }
}
