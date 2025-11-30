//! Convenient re-exports for common macroeconomy usage.

pub use super::mechanic::MacroeconomyMechanic;
pub use super::policies::EconomicPolicy;
pub use super::strategies::SimpleEconomicPolicy;
pub use super::types::{
    CyclePhase, EconomicEvent, EconomicIndicators, EconomicParameters, EconomicSnapshot,
    InflationModelConfig, SentimentDirection, SentimentThresholds, ShockType,
};
