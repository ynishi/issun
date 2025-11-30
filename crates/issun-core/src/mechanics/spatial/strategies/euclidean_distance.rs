//! Euclidean distance strategy.
//!
//! Calculates straight-line distance based on node positions.

use crate::mechanics::spatial::policies::DistancePolicy;
use crate::mechanics::spatial::types::{NodeId, SpatialGraph};

/// Euclidean distance policy using node positions.
///
/// Calculates distance as √((x₁-x₂)² + (y₁-y₂)² + (z₁-z₂)²).
/// Nodes must have positions set for this to work.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
///
/// let mut graph = SpatialGraph::new();
/// graph.add_node(
///     SpatialNode::new("A", NodeType::City)
///         .with_position(Position::new_2d(0.0, 0.0))
/// );
/// graph.add_node(
///     SpatialNode::new("B", NodeType::City)
///         .with_position(Position::new_2d(3.0, 4.0))
/// );
///
/// let distance_policy = EuclideanDistance;
/// let dist = distance_policy.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
/// assert_eq!(dist, Some(5.0));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EuclideanDistance;

impl DistancePolicy for EuclideanDistance {
    fn calculate_distance(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId) -> Option<f32> {
        // Same node = 0 distance
        if from == to {
            return Some(0.0);
        }

        let from_node = graph.get_node(from)?;
        let to_node = graph.get_node(to)?;

        let from_pos = from_node.position?;
        let to_pos = to_node.position?;

        Some(from_pos.distance_to(&to_pos))
    }

    fn movement_cost(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId) -> Option<f32> {
        // Use Euclidean distance as movement cost (ignores edge weights)
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
            SpatialNode::new("A", NodeType::City).with_position(Position::new_2d(0.0, 0.0)),
        );

        let distance = EuclideanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"A".to_string());
        assert_eq!(dist, Some(0.0));
    }

    #[test]
    fn test_2d_distance() {
        let mut graph = SpatialGraph::new();
        graph.add_node(
            SpatialNode::new("A", NodeType::City).with_position(Position::new_2d(0.0, 0.0)),
        );
        graph.add_node(
            SpatialNode::new("B", NodeType::City).with_position(Position::new_2d(3.0, 4.0)),
        );

        let distance = EuclideanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, Some(5.0));
    }

    #[test]
    fn test_3d_distance() {
        let mut graph = SpatialGraph::new();
        graph.add_node(
            SpatialNode::new("A", NodeType::Room).with_position(Position::new_3d(0.0, 0.0, 0.0)),
        );
        graph.add_node(
            SpatialNode::new("B", NodeType::Room).with_position(Position::new_3d(1.0, 2.0, 2.0)),
        );

        let distance = EuclideanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, Some(3.0)); // √(1² + 2² + 2²) = 3
    }

    #[test]
    fn test_no_position() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City)); // No position
        graph.add_node(SpatialNode::new("B", NodeType::City)); // No position

        let distance = EuclideanDistance;
        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, None);
    }

    #[test]
    fn test_nonexistent_node() {
        let graph = SpatialGraph::new();
        let distance = EuclideanDistance;

        let dist = distance.calculate_distance(&graph, &"A".to_string(), &"B".to_string());
        assert_eq!(dist, None);
    }
}
