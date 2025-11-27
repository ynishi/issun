//! Bevy-specific types and adapters for propagation_v2 plugin.
//!
//! This module provides the glue between issun-core's pure propagation logic
//! and Bevy's ECS system.

use bevy::{ecs::message::MessageWriter, prelude::*};
use issun_core::mechanics::propagation::{PropagationEvent, PropagationGraph, PropagationState};
use issun_core::mechanics::EventEmitter;
use std::marker::PhantomData;

/// Propagation graph resource - wraps issun-core's PropagationGraph
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource, Default)]
pub struct PropagationGraphResource {
    #[reflect(ignore)]
    pub graph: PropagationGraph,
}

impl Default for PropagationGraphResource {
    fn default() -> Self {
        Self {
            graph: PropagationGraph::new(Vec::new()),
        }
    }
}

impl PropagationGraphResource {
    pub fn new(graph: PropagationGraph) -> Self {
        Self { graph }
    }
}

/// Message wrapper for issun-core's PropagationEvent (Bevy 0.17+)
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct PropagationEventWrapper {
    pub event: PropagationEvent,
}

/// Bevy adapter: Wraps Bevy's MessageWriter to implement EventEmitter.
///
/// This allows issun-core's propagation mechanic to emit events into Bevy's
/// message system without depending on Bevy directly.
pub struct BevyPropagationEmitter<'a, 'b> {
    writer: &'a mut MessageWriter<'b, PropagationEventWrapper>,
}

impl<'a, 'b> BevyPropagationEmitter<'a, 'b> {
    /// Create a new Bevy propagation event emitter.
    pub fn new(writer: &'a mut MessageWriter<'b, PropagationEventWrapper>) -> Self {
        Self { writer }
    }
}

impl<'a, 'b> EventEmitter<PropagationEvent> for BevyPropagationEmitter<'a, 'b> {
    fn emit(&mut self, event: PropagationEvent) {
        self.writer.write(PropagationEventWrapper { event });
    }
}

/// Component: Propagation state for a specific mechanic type
///
/// This component stores the propagation state (node pressures) for the graph.
/// Generic over the mechanic type to allow different propagation behaviors.
///
/// Note: Generic types cannot implement Reflect traits due to Bevy's type registration requirements.
#[derive(Component)]
#[allow(unknown_lints, missing_reflect)]
pub struct PropagationStateComponent<M> {
    pub state: PropagationState,
    _marker: PhantomData<M>,
}

impl<M> Default for PropagationStateComponent<M> {
    fn default() -> Self {
        Self {
            state: PropagationState::default(),
            _marker: PhantomData,
        }
    }
}

impl<M> PropagationStateComponent<M> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_pressure(&self, node: &str) -> f32 {
        self.state.get_pressure(&node.to_string())
    }
}

/// Component: Node infection severity
///
/// Stores the infection severity for a specific node in the propagation graph.
#[derive(Component, Reflect, Clone, Debug)]
#[reflect(Component)]
pub struct NodeSeverity {
    /// Node identifier
    pub node_id: String,

    /// Current infection severity (0.0 to 100.0+)
    pub severity: f32,
}

impl NodeSeverity {
    pub fn new(node_id: String, severity: f32) -> Self {
        Self { node_id, severity }
    }

    pub fn is_infected(&self) -> bool {
        self.severity > 0.0
    }

    pub fn set_severity(&mut self, severity: f32) {
        self.severity = severity.max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_severity() {
        let node = NodeSeverity::new("node_a".to_string(), 50.0);
        assert_eq!(node.node_id, "node_a");
        assert_eq!(node.severity, 50.0);
        assert!(node.is_infected());
    }

    #[test]
    fn test_node_severity_clean() {
        let node = NodeSeverity::new("node_b".to_string(), 0.0);
        assert!(!node.is_infected());
    }

    #[test]
    fn test_set_severity() {
        let mut node = NodeSeverity::new("node_c".to_string(), 10.0);
        node.set_severity(75.0);
        assert_eq!(node.severity, 75.0);

        // Negative values clamped to 0
        node.set_severity(-5.0);
        assert_eq!(node.severity, 0.0);
    }
}
