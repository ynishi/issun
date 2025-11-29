//! Concrete strategy implementations for reputation policies.
//!
//! This module provides ready-to-use implementations of the policy traits
//! defined in the `policies` module. These strategies can be mixed and matched
//! to create custom reputation mechanics.
//!
//! # Available Strategies
//!
//! ## Change Strategies
//! - `LinearChange`: Direct delta application (most common)
//! - `LogarithmicChange`: Diminishing returns near extremes
//! - `ThresholdChange`: Different multipliers based on value tiers
//!
//! ## Decay Strategies
//! - `NoDecay`: No time-based degradation
//! - `LinearDecay`: Fixed amount per time unit
//! - `ExponentialDecay`: Percentage-based decay (natural forgetting)
//!
//! ## Clamp Strategies
//! - `HardClamp`: Strict min/max enforcement (most common)
//! - `ZeroClamp`: Only prevent negative values, unbounded growth
//! - `NoClamp`: Allow any value (for temperature, debt, etc.)

pub mod change;
pub mod clamp;
pub mod decay;

// Re-export all strategies for convenience
pub use change::{LinearChange, LogarithmicChange, ThresholdChange};
pub use clamp::{HardClamp, NoClamp, ZeroClamp};
pub use decay::{ExponentialDecay, LinearDecay, NoDecay};
