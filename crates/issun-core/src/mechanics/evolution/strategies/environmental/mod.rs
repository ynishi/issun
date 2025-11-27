//! Environmental strategy implementations.
//!
//! This module contains concrete implementations of the EnvironmentalPolicy trait.

mod comprehensive;
mod humidity_based;
mod no_environment;
mod temperature_based;

pub use comprehensive::ComprehensiveEnvironment;
pub use humidity_based::HumidityBased;
pub use no_environment::NoEnvironment;
pub use temperature_based::TemperatureBased;
