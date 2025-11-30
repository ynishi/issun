//! Fixed distance strategy.
//!
//! All adjacent nodes are considered to be exactly 1.0 distance apart.
//! Useful for abstract graphs where only hop count matters.

use crate::mechanics::spatial::policies::DistancePolicy;
use crate::mechanics::spatial::types::{NodeId, SpatialGraph};

/// Fixed distance policy where all edges have distance 1.0.
///
/// This is the simplest distance policy, treating the graph as unweighted.
/// Distance is measured in "hops" or graph edges traversed.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
///
/// let mut graph = SpatialGraph::new();
/// graph.add_node(SpatialNode::new("A", NodeType::City));
/// graph.add_node(SpatialNode::new("B", NodeType::City));
/// graph.add_edge(SpatialEdge::new("A", "B", 999.0)); // Cost ignored
///
/// let distance_policy = FixedDistance;
/// let dist = distance_policy.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
/// assert_eq!(dist, Some(1.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FixedDistance;

impl DistancePolicy for FixedDistance {
    fn calculate_distance(
        &self,
        graph: &SpatialGraph,
        from: &NodeId,
        to: &NodeId,
    ) -> Option<f32> {
        // Same node = 0 distance
        if from == to {
            return Some(0.0);
        }

        // Check if directly connected (using GraphTopology implicitly)
        let has_edge = graph.outgoing_edges(from).any(|e| &e.to == to)
            || graph
                .incoming_edges(from)
                .any(|e| &e.from == to && e.bidirectional);

        if has_edge {
            Some(1.0)
        } else {
            // Not directly connected, no distance available
            // (pathfinding would be needed for multi-hop distance)
            None
        }
    }

    fn movement_cost(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId) -> Option<f32> {
        // Override to always return 1.0 for existing edges
        let has_edge = graph.outgoing_edges(from).any(|e| &e.to == to)
            || graph
                .incoming_edges(from)
                .any(|e| &e.from == to && e.bidirectional);

        if has_edge {
            Some(1.0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::spatial::types::{NodeType, SpatialEdge, SpatialNode};

    #[test]
    fn test_same_node() {
        let graph = SpatialGraph::new();
        let distance = FixedDistance;

        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"A".to_string());
        assert_eq!(dist, Some(0.0));
    }

    #[test]
    fn test_adjacent_nodes() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_edge(SpatialEdge::new("A", "B", 42.0)); // Cost ignored

        let distance = FixedDistance;

        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, Some(1.0));

        let cost = distance.movement_cost(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(cost, Some(1.0));
    }

    #[test]
    fn test_not_connected() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        // No edge

        let distance = FixedDistance;

        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, None);
    }

    #[test]
    fn test_bidirectional() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City));
        graph.add_node(SpatialNode::new("B", NodeType::City));
        graph.add_edge(SpatialEdge::new_bidirectional("A", "B", 10.0));

        let distance = FixedDistance;

        let dist_ab = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        let dist_ba = distance.calculate_distance(&graph, &"B".to_string(), &"A".to_string());

        assert_eq!(dist_ab, Some(1.0));
        assert_eq!(dist_ba, Some(1.0));
    }
}
