//! Reputation mechanic: Score/value management system.
//!
//! This module provides a policy-based reputation system that can model
//! various types of scalar value mechanics, from NPC favorability to
//! durability systems.
//!
//! # Architecture
//!
//! The reputation mechanic follows a **Policy-Based Design**:
//! - The core `ReputationMechanic<C, D, K>` is generic over three policies
//! - `C: ChangePolicy` determines how delta changes are applied
//! - `D: DecayPolicy` determines how values naturally degrade over time
//! - `K: ClampPolicy` determines how out-of-range values are handled
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::reputation::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define your reputation type
//! type NPCFavor = ReputationMechanic<LinearChange, NoDecay, HardClamp>;
//!
//! // Create configuration
//! let config = ReputationConfig {
//!     min: 0.0,
//!     max: 100.0,
//!     decay_rate: 1.0, // No decay
//! };
//! let mut state = ReputationState::new(50.0);
//!
//! // Prepare input for this frame
//! let input = ReputationInput {
//!     delta: 10.0, // Player helped NPC
//!     elapsed_time: 0,
//! };
//!
//! // Simple event collector
//! # struct TestEmitter { events: Vec<ReputationEvent> }
//! # impl EventEmitter<ReputationEvent> for TestEmitter {
//! #     fn emit(&mut self, event: ReputationEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute one step
//! NPCFavor::step(&config, &mut state, input, &mut emitter);
//!
//! assert_eq!(state.value, 60.0);
//! ```
//!
//! # Module Organization
//!
//! ## Core Modules
//! - `types`: Basic data structures (Config, Input, Event, State)
//! - `policies`: Policy traits (ChangePolicy, DecayPolicy, ClampPolicy)
//! - `strategies`: Concrete implementations of policies
//! - `mechanic`: The `ReputationMechanic<C, D, K>` implementation
//! - `presets`: Ready-to-use type aliases for common configurations
//!
//! ## Convenience
//! - `prelude`: Convenient re-exports for common use
//!
//! # Common Use Cases
//!
//! ## NPC Favorability
//! ```
//! use issun_core::mechanics::reputation::presets::BasicReputation;
//! // LinearChange + NoDecay + HardClamp (0-100)
//! ```
//!
//! ## Item Durability
//! ```
//! use issun_core::mechanics::reputation::presets::DurabilitySystem;
//! // LinearChange + LinearDecay + ZeroClamp
//! ```
//!
//! ## Skill Progression
//! ```
//! use issun_core::mechanics::reputation::presets::SkillProgression;
//! // LogarithmicChange + ExponentialDecay + HardClamp
//! ```
//!
//! # Policy Combinations
//!
//! Mix and match policies to create custom systems:
//!
//! ```
//! use issun_core::mechanics::reputation::ReputationMechanic;
//! use issun_core::mechanics::reputation::strategies::*;
//!
//! // Custom: Temperature system
//! type Temperature = ReputationMechanic<LinearChange, LinearDecay, NoClamp>;
//!
//! // Custom: Debt system (can go negative)
//! type DebtSystem = ReputationMechanic<LinearChange, NoDecay, NoClamp>;
//!
//! // Custom: Rank with hard progression
//! type MilitaryRank = ReputationMechanic<ThresholdChange, NoDecay, HardClamp>;
//! ```

pub mod mechanic;
pub mod policies;
pub mod prelude;
pub mod presets;
pub mod strategies;
pub mod types;

// Re-export core types for convenience
pub use mechanic::ReputationMechanic;
pub use policies::{ChangePolicy, ClampPolicy, DecayPolicy};
pub use types::{ReputationConfig, ReputationEvent, ReputationInput, ReputationState};
