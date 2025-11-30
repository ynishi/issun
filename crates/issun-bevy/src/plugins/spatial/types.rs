//! Bevy-specific types for the spatial plugin.

use bevy::prelude::*;
use issun_core::mechanics::spatial::{NodeId, SpatialEvent, SpatialGraph, SpatialQuery};

/// Resource holding the spatial graph configuration.
#[derive(Resource, Clone)]
pub struct SpatialGraphResource {
    pub graph: SpatialGraph,
}

impl SpatialGraphResource {
    pub fn new(graph: SpatialGraph) -> Self {
        Self { graph }
    }
}

/// Component marking an entity's location in the spatial graph.
#[derive(Component, Clone, Debug)]
pub struct SpatialLocation {
    pub node: NodeId,
}

impl SpatialLocation {
    pub fn new(node: impl Into<String>) -> Self {
        Self { node: node.into() }
    }
}

/// Message requesting a spatial query.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct SpatialQueryRequest {
    pub query: SpatialQuery,
}

impl SpatialQueryRequest {
    pub fn new(query: SpatialQuery) -> Self {
        Self { query }
    }

    pub fn neighbors(node: impl Into<String>) -> Self {
        Self {
            query: SpatialQuery::Neighbors { node: node.into() },
        }
    }

    pub fn distance(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            query: SpatialQuery::Distance {
                from: from.into(),
                to: to.into(),
            },
        }
    }

    pub fn can_move(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            query: SpatialQuery::CanMove {
                from: from.into(),
                to: to.into(),
            },
        }
    }
}

/// Message containing spatial query results.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct SpatialQueryResult {
    pub event: SpatialEvent,
}

impl SpatialQueryResult {
    pub fn new(event: SpatialEvent) -> Self {
        Self { event }
    }
}

/// Message requesting entity movement.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct MoveEntityRequest {
    pub entity: Entity,
    pub from: Option<NodeId>,
    pub to: NodeId,
}

impl MoveEntityRequest {
    pub fn new(entity: Entity, from: Option<NodeId>, to: impl Into<String>) -> Self {
        Self {
            entity,
            from,
            to: to.into(),
        }
    }

    pub fn spawn(entity: Entity, to: impl Into<String>) -> Self {
        Self::new(entity, None, to)
    }

    pub fn move_to(entity: Entity, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::new(entity, Some(from.into()), to)
    }
}
