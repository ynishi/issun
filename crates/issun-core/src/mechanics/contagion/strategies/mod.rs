//! Strategy implementations for the contagion mechanic.
//!
//! This module contains concrete implementations of the various policy traits
//! used by the contagion mechanic. Strategies are organized by the policy they implement.

pub mod progression;
pub mod spread;

// Re-export common strategies for convenience
pub use progression::{LinearProgression, ThresholdProgression};
pub use spread::{ExponentialSpread, LinearSpread};
