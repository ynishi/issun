//! Convenient re-exports for the reputation mechanic.
//!
//! Import this module to get all the commonly used types and traits
//! for working with the reputation mechanic.
//!
//! # Example
//!
//! ```
//! use issun_core::mechanics::reputation::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // All types are available without qualification
//! let config = ReputationConfig::default();
//! let mut state = ReputationState::new(50.0);
//! let input = ReputationInput { delta: 10.0, elapsed_time: 0 };
//!
//! struct NoOpEmitter;
//! impl EventEmitter<ReputationEvent> for NoOpEmitter {
//!     fn emit(&mut self, _: ReputationEvent) {}
//! }
//! let mut emitter = NoOpEmitter;
//!
//! BasicReputation::step(&config, &mut state, input, &mut emitter);
//! ```

// Core mechanic
pub use super::mechanic::ReputationMechanic;

// Policy traits
pub use super::policies::{ChangePolicy, ClampPolicy, DecayPolicy};

// Strategies
pub use super::strategies::{
    ExponentialDecay, HardClamp, LinearChange, LinearDecay, LogarithmicChange, NoClamp, NoDecay,
    ThresholdChange, ZeroClamp,
};

// Types
pub use super::types::{ReputationConfig, ReputationEvent, ReputationInput, ReputationState};

// Presets
pub use super::presets::{
    BasicReputation, DurabilitySystem, EnvironmentalMetric, MoodSystem, RankSystem,
    ResourceQuantity, SkillProgression, TemporaryEffect,
};
