//! Policy traits for the spatial mechanic.
//!
//! These traits define customizable behaviors for topology and distance calculations.

use super::types::{NodeId, SpatialGraph};

/// Policy for determining spatial topology and adjacency.
///
/// This trait allows different spatial representations (graph, grid, hex, continuous)
/// to share the same core mechanic implementation.
pub trait TopologyPolicy {
    /// Get all directly connected neighbors of a node.
    ///
    /// # Arguments
    ///
    /// * `graph` - The spatial graph configuration
    /// * `node` - The node to query
    ///
    /// # Returns
    ///
    /// Vector of neighboring node IDs. Returns empty vector if node doesn't exist.
    fn neighbors(&self, graph: &SpatialGraph, node: &NodeId) -> Vec<NodeId>;

    /// Check if two nodes are directly adjacent.
    ///
    /// # Arguments
    ///
    /// * `graph` - The spatial graph configuration
    /// * `a` - First node ID
    /// * `b` - Second node ID
    ///
    /// # Returns
    ///
    /// `true` if nodes are adjacent, `false` otherwise.
    ///
    /// # Default Implementation
    ///
    /// Checks if `b` is in the neighbors of `a`. Override for bidirectional checks.
    fn are_adjacent(&self, graph: &SpatialGraph, a: &NodeId, b: &NodeId) -> bool {
        self.neighbors(graph, a).contains(b)
    }

    /// Check if movement is allowed from one node to another.
    ///
    /// # Arguments
    ///
    /// * `graph` - The spatial graph configuration
    /// * `from` - Source node ID
    /// * `to` - Target node ID
    ///
    /// # Returns
    ///
    /// `true` if movement is topologically allowed, `false` otherwise.
    ///
    /// # Default Implementation
    ///
    /// Checks adjacency. Override for more complex logic (e.g., locked doors).
    fn can_move(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId) -> bool {
        self.are_adjacent(graph, from, to)
    }
}

/// Policy for calculating distances between nodes.
///
/// This trait allows different distance metrics (Manhattan, Euclidean, path cost)
/// to be used with the same spatial mechanic.
pub trait DistancePolicy {
    /// Calculate distance between two nodes.
    ///
    /// # Arguments
    ///
    /// * `graph` - The spatial graph configuration
    /// * `from` - Source node ID
    /// * `to` - Target node ID
    ///
    /// # Returns
    ///
    /// Distance value, or `None` if distance cannot be calculated
    /// (e.g., nodes don't exist, no path exists).
    ///
    /// # Notes
    ///
    /// - Distance should be non-negative
    /// - Distance from a node to itself should be 0.0
    /// - Distance should be symmetric for undirected graphs
    fn calculate_distance(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId)
        -> Option<f32>;

    /// Get the movement cost for traversing an edge.
    ///
    /// # Arguments
    ///
    /// * `graph` - The spatial graph configuration
    /// * `from` - Source node ID
    /// * `to` - Target node ID
    ///
    /// # Returns
    ///
    /// Movement cost, or `None` if edge doesn't exist.
    ///
    /// # Default Implementation
    ///
    /// Returns the edge cost from the graph. Override for dynamic costs.
    fn movement_cost(&self, graph: &SpatialGraph, from: &NodeId, to: &NodeId) -> Option<f32> {
        graph
            .outgoing_edges(from)
            .find(|e| &e.to == to)
            .map(|e| e.cost)
    }
}
