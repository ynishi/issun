//! Damage calculation strategies.
//!
//! This module provides concrete implementations of `DamageCalculationPolicy`.

mod linear;
mod scaling;

pub use linear::LinearDamageCalculation;
pub use scaling::ScalingDamageCalculation;
