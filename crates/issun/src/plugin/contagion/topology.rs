//! Graph topology for contagion propagation

use super::types::{EdgeId, NodeId};
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Static graph topology (cities, trade routes, social networks)
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct GraphTopology {
    nodes: HashMap<NodeId, ContagionNode>,
    edges: HashMap<EdgeId, PropagationEdge>,
}

/// Node in the propagation graph
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContagionNode {
    pub id: NodeId,
    pub node_type: NodeType,
    /// Population size (affects propagation speed)
    pub population: usize,
    /// Resistance to contagion (0.0-1.0)
    ///
    /// Higher resistance = harder to spread to this node
    pub resistance: f32,
}

/// Type of node in the graph
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    City,
    Village,
    TradingPost,
    MilitaryBase,
    Custom(String),
}

/// Edge connecting two nodes for propagation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PropagationEdge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    /// Transmission rate (0.0-1.0)
    ///
    /// Probability multiplier for propagation across this edge
    pub transmission_rate: f32,
    /// Noise level during transmission (0.0-1.0)
    ///
    /// Higher noise = more likely to mutate during propagation
    pub noise_level: f32,
}

impl Resource for GraphTopology {}

impl GraphTopology {
    /// Create a new empty graph topology
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),
        }
    }

    /// Add a node to the graph
    pub fn add_node(&mut self, node: ContagionNode) -> &mut Self {
        self.nodes.insert(node.id.clone(), node);
        self
    }

    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: PropagationEdge) -> &mut Self {
        self.edges.insert(edge.id.clone(), edge);
        self
    }

    /// Get a node by ID
    pub fn get_node(&self, id: &NodeId) -> Option<&ContagionNode> {
        self.nodes.get(id)
    }

    /// Get an edge by ID
    pub fn get_edge(&self, id: &EdgeId) -> Option<&PropagationEdge> {
        self.edges.get(id)
    }

    /// Get all outgoing edges from a node
    pub fn get_outgoing_edges(&self, node_id: &NodeId) -> Vec<&PropagationEdge> {
        self.edges
            .values()
            .filter(|edge| &edge.from == node_id)
            .collect()
    }

    /// Get all incoming edges to a node
    pub fn get_incoming_edges(&self, node_id: &NodeId) -> Vec<&PropagationEdge> {
        self.edges
            .values()
            .filter(|edge| &edge.to == node_id)
            .collect()
    }

    /// Get all neighbors of a node (connected by outgoing edges)
    pub fn get_neighbors(&self, node_id: &NodeId) -> Vec<&ContagionNode> {
        self.get_outgoing_edges(node_id)
            .iter()
            .filter_map(|edge| self.get_node(&edge.to))
            .collect()
    }

    /// Get the number of nodes
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of edges
    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Check if a node exists
    pub fn has_node(&self, id: &NodeId) -> bool {
        self.nodes.contains_key(id)
    }

    /// Check if an edge exists
    pub fn has_edge(&self, id: &EdgeId) -> bool {
        self.edges.contains_key(id)
    }

    /// Get all nodes
    pub fn all_nodes(&self) -> impl Iterator<Item = &ContagionNode> {
        self.nodes.values()
    }

    /// Get all edges
    pub fn all_edges(&self) -> impl Iterator<Item = &PropagationEdge> {
        self.edges.values()
    }

    /// Remove a node and all connected edges
    pub fn remove_node(&mut self, id: &NodeId) -> Option<ContagionNode> {
        // Remove all edges connected to this node
        self.edges
            .retain(|_, edge| edge.from != *id && edge.to != *id);

        self.nodes.remove(id)
    }

    /// Remove an edge
    pub fn remove_edge(&mut self, id: &EdgeId) -> Option<PropagationEdge> {
        self.edges.remove(id)
    }
}

impl ContagionNode {
    /// Create a new node with default resistance
    pub fn new(id: impl Into<String>, node_type: NodeType, population: usize) -> Self {
        Self {
            id: id.into(),
            node_type,
            population,
            resistance: 0.0,
        }
    }

    /// Set resistance (clamped to 0.0-1.0)
    pub fn with_resistance(mut self, resistance: f32) -> Self {
        self.resistance = resistance.clamp(0.0, 1.0);
        self
    }
}

