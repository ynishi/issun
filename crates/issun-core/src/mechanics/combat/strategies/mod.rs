//! Concrete strategy implementations for combat policies.
//!
//! This module contains all the concrete implementations of the policy traits
//! defined in `policies.rs`. Strategies are organized by policy type:
//!
//! - `damage`: Damage calculation strategies
//! - `defense`: Defense application strategies
//! - `elemental`: Elemental affinity strategies

pub mod damage;
pub mod defense;
pub mod elemental;

// Re-export common strategies for convenience
pub use damage::{LinearDamageCalculation, ScalingDamageCalculation};
pub use defense::{PercentageReduction, SubtractiveDefense};
pub use elemental::{ElementalAffinity, NoElemental};
