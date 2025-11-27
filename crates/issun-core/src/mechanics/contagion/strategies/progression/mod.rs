//! Progression strategy implementations.
//!
//! This module provides concrete implementations of the `ProgressionPolicy` trait.

mod linear;
mod threshold;

pub use linear::LinearProgression;
pub use threshold::ThresholdProgression;
