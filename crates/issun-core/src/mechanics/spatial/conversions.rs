//! Type conversions between spatial and other mechanics.
//!
//! This module provides conversion utilities to enable interoperability
//! between the spatial mechanic and other mechanics (e.g., propagation).

use crate::mechanics::propagation::{PropagationEdge, PropagationGraph};

use super::types::{NodeType, SpatialEdge, SpatialGraph, SpatialNode};

/// Convert a PropagationGraph to a SpatialGraph.
///
/// This allows existing propagation-based systems to use the spatial mechanic
/// without breaking changes.
///
/// # Conversion Details
///
/// - `PropagationEdge.rate` → `SpatialEdge.cost` (inverted: high rate = low cost)
/// - All edges are unidirectional (matching PropagationGraph behavior)
/// - Nodes are auto-generated with `NodeType::Custom` and no position
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::propagation::{PropagationGraph, PropagationEdge};
/// use issun_core::mechanics::spatial::conversions::propagation_to_spatial;
///
/// let prop_graph = PropagationGraph::new(vec![
///     PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
/// ]);
///
/// let spatial_graph = propagation_to_spatial(&prop_graph);
/// assert_eq!(spatial_graph.edges.len(), 1);
/// ```
pub fn propagation_to_spatial(propagation: &PropagationGraph) -> SpatialGraph {
    let mut spatial = SpatialGraph::new();

    // Extract all unique nodes from edges
    let node_ids = propagation.all_nodes();

    // Create SpatialNodes for each unique node
    for node_id in node_ids {
        spatial.add_node(SpatialNode::new(node_id, NodeType::Custom));
    }

    // Convert edges
    for prop_edge in &propagation.edges {
        // Convert rate to cost: high rate = low cost
        // rate = 1.0 → cost = 1.0
        // rate = 0.5 → cost = 2.0
        // This maintains relative weights while inverting semantics
        let cost = if prop_edge.rate > 0.0 {
            1.0 / prop_edge.rate
        } else {
            f32::MAX // Impassable edge
        };

        spatial.add_edge(SpatialEdge::new(
            prop_edge.from.clone(),
            prop_edge.to.clone(),
            cost,
        ));
    }

    spatial
}

/// Convert a SpatialGraph to a PropagationGraph.
///
/// This allows spatial-based systems to be used with propagation mechanics.
///
/// # Conversion Details
///
/// - `SpatialEdge.cost` → `PropagationEdge.rate` (inverted: high cost = low rate)
/// - Bidirectional edges are split into two unidirectional edges
/// - Node metadata (position, capacity) is lost
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::spatial::prelude::*;
/// use issun_core::mechanics::spatial::conversions::spatial_to_propagation;
///
/// let mut spatial = SpatialGraph::new();
/// spatial.add_node(SpatialNode::new("A", NodeType::City));
/// spatial.add_node(SpatialNode::new("B", NodeType::City));
/// spatial.add_edge(SpatialEdge::new("A", "B", 2.0));
///
/// let prop_graph = spatial_to_propagation(&spatial);
/// assert_eq!(prop_graph.edges.len(), 1);
/// assert_eq!(prop_graph.edges[0].rate, 0.5); // cost 2.0 → rate 0.5
/// ```
pub fn spatial_to_propagation(spatial: &SpatialGraph) -> PropagationGraph {
    let mut edges = Vec::new();

    for spatial_edge in &spatial.edges {
        // Convert cost to rate: high cost = low rate
        // cost = 1.0 → rate = 1.0
        // cost = 2.0 → rate = 0.5
        let rate = if spatial_edge.cost > 0.0 {
            1.0 / spatial_edge.cost
        } else {
            1.0 // Default to full transmission if cost is 0
        };

        edges.push(PropagationEdge::new(
            spatial_edge.from.clone(),
            spatial_edge.to.clone(),
            rate,
        ));

        // Handle bidirectional edges
        if spatial_edge.bidirectional {
            edges.push(PropagationEdge::new(
                spatial_edge.to.clone(),
                spatial_edge.from.clone(),
                rate,
            ));
        }
    }

    PropagationGraph::new(edges)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_propagation_to_spatial_basic() {
        let prop_graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.5,
        )]);

        let spatial = propagation_to_spatial(&prop_graph);

        assert_eq!(spatial.nodes.len(), 2);
        assert!(spatial.has_node(&"A".to_string()));
        assert!(spatial.has_node(&"B".to_string()));

        assert_eq!(spatial.edges.len(), 1);
        assert_eq!(spatial.edges[0].cost, 2.0); // rate 0.5 → cost 2.0
    }

    #[test]
    fn test_spatial_to_propagation_basic() {
        let mut spatial = SpatialGraph::new();
        spatial.add_node(SpatialNode::new("A", NodeType::City));
        spatial.add_node(SpatialNode::new("B", NodeType::City));
        spatial.add_edge(SpatialEdge::new("A", "B", 2.0));

        let prop_graph = spatial_to_propagation(&spatial);

        assert_eq!(prop_graph.edges.len(), 1);
        assert_eq!(prop_graph.edges[0].rate, 0.5); // cost 2.0 → rate 0.5
    }

    #[test]
    fn test_bidirectional_conversion() {
        let mut spatial = SpatialGraph::new();
        spatial.add_node(SpatialNode::new("A", NodeType::City));
        spatial.add_node(SpatialNode::new("B", NodeType::City));
        spatial.add_edge(SpatialEdge::new_bidirectional("A", "B", 1.0));

        let prop_graph = spatial_to_propagation(&spatial);

        // Bidirectional edge should become 2 directed edges
        assert_eq!(prop_graph.edges.len(), 2);

        let has_ab = prop_graph
            .edges
            .iter()
            .any(|e| e.from == "A" && e.to == "B");
        let has_ba = prop_graph
            .edges
            .iter()
            .any(|e| e.from == "B" && e.to == "A");

        assert!(has_ab);
        assert!(has_ba);
    }

    #[test]
    fn test_roundtrip_preserves_topology() {
        let prop_graph = PropagationGraph::new(vec![
            PropagationEdge::new("A".to_string(), "B".to_string(), 0.5),
            PropagationEdge::new("B".to_string(), "C".to_string(), 0.25),
        ]);

        let spatial = propagation_to_spatial(&prop_graph);
        let prop_graph2 = spatial_to_propagation(&spatial);

        // Should have same number of edges
        assert_eq!(prop_graph.edges.len(), prop_graph2.edges.len());

        // Check topology preservation
        for original_edge in &prop_graph.edges {
            let found = prop_graph2.edges.iter().any(|e| {
                e.from == original_edge.from
                    && e.to == original_edge.to
                    && (e.rate - original_edge.rate).abs() < 0.001
            });
            assert!(found, "Edge {:?} not found after roundtrip", original_edge);
        }
    }

    #[test]
    fn test_high_rate_low_cost() {
        let prop_graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            1.0,
        )]);

        let spatial = propagation_to_spatial(&prop_graph);
        assert_eq!(spatial.edges[0].cost, 1.0); // rate 1.0 → cost 1.0
    }

    #[test]
    fn test_zero_rate_max_cost() {
        let prop_graph = PropagationGraph::new(vec![PropagationEdge::new(
            "A".to_string(),
            "B".to_string(),
            0.0,
        )]);

        let spatial = propagation_to_spatial(&prop_graph);
        assert_eq!(spatial.edges[0].cost, f32::MAX); // rate 0.0 → max cost (impassable)
    }
}
