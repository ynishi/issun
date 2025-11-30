//! Manhattan distance strategy.
//!
//! Calculates grid-based distance using Manhattan (taxicab) metric.

use crate::mechanics::spatial::policies::DistancePolicy;
use crate::mechanics::spatial::types::{NodeId, SpatialGraph};

/// Manhattan distance policy using node positions.
///
/// Calculates distance as |x₁-x₂| + |y₁-y₂|.
/// Useful for grid-based movement where diagonal movement is not allowed.
/// Nodes must have positions set for this to work.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
///
/// let mut graph = SpatialGraph::new();
/// graph.add_node(
///     SpatialNode::new("A", NodeType::Cell)
///         .with_position(Position::new_2d(0.0, 0.0))
/// );
/// graph.add_node(
///     SpatialNode::new("B", NodeType::Cell)
///         .with_position(Position::new_2d(3.0, 4.0))
/// );
///
/// let distance_policy = ManhattanDistance;
/// let dist = distance_policy.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
/// assert_eq!(dist, Some(7.0)); // |3-0| + |4-0| = 7
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ManhattanDistance;

impl DistancePolicy for ManhattanDistance {
    fn calculate_distance(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId) -> Option<f32> {
        // Same node = 0 distance
        if from == to {
            return Some(0.0);
        }

        let from_node = graph.get_node(from)?;
        let to_node = graph.get_node(to)?;

        let from_pos = from_node.position?;
        let to_pos = to_node.position?;

        Some(from_pos.manhattan_distance_to(&to_pos))
    }

    fn movement_cost(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId) -> Option<f32> {
        // Use Manhattan distance as movement cost (ignores edge weights)
        self.calculate_distance(graph, from, to)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::spatial::types::{NodeType, Position, SpatialNode};

    #[test]
    fn test_same_node() {
        let mut graph = SpatialGraph::new();
        graph.add_node(
            SpatialNode::new("A", NodeType::Cell).with_position(Position::new_2d(0.0, 0.0)),
        );

        let distance = ManhattanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"A".to_string());
        assert_eq!(dist, Some(0.0));
    }

    #[test]
    fn test_manhattan_distance() {
        let mut graph = SpatialGraph::new();
        graph.add_node(
            SpatialNode::new("A", NodeType::Cell).with_position(Position::new_2d(0.0, 0.0)),
        );
        graph.add_node(
            SpatialNode::new("B", NodeType::Cell).with_position(Position::new_2d(3.0, 4.0)),
        );

        let distance = ManhattanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, Some(7.0)); // |3-0| + |4-0| = 7
    }

    #[test]
    fn test_diagonal_costs_more() {
        let mut graph = SpatialGraph::new();
        graph.add_node(
            SpatialNode::new("A", NodeType::Cell).with_position(Position::new_2d(0.0, 0.0)),
        );
        graph.add_node(
            SpatialNode::new("B", NodeType::Cell).with_position(Position::new_2d(2.0, 2.0)),
        );

        let distance = ManhattanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, Some(4.0)); // |2-0| + |2-0| = 4

        // Compare with Euclidean: √(2² + 2²) = √8 ≈ 2.83
        // Manhattan is always >= Euclidean for same points
    }

    #[test]
    fn test_no_position() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::Cell)); // No position
        graph.add_node(SpatialNode::new("B", NodeType::Cell)); // No position

        let distance = ManhattanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, None);
    }

    #[test]
    fn test_symmetric() {
        let mut graph = SpatialGraph::new();
        graph.add_node(
            SpatialNode::new("A", NodeType::Cell).with_position(Position::new_2d(1.0, 2.0)),
        );
        graph.add_node(
            SpatialNode::new("B", NodeType::Cell).with_position(Position::new_2d(4.0, 6.0)),
        );

        let distance = ManhattanDistance;
        let dist_ab = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        let dist_ba = distance.calculate_distance(&graph, &"B".to_string(), &"A".to_string());

        assert_eq!(dist_ab, dist_ba);
        assert_eq!(dist_ab, Some(7.0)); // |4-1| + |6-2| = 3 + 4 = 7
    }
}
