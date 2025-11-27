//! State machine mechanic - Manages 4-stage infection lifecycle
//!
//! This module provides a mechanic for managing infection state transitions
//! through a 4-stage lifecycle: Plain → Incubating → Active → Recovered → Plain.
//!
//! # Architecture
//!
//! - **StateMachineConfig**: Configuration for state durations and reinfection
//! - **StateMachinePolicy**: Strategy for determining state transitions
//! - **StateMachineMechanic**: Implements the Mechanic trait for lifecycle management
//!
//! # Design Philosophy
//!
//! This mechanic is separated from ContagionMechanic to achieve:
//! 1. **Single Responsibility**: State lifecycle management independent of infection mechanics
//! 2. **Reusability**: Can be used with different infection models
//! 3. **Testability**: State transitions can be tested independently
//!
//! # Examples
//!
//! ```
//! use issun_core::mechanics::state_machine::prelude::*;
//! use issun_core::mechanics::contagion::{InfectionState, Duration};
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Event collector
//! struct Collector { events: Vec<StateMachineEvent> }
//! impl EventEmitter<StateMachineEvent> for Collector {
//!     fn emit(&mut self, e: StateMachineEvent) { self.events.push(e); }
//! }
//!
//! // Configure state machine
//! let config = StateMachineConfig {
//!     incubation_duration: Duration::Turns(3),
//!     active_duration: Duration::Turns(5),
//!     immunity_duration: Duration::Turns(10),
//!     allow_reinfection: true,
//! };
//!
//! // Start with incubating state
//! let mut state = InfectionState::Incubating {
//!     elapsed: Duration::Turns(0),
//!     total_duration: Duration::Turns(3),
//! };
//!
//! // Advance time by 1 turn
//! let input = StateMachineInput {
//!     time_delta: Duration::Turns(1),
//! };
//!
//! let mut emitter = Collector { events: Vec::new() };
//!
//! StandardStateMachine::step(&config, &mut state, input, &mut emitter);
//!
//! // State should still be Incubating (1 < 3 turns)
//! assert!(matches!(state, InfectionState::Incubating { .. }));
//! ```

mod mechanic;
mod policies;
mod strategies;
mod types;

pub use mechanic::{StandardStateMachine, StateMachineMechanic};
pub use policies::StateMachinePolicy;
pub use strategies::*;
pub use types::*;

/// Prelude for state machine mechanics
pub mod prelude {
    pub use super::mechanic::{StandardStateMachine, StateMachineMechanic};
    pub use super::policies::StateMachinePolicy;
    pub use super::strategies::*;
    pub use super::types::*;
}
