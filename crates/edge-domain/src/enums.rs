use serde::{Deserialize, Serialize};
use std::fmt;

/// Originating channel of an ingested financial event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Channel {
    Web,
    Mobile,
    Api,
    Pos,
    Atm,
    Batch,
}

impl Channel {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Web => "WEB",
            Self::Mobile => "MOBILE",
            Self::Api => "API",
            Self::Pos => "POS",
            Self::Atm => "ATM",
            Self::Batch => "BATCH",
        }
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Nature of the financial event/transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EventType {
    Payment,
    Transfer,
    Withdrawal,
    Deposit,
    Refund,
    Chargeback,
    Authorization,
}

impl EventType {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Payment => "PAYMENT",
            Self::Transfer => "TRANSFER",
            Self::Withdrawal => "WITHDRAWAL",
            Self::Deposit => "DEPOSIT",
            Self::Refund => "REFUND",
            Self::Chargeback => "CHARGEBACK",
            Self::Authorization => "AUTHORIZATION",
        }
    }
}

impl fmt::Display for EventType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Assessed risk tier resulting from intelligence evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl RiskLevel {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Low => "LOW",
            Self::Medium => "MEDIUM",
            Self::High => "HIGH",
            Self::Critical => "CRITICAL",
        }
    }
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
