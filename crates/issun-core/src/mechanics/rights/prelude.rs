//! Convenience re-exports for the rights mechanic.
//!
//! This module provides a convenient way to import all commonly used types
//! and traits for working with the rights mechanic.
//!
//! # Examples
//!
//! ```
//! use issun_core::mechanics::rights::prelude::*;
//!
//! // Now you have access to:
//! // - RightsMechanic
//! // - All strategies (AbsoluteRights, PartialRights, etc.)
//! // - All presets (ModernPropertyRights, StockOwnership, etc.)
//! // - Core types (RightsConfig, RightsState, etc.)
//! ```

// Core mechanic
pub use super::mechanic::RightsMechanic;

// Policies
pub use super::policies::{RecognitionPolicy, RightsSystemPolicy, TransferPolicy};

// Strategies
pub use super::strategies::{
    AbsoluteRights, AuthorityRecognition, ConsensusRecognition, FreeTransfer, LayeredRights,
    NonTransferable, PartialRights, RestrictedTransfer, SelfRecognition,
};

// Presets
pub use super::presets::{
    ContestedTerritory, DAOGovernance, FeudalRights, LeaseRights, ModernPropertyRights,
    PersonalRights, StateRecognizedProperty, StockOwnership,
};

// Core types
pub use super::types::{
    AssetId, Claim, ClaimStrength, EntityId, RejectionReason, RightsAction, RightsConfig,
    RightsEvent, RightsInput, RightsState,
};
