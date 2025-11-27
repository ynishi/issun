//! Diplomacy mechanic: Social interaction and negotiation system.
//!
//! This module provides a policy-based system for modeling social interactions,
//! negotiations, and debates.

pub mod mechanic;
pub mod policies;
pub mod strategies;
pub mod types;

pub mod prelude;

pub use mechanic::DiplomacyMechanic;
pub use policies::{ContextPolicy, InfluencePolicy, ResistancePolicy};
pub use types::{ArgumentType, DiplomacyConfig, DiplomacyEvent, DiplomacyInput, DiplomacyState};
