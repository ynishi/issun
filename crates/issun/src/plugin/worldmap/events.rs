//! Events for WorldMapPlugin

use super::state::EntityPosition;
use super::types::*;
use serde::{Deserialize, Serialize};

// ==================== Command Events (Requests) ====================

/// Request to start travel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelStartRequested {
    pub entity_id: EntityId,
    pub from: LocationId,
    pub to: LocationId,
}

/// Request to cancel travel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelCancelRequested {
    pub entity_id: EntityId,
}

/// Request to pause travel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelPauseRequested {
    pub entity_id: EntityId,
}

/// Request to resume travel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelResumeRequested {
    pub entity_id: EntityId,
}

/// Request to teleport entity to a location
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityTeleportRequested {
    pub entity_id: EntityId,
    pub location_id: LocationId,
}

/// Request to discover a location (for fog of war)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationDiscoverRequested {
    pub entity_id: EntityId,
    pub location_id: LocationId,
}

// ==================== State Events (Notifications) ====================

/// Travel started
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelStartedEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub from: LocationId,
    pub to: LocationId,
    pub estimated_duration: f32,
}

/// Travel progress update
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelProgressEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub progress: f32,
    pub current_position: Position,
}

/// Travel completed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelCompletedEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub destination: LocationId,
}

/// Travel cancelled
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelCancelledEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub progress: f32,
}

/// Travel paused
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelPausedEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub progress: f32,
}

/// Travel resumed
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TravelResumedEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub progress: f32,
}

/// Encounter triggered during travel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterTriggeredEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub encounter: Encounter,
}

/// Encounter resolved
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EncounterResolvedEvent {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub encounter_id: EncounterId,
    pub success: bool,
}

/// Location discovered (fog of war)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LocationDiscoveredEvent {
    pub entity_id: EntityId,
    pub location_id: LocationId,
}

/// Entity position changed (teleport or arrival)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EntityPositionChangedEvent {
    pub entity_id: EntityId,
    pub old_position: Option<EntityPosition>,
    pub new_position: EntityPosition,
}
