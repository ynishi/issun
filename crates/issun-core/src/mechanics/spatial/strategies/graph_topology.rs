//! Graph-based topology strategy.
//!
//! Uses explicit edges to define adjacency. This is the most flexible
//! topology and works with any graph structure.

use crate::mechanics::spatial::policies::TopologyPolicy;
use crate::mechanics::spatial::types::{NodeId, SpatialGraph};

/// Graph-based topology using explicit edges.
///
/// This strategy determines neighbors by looking at outgoing edges in the graph.
/// Supports both directed and undirected (bidirectional) edges.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
///
/// let mut graph = SpatialGraph::new();
/// graph.add_node(SpatialNode::new("A", NodeType::City));
/// graph.add_node(SpatialNode::new("B", NodeType::City));
/// graph.add_edge(SpatialEdge::new_bidirectional("A", "B", 10.0));
///
/// let topology = GraphTopology;
/// let neighbors = topology.neighbors(&graph, &"A".to_string());
/// assert!(neighbors.contains(&"B".to_string()));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GraphTopology;

impl TopologyPolicy for GraphTopology {
    fn neighbors(&self, graph: &SpatialGraph, node: &NodeId) -> Vec<NodeId> {
        let mut neighbors = Vec::new();

        // Add all direct outgoing edges
        for edge in graph.outgoing_edges(node) {
            neighbors.push(edge.to.clone());
        }

        // Add bidirectional edges (reverse direction)
        for edge in graph.incoming_edges(node) {
            if edge.bidirectional {
                neighbors.push(edge.from.clone());
            }
        }

        neighbors
    }

    fn are_adjacent(&self, graph: &SpatialGraph, a: &NodeId, b: &NodeId) -> bool {
        // Check both directions for bidirectional edges
        graph.outgoing_edges(a).any(|e| &e.to == b)
            || graph
                .incoming_edges(a)
                .any(|e| &e.from == b && e.bidirectional)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::spatial::types::{NodeType, SpatialEdge, SpatialNode};

    #[test]
    fn test_directed_graph() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_edge(SpatialEdge::new("A", "B", 1.0));

        let topology = GraphTopology;

        let neighbors_a = topology.neighbors(&graph, &"A".to_string());
        assert_eq!(neighbors_a, vec!["B".to_string()]);

        let neighbors_b = topology.neighbors(&graph, &"B".to_string());
        assert!(neighbors_b.is_empty());

        assert!(topology.are_adjacent(&graph, &"A".to_string(), &"B".to_string()));
        assert!(!topology.are_adjacent(&graph, &"B".to_string(), &"A".to_string()));
    }

    #[test]
    fn test_bidirectional_graph() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_edge(SpatialEdge::new_bidirectional("A", "B", 1.0));

        let topology = GraphTopology;

        let neighbors_a = topology.neighbors(&graph, &"A".to_string());
        assert_eq!(neighbors_a, vec!["B".to_string()]);

        let neighbors_b = topology.neighbors(&graph, &"B".to_string());
        assert_eq!(neighbors_b, vec!["A".to_string()]);

        assert!(topology.are_adjacent(&graph, &"A".to_string(), &"B".to_string()));
        assert!(topology.are_adjacent(&graph, &"B".to_string(), &"A".to_string()));
    }

    #[test]
    fn test_multiple_neighbors() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_node(SpatialNode::new("C", NodeType::City));
        graph.add_edge(SpatialEdge::new("A", "B", 1.0));
        graph.add_edge(SpatialEdge::new("A", "C", 1.0));

        let topology = GraphTopology;
        let mut neighbors = topology.neighbors(&graph, &"A".to_string());
        neighbors.sort();

        assert_eq!(neighbors, vec!["B".to_string(), "C".to_string()]);
    }
}
