//! Core types for propagation mechanics

use std::collections::HashMap;

/// Node identifier type
///
/// Uses String for flexibility. Games can use any string format
/// (e.g., "downtown", "node_42", UUIDs, etc.)
pub type NodeId = String;

/// Propagation graph topology
///
/// Represents a directed graph where edges have weights (transmission rates).
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::propagation::{PropagationGraph, PropagationEdge};
///
/// let graph = PropagationGraph::new(vec![
///     PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
///     PropagationEdge::new("B".to_string(), "C".to_string(), 0.3),
/// ]);
///
/// assert_eq!(graph.edges.len(), 2);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PropagationGraph {
    pub edges: Vec<PropagationEdge>,
}

impl PropagationGraph {
    pub fn new(edges: Vec<PropagationEdge>) -> Self {
        Self { edges }
    }

    /// Get all edges leading to a specific node
    pub fn incoming_edges(&self, node: &NodeId) -> Vec<&PropagationEdge> {
        self.edges.iter().filter(|e| &e.to == node).collect()
    }

    /// Get all edges originating from a specific node
    pub fn outgoing_edges(&self, node: &NodeId) -> Vec<&PropagationEdge> {
        self.edges.iter().filter(|e| &e.from == node).collect()
    }

    /// Get all unique node IDs in the graph
    pub fn all_nodes(&self) -> Vec<NodeId> {
        let mut nodes = std::collections::HashSet::new();
        for edge in &self.edges {
            nodes.insert(edge.from.clone());
            nodes.insert(edge.to.clone());
        }
        nodes.into_iter().collect()
    }
}

/// Directed edge in the propagation graph
///
/// Represents a transmission pathway from one node to another
/// with an associated transmission rate.
#[derive(Debug, Clone, PartialEq)]
pub struct PropagationEdge {
    /// Source node
    pub from: NodeId,
    /// Target node
    pub to: NodeId,
    /// Transmission rate (0.0 to 1.0)
    pub rate: f32,
}

impl PropagationEdge {
    pub fn new(from: NodeId, to: NodeId, rate: f32) -> Self {
        Self { from, to, rate }
    }
}

/// Input for propagation calculation
///
/// Contains the current infection state of all nodes in the graph.
#[derive(Debug, Clone, PartialEq)]
pub struct PropagationInput {
    /// Node ID -> infection severity (0.0 to 100.0+)
    pub node_states: HashMap<NodeId, f32>,
}

impl PropagationInput {
    pub fn new() -> Self {
        Self {
            node_states: HashMap::new(),
        }
    }

    pub fn with_state(mut self, node: NodeId, severity: f32) -> Self {
        self.node_states.insert(node, severity);
        self
    }
}

impl Default for PropagationInput {
    fn default() -> Self {
        Self::new()
    }
}

/// State maintained by propagation mechanic
///
/// Stores the calculated infection pressure at each node.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct PropagationState {
    /// Node ID -> infection pressure (accumulated from incoming edges)
    pub node_pressures: HashMap<NodeId, f32>,
}

impl PropagationState {
    pub fn new() -> Self {
        Self {
            node_pressures: HashMap::new(),
        }
    }

    pub fn get_pressure(&self, node: &NodeId) -> f32 {
        self.node_pressures.get(node).copied().unwrap_or(0.0)
    }
}

/// Events emitted by propagation mechanic
#[derive(Debug, Clone, PartialEq)]
pub enum PropagationEvent {
    /// Infection pressure calculated for a node
    PressureCalculated { node: NodeId, pressure: f32 },

    /// Initial infection triggered at a node
    InitialInfection { node: NodeId, initial_severity: u32 },

    /// Pressure increased at already-infected node
    PressureIncreased {
        node: NodeId,
        old_pressure: f32,
        new_pressure: f32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_graph_incoming_edges() {
        let graph = PropagationGraph::new(vec![
            PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
            PropagationEdge::new("C".to_string(), "B".to_string(), 0.3),
            PropagationEdge::new("B".to_string(), "D".to_string(), 0.2),
        ]);

        let incoming = graph.incoming_edges(&"B".to_string());
        assert_eq!(incoming.len(), 2);
        assert!(incoming.iter().any(|e| e.from == "A"));
        assert!(incoming.iter().any(|e| e.from == "C"));
    }

    #[test]
    fn test_propagation_graph_outgoing_edges() {
        let graph = PropagationGraph::new(vec![
            PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
            PropagationEdge::new("A".to_string(), "C".to_string(), 0.3),
            PropagationEdge::new("B".to_string(), "D".to_string(), 0.2),
        ]);

        let outgoing = graph.outgoing_edges(&"A".to_string());
        assert_eq!(outgoing.len(), 2);
        assert!(outgoing.iter().any(|e| e.to == "B"));
        assert!(outgoing.iter().any(|e| e.to == "C"));
    }

    #[test]
    fn test_propagation_graph_all_nodes() {
        let graph = PropagationGraph::new(vec![
            PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
            PropagationEdge::new("B".to_string(), "C".to_string(), 0.3),
        ]);

        let nodes = graph.all_nodes();
        assert_eq!(nodes.len(), 3);
        assert!(nodes.contains(&"A".to_string()));
        assert!(nodes.contains(&"B".to_string()));
        assert!(nodes.contains(&"C".to_string()));
    }

    #[test]
    fn test_propagation_input_builder() {
        let input = PropagationInput::new()
            .with_state("A".to_string(), 100.0)
            .with_state("B".to_string(), 50.0);

        assert_eq!(input.node_states.get("A"), Some(&100.0));
        assert_eq!(input.node_states.get("B"), Some(&50.0));
    }

    #[test]
    fn test_propagation_state_get_pressure() {
        let mut state = PropagationState::new();
        state.node_pressures.insert("A".to_string(), 0.42);

        assert_eq!(state.get_pressure(&"A".to_string()), 0.42);
        assert_eq!(state.get_pressure(&"B".to_string()), 0.0);
    }
}
