//! Grid-based topology strategy.
//!
//! Treats nodes as cells in a 2D grid, with neighbors determined by
//! adjacent grid positions (4-way or 8-way).

use crate::mechanics::spatial::policies::TopologyPolicy;
use crate::mechanics::spatial::types::{NodeId, SpatialGraph};

/// Grid-based topology using node positions.
///
/// This strategy determines neighbors based on grid coordinates.
/// Nodes must have positions set for this to work.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
///
/// let mut graph = SpatialGraph::new();
/// graph.add_node(
///     SpatialNode::new("0,0", NodeType::Cell)
///         .with_position(Position::new_2d(0.0, 0.0))
/// );
/// graph.add_node(
///     SpatialNode::new("1,0", NodeType::Cell)
///         .with_position(Position::new_2d(1.0, 0.0))
/// );
///
/// let topology = GridTopology::new(false); // 4-way
/// let neighbors = topology.neighbors(&graph, &"0,0".to_string());
/// assert!(neighbors.contains(&"1,0".to_string()));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridTopology {
    /// Whether to include diagonal neighbors (8-way grid)
    pub diagonal: bool,
}

impl GridTopology {
    /// Create a new grid topology.
    ///
    /// # Arguments
    ///
    /// * `diagonal` - If true, uses 8-way connectivity. If false, uses 4-way.
    pub fn new(diagonal: bool) -> Self {
        Self { diagonal }
    }

    /// Create a 4-way grid topology.
    pub fn four_way() -> Self {
        Self::new(false)
    }

    /// Create an 8-way grid topology.
    pub fn eight_way() -> Self {
        Self::new(true)
    }
}

impl Default for GridTopology {
    fn default() -> Self {
        Self::four_way()
    }
}

impl TopologyPolicy for GridTopology {
    fn neighbors(&self, graph: &SpatialGraph, node: &NodeId) -> Vec<NodeId> {
        let Some(current_node) = graph.get_node(node) else {
            return Vec::new();
        };

        let Some(pos) = current_node.position else {
            return Vec::new();
        };

        let mut neighbors = Vec::new();

        // Define neighbor offsets
        let offsets = if self.diagonal {
            // 8-way: cardinal + diagonal
            vec![
                (-1.0, 0.0),
                (1.0, 0.0),
                (0.0, -1.0),
                (0.0, 1.0),
                (-1.0, -1.0),
                (-1.0, 1.0),
                (1.0, -1.0),
                (1.0, 1.0),
            ]
        } else {
            // 4-way: cardinal only
            vec![(-1.0, 0.0), (1.0, 0.0), (0.0, -1.0), (0.0, 1.0)]
        };

        // Check each neighbor position
        for (dx, dy) in offsets {
            let neighbor_x = pos.x + dx;
            let neighbor_y = pos.y + dy;

            // Find node at this position
            for (other_id, other_node) in &graph.nodes {
                if let Some(other_pos) = other_node.position {
                    // Use epsilon comparison for floating point
                    const EPSILON: f32 = 0.01;
                    if (other_pos.x - neighbor_x).abs() < EPSILON
                        && (other_pos.y - neighbor_y).abs() < EPSILON
                    {
                        neighbors.push(other_id.clone());
                        break;
                    }
                }
            }
        }

        neighbors
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::spatial::types::{NodeType, Position, SpatialNode};

    fn create_grid(width: usize, height: usize) -> SpatialGraph {
        let mut graph = SpatialGraph::new();

        for y in 0..height {
            for x in 0..width {
                let id = format!("{},{}", x, y);
                graph.add_node(
                    SpatialNode::new(&id, NodeType::Cell)
                        .with_position(Position::new_2d(x as f32, y as f32)),
                );
            }
        }

        graph
    }

    #[test]
    fn test_four_way_grid() {
        let graph = create_grid(3, 3);
        let topology = GridTopology::four_way();

        // Center cell should have 4 neighbors
        let neighbors = topology.neighbors(&graph, &"1,1".to_string());
        assert_eq!(neighbors.len(), 4);
        assert!(neighbors.contains(&"0,1".to_string()));
        assert!(neighbors.contains(&"2,1".to_string()));
        assert!(neighbors.contains(&"1,0".to_string()));
        assert!(neighbors.contains(&"1,2".to_string()));

        // Corner cell should have 2 neighbors
        let neighbors = topology.neighbors(&graph, &"0,0".to_string());
        assert_eq!(neighbors.len(), 2);
        assert!(neighbors.contains(&"1,0".to_string()));
        assert!(neighbors.contains(&"0,1".to_string()));
    }

    #[test]
    fn test_eight_way_grid() {
        let graph = create_grid(3, 3);
        let topology = GridTopology::eight_way();

        // Center cell should have 8 neighbors
        let neighbors = topology.neighbors(&graph, &"1,1".to_string());
        assert_eq!(neighbors.len(), 8);

        // Corner cell should have 3 neighbors
        let neighbors = topology.neighbors(&graph, &"0,0".to_string());
        assert_eq!(neighbors.len(), 3);
        assert!(neighbors.contains(&"1,0".to_string()));
        assert!(neighbors.contains(&"0,1".to_string()));
        assert!(neighbors.contains(&"1,1".to_string())); // Diagonal
    }

    #[test]
    fn test_no_position() {
        let mut graph = SpatialGraph::new();
        graph.add_node(SpatialNode::new("A", NodeType::City)); // No position

        let topology = GridTopology::four_way();
        let neighbors = topology.neighbors(&graph, &"A".to_string());
        assert!(neighbors.is_empty());
    }
}
