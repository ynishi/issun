//! Rate calculation strategy implementations.
//!
//! This module contains concrete implementations of the RateCalculationPolicy trait.

mod diminishing;
mod exponential;
mod linear;
mod threshold;

pub use diminishing::DiminishingRate;
pub use exponential::ExponentialRate;
pub use linear::LinearRate;
pub use threshold::ThresholdRate;
