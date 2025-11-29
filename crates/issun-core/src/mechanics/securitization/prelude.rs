//! Convenient re-exports for common securitization usage.

pub use super::mechanic::SecuritizationMechanic;
pub use super::policies::{CollateralPolicy, IssuancePolicy};
pub use super::strategies::{FullBackingIssuance, SimpleCollateral};
pub use super::types::{
    RejectionReason, SecuritizationAction, SecuritizationConfig, SecuritizationEvent,
    SecuritizationInput, SecuritizationState,
};
