//! Spatial plugin definition.

use bevy::prelude::*;
use issun_core::mechanics::spatial::{OccupancyState, SpatialGraph};

use super::systems::{
    handle_move_entity, handle_spatial_queries, log_spatial_events, OccupancyStateResource,
};
use super::types::{
    MoveEntityRequest, SpatialGraphResource, SpatialLocation, SpatialQueryRequest,
    SpatialQueryResult,
};
use crate::IssunSet;

/// Spatial plugin for Bevy integration.
///
/// Provides spatial graph management, occupancy tracking, and spatial queries.
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::spatial::*;
/// use issun_core::mechanics::spatial::prelude::*;
///
/// let mut graph = SpatialGraph::new();
/// graph.add_node(SpatialNode::new("spawn", NodeType::City));
/// graph.add_node(SpatialNode::new("dungeon", NodeType::Room));
/// graph.add_edge(SpatialEdge::new("spawn", "dungeon", 100.0));
///
/// App::new()
///     .add_plugins(SpatialPlugin::with_graph(graph))
///     .run();
/// ```
pub struct SpatialPlugin {
    graph: SpatialGraph,
}

impl SpatialPlugin {
    /// Create a new spatial plugin with a graph.
    pub fn with_graph(graph: SpatialGraph) -> Self {
        Self { graph }
    }

    /// Create a new spatial plugin with an empty graph.
    pub fn new() -> Self {
        Self {
            graph: SpatialGraph::new(),
        }
    }
}

impl Default for SpatialPlugin {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for SpatialPlugin {
    fn build(&self, app: &mut App) {
        // Register types for reflection
        app.register_type::<SpatialGraphResource>()
            .register_type::<SpatialLocation>()
            .register_type::<OccupancyStateResource>();

        // Register resources
        app.insert_resource(SpatialGraphResource::new(self.graph.clone()))
            .insert_resource(OccupancyStateResource(OccupancyState::new()));

        // Register messages
        app.add_message::<SpatialQueryRequest>()
            .add_message::<SpatialQueryResult>()
            .add_message::<MoveEntityRequest>();

        // Register systems
        app.add_systems(
            Update,
            (
                handle_spatial_queries,
                handle_move_entity,
                log_spatial_events,
            )
                .chain()
                .in_set(IssunSet::Logic),
        );
    }
}
