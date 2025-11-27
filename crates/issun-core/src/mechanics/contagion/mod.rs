//! Contagion mechanic: Disease/infection spreading system.
//!
//! This module provides a policy-based contagion system that can model
//! various types of disease spread, from zombie viruses to subtle plagues.
//!
//! # Architecture
//!
//! The contagion mechanic follows a **Policy-Based Design**:
//! - The core `ContagionMechanic<S, P>` is generic over two policies
//! - `S: SpreadPolicy` determines how infection spreads
//! - `P: ProgressionPolicy` determines how infection progresses
//! - All logic is resolved at compile time via static dispatch
//!
//! # Quick Start
//!
//! ```
//! use issun_core::mechanics::contagion::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define your virus type
//! type ZombieVirus = ContagionMechanic<ExponentialSpread, ThresholdProgression>;
//!
//! // Create configuration
//! let config = ContagionConfig { base_rate: 0.1 };
//! let mut state = SimpleSeverity::default();
//!
//! // Prepare input for this frame
//! let input = ContagionInput {
//!     density: 0.8,
//!     resistance: 5,
//!     rng: 0.05,
//! };
//!
//! // Simple event collector
//! # struct TestEmitter { events: Vec<ContagionEvent> }
//! # impl EventEmitter<ContagionEvent> for TestEmitter {
//! #     fn emit(&mut self, event: ContagionEvent) { self.events.push(event); }
//! # }
//! let mut emitter = TestEmitter { events: vec![] };
//!
//! // Execute one step
//! ZombieVirus::step(&config, &mut state, input, &mut emitter);
//! ```
//!
//! # Module Organization
//!
//! ## Core Modules
//! - `types`: Basic data structures (Config, Input, Event, SimpleSeverity)
//! - `policies`: Basic policy traits (SpreadPolicy, ProgressionPolicy)
//! - `strategies`: Concrete implementations of basic policies
//! - `mechanic`: The basic `ContagionMechanic<S, P>` implementation
//! - `presets`: Ready-to-use type aliases for common configurations
//!
//! ## Advanced Modules
//! - `duration`: Time abstraction (Turns/Ticks/Seconds)
//! - `state`: 4-stage infection state machine (Incubating/Active/Recovered/Plain)
//! - `content`: Rich content types (Disease, Political, Market trends, etc.)
//! - `advanced_policies`: Advanced policy traits (StateMachine, Mutation, Credibility, etc.)
//!
//! ## Convenience
//! - `prelude`: Convenient re-exports for common use

// Core modules (simple implementation)
pub mod mechanic;
pub mod policies;
pub mod presets;
pub mod strategies;
pub mod types;

// Advanced modules (stateful implementation)
pub mod advanced_policies;
pub mod content;
pub mod duration;
pub mod state;

// Convenience
pub mod prelude;

// Re-export core types for convenience
pub use mechanic::ContagionMechanic;
pub use policies::{ProgressionPolicy, SpreadPolicy};
pub use types::{ContagionConfig, ContagionEvent, ContagionInput, SimpleSeverity};

// Re-export advanced types
pub use content::{ContagionContent, DiseaseLevel, TrendDirection};
pub use duration::Duration;
pub use state::{InfectionState, InfectionStateType};