impl PropagationEdge {
    /// Create a new edge with default noise
    pub fn new(
        id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        transmission_rate: f32,
    ) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            transmission_rate: transmission_rate.clamp(0.0, 1.0),
            noise_level: 0.0,
        }
    }

    /// Set noise level (clamped to 0.0-1.0)
    pub fn with_noise(mut self, noise: f32) -> Self {
        self.noise_level = noise.clamp(0.0, 1.0);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topology_creation() {
        let topology = GraphTopology::new();
        assert_eq!(topology.node_count(), 0);
        assert_eq!(topology.edge_count(), 0);
    }

    #[test]
    fn test_add_node() {
        let mut topology = GraphTopology::new();

        let node = ContagionNode::new("london", NodeType::City, 100000);
        topology.add_node(node);

        assert_eq!(topology.node_count(), 1);
        assert!(topology.has_node(&"london".to_string()));
    }

    #[test]
    fn test_add_edge() {
        let mut topology = GraphTopology::new();

        topology.add_node(ContagionNode::new("london", NodeType::City, 100000));
        topology.add_node(ContagionNode::new("paris", NodeType::City, 80000));

        let edge = PropagationEdge::new("london_paris", "london", "paris", 0.8);
        topology.add_edge(edge);

        assert_eq!(topology.edge_count(), 1);
        assert!(topology.has_edge(&"london_paris".to_string()));
    }

    #[test]
    fn test_get_outgoing_edges() {
        let mut topology = GraphTopology::new();

        topology.add_node(ContagionNode::new("london", NodeType::City, 100000));
        topology.add_node(ContagionNode::new("paris", NodeType::City, 80000));
        topology.add_node(ContagionNode::new("berlin", NodeType::City, 70000));

        topology.add_edge(PropagationEdge::new("london_paris", "london", "paris", 0.8));
        topology.add_edge(PropagationEdge::new(
            "london_berlin",
            "london",
            "berlin",
            0.6,
        ));
        topology.add_edge(PropagationEdge::new("paris_berlin", "paris", "berlin", 0.7));

        let outgoing = topology.get_outgoing_edges(&"london".to_string());
        assert_eq!(outgoing.len(), 2);
    }

    #[test]
    fn test_get_neighbors() {
        let mut topology = GraphTopology::new();

        topology.add_node(ContagionNode::new("london", NodeType::City, 100000));
        topology.add_node(ContagionNode::new("paris", NodeType::City, 80000));
        topology.add_node(ContagionNode::new("berlin", NodeType::City, 70000));

        topology.add_edge(PropagationEdge::new("london_paris", "london", "paris", 0.8));
        topology.add_edge(PropagationEdge::new(
            "london_berlin",
            "london",
            "berlin",
            0.6,
        ));

        let neighbors = topology.get_neighbors(&"london".to_string());
        assert_eq!(neighbors.len(), 2);
    }

    #[test]
    fn test_node_with_resistance() {
        let node =
            ContagionNode::new("fortress", NodeType::MilitaryBase, 1000).with_resistance(0.8);

        assert_eq!(node.resistance, 0.8);
    }

    #[test]
    fn test_resistance_clamping() {
        let node = ContagionNode::new("test", NodeType::City, 1000).with_resistance(1.5);

        assert_eq!(node.resistance, 1.0);
    }

    #[test]
    fn test_edge_with_noise() {
        let edge = PropagationEdge::new("edge1", "a", "b", 0.5).with_noise(0.3);

        assert_eq!(edge.noise_level, 0.3);
    }

    #[test]
    fn test_remove_node() {
        let mut topology = GraphTopology::new();

        topology.add_node(ContagionNode::new("london", NodeType::City, 100000));
        topology.add_node(ContagionNode::new("paris", NodeType::City, 80000));
        topology.add_edge(PropagationEdge::new("london_paris", "london", "paris", 0.8));

        assert_eq!(topology.node_count(), 2);
        assert_eq!(topology.edge_count(), 1);

        topology.remove_node(&"london".to_string());

        assert_eq!(topology.node_count(), 1);
        assert_eq!(topology.edge_count(), 0); // Edge should be removed too
    }

    #[test]
    fn test_serialization() {
        let mut topology = GraphTopology::new();

        topology.add_node(ContagionNode::new("london", NodeType::City, 100000));
        topology.add_edge(PropagationEdge::new("edge1", "london", "paris", 0.8));

        let json = serde_json::to_string(&topology).unwrap();
        let deserialized: GraphTopology = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.node_count(), 1);
        assert_eq!(deserialized.edge_count(), 1);
    }

    #[test]
    fn test_custom_node_type() {
        let node = ContagionNode::new(
            "spacestation",
            NodeType::Custom("SpaceStation".to_string()),
            5000,
        );

        assert_eq!(node.node_type, NodeType::Custom("SpaceStation".to_string()));
    }
}
