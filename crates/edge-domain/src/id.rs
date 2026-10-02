use crate::error::DomainError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

macro_rules! define_id {
    ($name:ident, $doc:expr) => {
        #[doc = $doc]
        #[derive(
            Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[repr(transparent)]
        #[serde(transparent)]
        pub struct $name(Uuid);

        impl $name {
            /// Generates a new random UUIDv4 identifier.
            #[must_use]
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            /// Creates an identifier wrapping a specific UUID.
            #[must_use]
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            /// Returns the underlying raw UUID.
            #[must_use]
            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }

            /// Returns a nil (all zero) identifier.
            #[must_use]
            pub const fn nil() -> Self {
                Self(Uuid::nil())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl From<Uuid> for $name {
            fn from(uuid: Uuid) -> Self {
                Self(uuid)
            }
        }

        impl From<$name> for Uuid {
            fn from(id: $name) -> Self {
                id.0
            }
        }

        impl FromStr for $name {
            type Err = DomainError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::parse_str(s).map(Self).map_err(|e| {
                    DomainError::ValidationFailed(format!(
                        "invalid UUID for {}: {e}",
                        stringify!($name)
                    ))
                })
            }
        }
    };
}

define_id!(
    UserId,
    "Strongly-typed identifier for an authenticated user or customer account."
);
define_id!(
    EventId,
    "Strongly-typed identifier for an ingested financial event."
);
define_id!(
    TransactionId,
    "Strongly-typed identifier for an end-to-end financial transaction."
);
define_id!(
    RuleId,
    "Strongly-typed identifier for an evaluation or compliance rule."
);
define_id!(
    ModelId,
    "Strongly-typed identifier for an ML inference model version."
);
