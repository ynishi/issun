//! Convenient re-exports for the combat mechanic.
//!
//! This module provides a prelude that includes the most commonly used types
//! for working with the combat system.
//!
//! # Usage
//!
//! ```
//! use issun_core::mechanics::combat::prelude::*;
//!
//! // All common types are now in scope:
//! type MyGame = CombatMechanic<LinearDamageCalculation, SubtractiveDefense, NoElemental>;
//! ```

// Core mechanic
pub use super::mechanic::CombatMechanic;

// Types
pub use super::types::{CombatConfig, CombatEvent, CombatInput, CombatState, Element};

// Policies (for custom implementations)
pub use super::policies::{DamageCalculationPolicy, DefensePolicy, ElementalPolicy};

// Common strategies
pub use super::strategies::{
    ElementalAffinity, LinearDamageCalculation, NoElemental, PercentageReduction,
    ScalingDamageCalculation, SubtractiveDefense,
};
