//! Propagation mechanic - Graph-based infection spread
//!
//! This module provides a mechanic for propagating infection across
//! a network topology. It calculates infection pressure at each node
//! based on connected nodes' infection states and edge weights.
//!
//! # Architecture
//!
//! - **PropagationGraph**: Defines network topology (nodes + edges)
//! - **PropagationPolicy**: Strategy for calculating pressure and initial infection
//! - **PropagationMechanic**: Implements the Mechanic trait for graph-based spread
//!
//! # Design Philosophy
//!
//! This mechanic is separated from ContagionMechanic to achieve:
//! 1. **Single Responsibility**: Propagation logic independent of local infection mechanics
//! 2. **Reusability**: Can be used with different node state types (not just ContagionState)
//! 3. **Testability**: Graph algorithms can be tested independently
//!
//! # Examples
//!
//! ```
//! use issun_core::mechanics::propagation::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Event collector
//! struct Collector { events: Vec<PropagationEvent> }
//! impl EventEmitter<PropagationEvent> for Collector {
//!     fn emit(&mut self, e: PropagationEvent) { self.events.push(e); }
//! }
//!
//! // Define a simple 3-node graph: A -> B -> C
//! let graph = PropagationGraph::new(vec![
//!     PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
//!     PropagationEdge::new("B".to_string(), "C".to_string(), 0.3),
//! ]);
//!
//! // Node A is infected (severity 100), B and C are clean
//! let mut node_states = std::collections::HashMap::new();
//! node_states.insert("A".to_string(), 100.0);
//! node_states.insert("B".to_string(), 0.0);
//! node_states.insert("C".to_string(), 0.0);
//!
//! let input = PropagationInput { node_states };
//!
//! // Calculate infection pressure
//! let mut state = PropagationState::default();
//! let mut emitter = Collector { events: Vec::new() };
//!
//! LinearPropagationMechanic::step(&graph, &mut state, input, &mut emitter);
//!
//! // Node B should have pressure from A
//! assert!(state.node_pressures.get("B").unwrap() > &0.0);
//! ```

mod mechanic;
mod policies;
mod strategies;
mod types;

pub use mechanic::{LinearPropagationMechanic, PropagationMechanic};
pub use policies::PropagationPolicy;
pub use strategies::*;
pub use types::*;

/// Prelude for propagation mechanics
pub mod prelude {
    pub use super::mechanic::{LinearPropagationMechanic, PropagationMechanic};
    pub use super::policies::PropagationPolicy;
    pub use super::strategies::*;
    pub use super::types::*;
}
