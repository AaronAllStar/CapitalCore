use crate::error::NumericError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Supported currencies with metadata for decimal precision.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[allow(clippy::upper_case_acronyms)]
#[non_exhaustive]
pub enum Currency {
    USD,
    EUR,
    GBP,
    JPY,
    CAD,
    AUD,
    CHF,
    CNY,
    BTC,
    ETH,
    USDT,
    USDC,
}

impl Currency {
    /// Returns the number of decimal places (minor units) for this currency.
    /// e.g. USD = 2 (cents), JPY = 0, BTC = 8 (satoshis).
    #[must_use]
    pub const fn precision(&self) -> u32 {
        match self {
            Self::JPY => 0,
            Self::USD | Self::EUR | Self::GBP | Self::CAD | Self::AUD | Self::CHF | Self::CNY => 2,
            Self::USDC | Self::USDT => 6,
            Self::BTC => 8,
            Self::ETH => 9, // Gwei scale within i64 bounds
        }
    }

    /// Scaling factor: 10^precision.
    #[must_use]
    pub const fn minor_unit_factor(&self) -> i64 {
        match self.precision() {
            0 => 1,
            1 => 10,
            2 => 100,
            3 => 1_000,
            4 => 10_000,
            5 => 100_000,
            6 => 1_000_000,
            7 => 10_000_000,
            8 => 100_000_000,
            9 => 1_000_000_000,
            _ => 1,
        }
    }

    /// Standard ISO-4217 or crypto ticker code.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::USD => "USD",
            Self::EUR => "EUR",
            Self::GBP => "GBP",
            Self::JPY => "JPY",
            Self::CAD => "CAD",
            Self::AUD => "AUD",
            Self::CHF => "CHF",
            Self::CNY => "CNY",
            Self::BTC => "BTC",
            Self::ETH => "ETH",
            Self::USDT => "USDT",
            Self::USDC => "USDC",
        }
    }

    /// Primary display symbol.
    #[must_use]
    pub const fn symbol(&self) -> &'static str {
        match self {
            Self::USD => "$",
            Self::EUR => "€",
            Self::GBP => "£",
            Self::JPY => "¥",
            Self::CAD => "CA$",
            Self::AUD => "A$",
            Self::CHF => "CHF",
            Self::CNY => "¥",
            Self::BTC => "₿",
            Self::ETH => "Ξ",
            Self::USDT => "₮",
            Self::USDC => "USDC",
        }
    }
}

impl fmt::Display for Currency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.code())
    }
}

impl FromStr for Currency {
    type Err = NumericError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_ascii_uppercase().as_str() {
            "USD" => Ok(Self::USD),
            "EUR" => Ok(Self::EUR),
            "GBP" => Ok(Self::GBP),
            "JPY" => Ok(Self::JPY),
            "CAD" => Ok(Self::CAD),
            "AUD" => Ok(Self::AUD),
            "CHF" => Ok(Self::CHF),
            "CNY" => Ok(Self::CNY),
            "BTC" => Ok(Self::BTC),
            "ETH" => Ok(Self::ETH),
            "USDT" => Ok(Self::USDT),
            "USDC" => Ok(Self::USDC),
            other => Err(NumericError::ParseError(format!(
                "unknown currency code: '{other}'"
            ))),
        }
    }
}
