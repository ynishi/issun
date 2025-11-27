//! Convenient re-exports for the Evolution mechanic.
//!
//! This module provides a convenient way to import commonly used types and traits.
//!
//! # Examples
//!
//! ```
//! use issun_core::mechanics::evolution::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // All common types and strategies are available
//! type FoodSpoilage = EvolutionMechanic<Decay, HumidityBased, ExponentialRate>;
//!
//! let config = EvolutionConfig::default();
//! let mut state = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
//! let input = EvolutionInput {
//!     time_delta: 1.0,
//!     environment: Environment::new(25.0, 0.8),
//! };
//!
//! struct EventCollector { events: Vec<EvolutionEvent> }
//! impl EventEmitter<EvolutionEvent> for EventCollector {
//!     fn emit(&mut self, e: EvolutionEvent) { self.events.push(e); }
//! }
//! let mut emitter = EventCollector { events: vec![] };
//!
//! FoodSpoilage::step(&config, &mut state, input, &mut emitter);
//! ```

// Core mechanic
pub use super::mechanic::EvolutionMechanic;

// Policy traits
pub use super::policies::{DirectionPolicy, EnvironmentalPolicy, RateCalculationPolicy};

// All strategies
pub use super::strategies::{
    // Environmental strategies
    ComprehensiveEnvironment,
    // Direction strategies
    Cyclic,
    Decay,
    // Rate strategies
    DiminishingRate,
    ExponentialRate,
    Growth,
    HumidityBased,
    LinearRate,
    NoEnvironment,
    Oscillating,
    TemperatureBased,
    ThresholdRate,
};

// Types
pub use super::types::{
    Direction, Environment, EvolutionConfig, EvolutionEvent, EvolutionInput, EvolutionState,
    EvolutionStatus, SubjectType,
};

// ===== Preset Type Aliases =====

/// Organic growth with temperature dependency.
///
/// Use for: Plants, bacteria, biological growth
pub type OrganicGrowth = EvolutionMechanic<Growth, TemperatureBased, LinearRate>;

/// Food spoilage affected by humidity.
///
/// Use for: Perishable items, organic decay
pub type FoodDecay = EvolutionMechanic<Decay, HumidityBased, ExponentialRate>;

/// Natural resource regeneration with diminishing returns.
///
/// Use for: Mana, stamina, renewable resources
pub type ResourceRegeneration = EvolutionMechanic<Growth, NoEnvironment, DiminishingRate>;

/// Equipment wear and tear (linear decay, no environment).
///
/// Use for: Weapons, armor, tools
pub type EquipmentDegradation = EvolutionMechanic<Decay, NoEnvironment, LinearRate>;

/// Complex population growth/decay with environmental factors.
///
/// Use for: Animal populations, NPC demographics
pub type PopulationDynamics = EvolutionMechanic<Cyclic, ComprehensiveEnvironment, ThresholdRate>;

/// Oscillating seasonal changes.
///
/// Use for: Day/night cycles, seasonal effects
pub type SeasonalCycle = EvolutionMechanic<Oscillating, NoEnvironment, LinearRate>;

/// Simple decay (default configuration).
///
/// Use for: Basic degradation without environmental influence
pub type SimpleDecay = EvolutionMechanic<Decay, NoEnvironment, LinearRate>;

/// Simple growth (no environmental influence).
///
/// Use for: Basic accumulation without environmental influence
pub type SimpleGrowth = EvolutionMechanic<Growth, NoEnvironment, LinearRate>;
