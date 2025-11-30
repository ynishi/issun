//! Systems for the spatial plugin.

use bevy::{
    ecs::message::{MessageReader, MessageWriter},
    prelude::*,
};
use issun_core::mechanics::spatial::{GraphSpatialMechanic, OccupancyState, SpatialQuery};
use issun_core::mechanics::{EventEmitter, Mechanic};

use super::types::{
    MoveEntityRequest, SpatialGraphResource, SpatialLocation, SpatialQueryRequest,
    SpatialQueryResult,
};

/// System: Process spatial queries.
pub fn handle_spatial_queries(
    graph: Res<SpatialGraphResource>,
    mut state: ResMut<OccupancyStateResource>,
    mut query_requests: MessageReader<SpatialQueryRequest>,
    mut query_results: MessageWriter<SpatialQueryResult>,
) {
    for request in query_requests.read() {
        let mut emitter = BevySpatialEventEmitter {
            writer: &mut query_results,
        };

        GraphSpatialMechanic::step(
            &graph.graph,
            &mut state.0,
            request.query.clone(),
            &mut emitter,
        );
    }
}

/// System: Handle entity movement requests.
pub fn handle_move_entity(
    graph: Res<SpatialGraphResource>,
    mut state: ResMut<OccupancyStateResource>,
    mut move_requests: MessageReader<MoveEntityRequest>,
    mut locations: Query<&mut SpatialLocation>,
    mut query_results: MessageWriter<SpatialQueryResult>,
) {
    for request in move_requests.read() {
        // Convert Entity to string ID
        let entity_id = format!("entity_{}", request.entity.index());

        // Create spatial query
        let query = SpatialQuery::UpdateOccupancy {
            entity: entity_id.clone(),
            from: request.from.clone(),
            to: request.to.clone(),
        };

        let mut emitter = BevySpatialEventEmitter {
            writer: &mut query_results,
        };

        GraphSpatialMechanic::step(&graph.graph, &mut state.0, query, &mut emitter);

        // Update component
        if let Ok(mut location) = locations.get_mut(request.entity) {
            location.node = request.to.clone();
        }
    }
}

/// System: Log spatial events for debugging.
pub fn log_spatial_events(mut events: MessageReader<SpatialQueryResult>) {
    for result in events.read() {
        debug!("Spatial event: {:?}", result.event);
    }
}

/// Resource wrapper for OccupancyState
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct OccupancyStateResource(#[reflect(ignore)] pub OccupancyState);

/// Bevy event emitter for spatial mechanic.
struct BevySpatialEventEmitter<'a, 'b> {
    writer: &'a mut MessageWriter<'b, SpatialQueryResult>,
}

impl<'a, 'b> EventEmitter<issun_core::mechanics::spatial::SpatialEvent>
    for BevySpatialEventEmitter<'a, 'b>
{
    fn emit(&mut self, event: issun_core::mechanics::spatial::SpatialEvent) {
        self.writer.write(SpatialQueryResult::new(event));
    }
}
