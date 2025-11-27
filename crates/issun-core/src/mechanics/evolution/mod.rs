//! Natural Evolution Mechanic - Time-based state changes.
//!
//! The Evolution mechanic models natural changes in entity state over time,
//! unifying concepts of growth, decay, and oscillation into a single system.
//!
//! # Overview
//!
//! This mechanic uses **Policy-Based Design** with three independent dimensions:
//!
//! 1. **DirectionPolicy**: Determines whether values grow, decay, cycle, or oscillate
//! 2. **EnvironmentalPolicy**: Determines how environmental factors affect evolution
//! 3. **RateCalculationPolicy**: Determines how the rate scales with current value
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::evolution::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Use a preset configuration
//! type MyMechanic = FoodDecay;  // Humidity-based exponential decay
//!
//! // Or build custom
//! type CustomMechanic = EvolutionMechanic<Growth, TemperatureBased, LinearRate>;
//!
//! // Create configuration
//! let config = EvolutionConfig {
//!     base_rate: 1.0,
//!     time_delta: 1.0,
//! };
//!
//! // Create state
//! let mut state = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
//!
//! // Create input
//! let input = EvolutionInput {
//!     time_delta: 1.0,
//!     environment: Environment::new(25.0, 0.8),
//! };
//!
//! // Event collector
//! struct EventCollector { events: Vec<EvolutionEvent> }
//! impl EventEmitter<EvolutionEvent> for EventCollector {
//!     fn emit(&mut self, e: EvolutionEvent) { self.events.push(e); }
//! }
//! let mut emitter = EventCollector { events: vec![] };
//!
//! // Execute
//! MyMechanic::step(&config, &mut state, input, &mut emitter);
//! ```
//!
//! # Available Presets
//!
//! - [`OrganicGrowth`](prelude::OrganicGrowth) - Plant/biological growth with temperature
//! - [`FoodDecay`](prelude::FoodDecay) - Food spoilage with humidity
//! - [`ResourceRegeneration`](prelude::ResourceRegeneration) - Resource regeneration with diminishing returns
//! - [`EquipmentDegradation`](prelude::EquipmentDegradation) - Linear equipment wear
//! - [`PopulationDynamics`](prelude::PopulationDynamics) - Complex population cycles
//! - [`SeasonalCycle`](prelude::SeasonalCycle) - Oscillating seasonal changes
//!
//! # Policy Implementations
//!
//! ## Direction Policies
//!
//! - [`Growth`](strategies::Growth) - Always increases
//! - [`Decay`](strategies::Decay) - Always decreases
//! - [`Cyclic`](strategies::Cyclic) - Switches between growth/decay at thresholds
//! - [`Oscillating`](strategies::Oscillating) - Sinusoidal oscillation over time
//!
//! ## Environmental Policies
//!
//! - [`NoEnvironment`](strategies::NoEnvironment) - No environmental influence
//! - [`TemperatureBased`](strategies::TemperatureBased) - Affected by temperature
//! - [`HumidityBased`](strategies::HumidityBased) - Affected by humidity
//! - [`ComprehensiveEnvironment`](strategies::ComprehensiveEnvironment) - Multiple factors
//!
//! ## Rate Calculation Policies
//!
//! - [`LinearRate`](strategies::LinearRate) - Constant rate
//! - [`ExponentialRate`](strategies::ExponentialRate) - Proportional to current value
//! - [`DiminishingRate`](strategies::DiminishingRate) - Decreases near limits
//! - [`ThresholdRate`](strategies::ThresholdRate) - Changes at specific thresholds
//!
//! # Examples
//!
//! ## Food Spoilage
//!
//! ```
//! use issun_core::mechanics::evolution::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! type FoodSpoilage = EvolutionMechanic<Decay, HumidityBased, ExponentialRate>;
//!
//! let config = EvolutionConfig { base_rate: 1.0, time_delta: 1.0 };
//! let mut food = EvolutionState::new(100.0, 0.0, 100.0, SubjectType::Food);
//!
//! // High humidity accelerates spoilage
//! let humid_env = EvolutionInput {
//!     time_delta: 1.0,
//!     environment: Environment::new(25.0, 0.9),
//! };
//!
//! struct EventCollector { events: Vec<EvolutionEvent> }
//! impl EventEmitter<EvolutionEvent> for EventCollector {
//!     fn emit(&mut self, e: EvolutionEvent) { self.events.push(e); }
//! }
//! let mut emitter = EventCollector { events: vec![] };
//!
//! FoodSpoilage::step(&config, &mut food, humid_env, &mut emitter);
//!
//! // Food freshness has decreased
//! assert!(food.value < 100.0);
//! ```
//!
//! ## Plant Growth
//!
//! ```
//! use issun_core::mechanics::evolution::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! type PlantGrowth = EvolutionMechanic<Growth, TemperatureBased, DiminishingRate>;
//!
//! let config = EvolutionConfig { base_rate: 2.0, time_delta: 1.0 };
//! let mut plant = EvolutionState::new(10.0, 0.0, 100.0, SubjectType::Plant);
//!
//! let optimal_env = EvolutionInput {
//!     time_delta: 1.0,
//!     environment: Environment::new(25.0, 0.5),
//! };
//!
//! struct EventCollector { events: Vec<EvolutionEvent> }
//! impl EventEmitter<EvolutionEvent> for EventCollector {
//!     fn emit(&mut self, e: EvolutionEvent) { self.events.push(e); }
//! }
//! let mut emitter = EventCollector { events: vec![] };
//!
//! PlantGrowth::step(&config, &mut plant, optimal_env, &mut emitter);
//!
//! // Plant has grown, but with diminishing returns
//! assert!(plant.value > 10.0);
//! assert!(plant.value < 100.0);
//! ```

pub mod mechanic;
pub mod policies;
pub mod prelude;
pub mod strategies;
pub mod types;

// Re-export main types for convenience
pub use mechanic::EvolutionMechanic;
pub use policies::{DirectionPolicy, EnvironmentalPolicy, RateCalculationPolicy};
pub use types::{
    Direction, Environment, EvolutionConfig, EvolutionEvent, EvolutionInput, EvolutionState,
    EvolutionStatus, SubjectType,
};
