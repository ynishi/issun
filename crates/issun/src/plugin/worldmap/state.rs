//! World map runtime state

use super::types::*;
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// World map runtime state
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorldMapState {
    /// Active travels (entity_id -> travel)
    travels: HashMap<EntityId, Travel>,

    /// Entity positions (entity_id -> location_id for static, or travel_id for in-transit)
    positions: HashMap<EntityId, EntityPosition>,

    /// Discovered locations (for fog of war)
    discovered_locations: HashSet<LocationId>,

    /// Travel history (entity_id -> visited locations)
    travel_history: HashMap<EntityId, Vec<LocationId>>,
}

impl State for WorldMapState {}

impl WorldMapState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new travel
    ///
    /// # Arguments
    ///
    /// * `travel` - Travel instance to start
    ///
    /// # Returns
    ///
    /// Travel ID
    pub fn start_travel(&mut self, travel: Travel) -> TravelId {
        let entity_id = travel.entity_id.clone();
        let travel_id = travel.id.clone();

        // Update position to "traveling"
        self.positions.insert(
            entity_id.clone(),
            EntityPosition::Traveling(travel_id.clone()),
        );

        self.travels.insert(entity_id, travel);
        travel_id
    }

    /// Update all active travels
    ///
    /// # Arguments
    ///
    /// * `delta_time` - Time elapsed since last update (seconds)
    ///
    /// # Returns
    ///
    /// Vector of completed travels
    pub fn update_travels(&mut self, delta_time: f32) -> Vec<TravelCompleted> {
        let mut completed = Vec::new();

        for (entity_id, travel) in &mut self.travels {
            if travel.status == TravelStatus::InProgress {
                travel.update(delta_time);

                if travel.status == TravelStatus::Arrived {
                    completed.push(TravelCompleted {
                        entity_id: entity_id.clone(),
                        travel_id: travel.id.clone(),
                        destination: travel.to.clone(),
                    });
                }
            }
        }

        // Update positions for completed travels
        for complete in &completed {
            self.positions.insert(
                complete.entity_id.clone(),
                EntityPosition::AtLocation(complete.destination.clone()),
            );

            // Add to travel history
            self.travel_history
                .entry(complete.entity_id.clone())
                .or_default()
                .push(complete.destination.clone());

            // Discover location
            self.discovered_locations
                .insert(complete.destination.clone());

            // Remove completed travel
            self.travels.remove(&complete.entity_id);
        }

        completed
    }

    /// Get entity position
    pub fn get_position(&self, entity_id: &EntityId) -> Option<&EntityPosition> {
        self.positions.get(entity_id)
    }

    /// Set entity position (teleport)
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity to position
    /// * `location_id` - Location to place entity at
    pub fn set_position(&mut self, entity_id: impl Into<String>, location_id: impl Into<String>) {
        let entity = entity_id.into();
        let location = location_id.into();

        self.positions
            .insert(entity.clone(), EntityPosition::AtLocation(location.clone()));

        self.discovered_locations.insert(location.clone());
        self.travel_history
            .entry(entity)
            .or_default()
            .push(location);
    }

    /// Get active travel for entity
    pub fn get_travel(&self, entity_id: &EntityId) -> Option<&Travel> {
        self.travels.get(entity_id)
    }

    /// Get mutable travel
    pub fn get_travel_mut(&mut self, entity_id: &EntityId) -> Option<&mut Travel> {
        self.travels.get_mut(entity_id)
    }

    /// Get all active travels
    pub fn travels(&self) -> &HashMap<EntityId, Travel> {
        &self.travels
    }

    /// Cancel travel
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity to cancel travel for
    ///
    /// # Returns
    ///
    /// Cancelled travel, or None if not traveling
    pub fn cancel_travel(&mut self, entity_id: &EntityId) -> Option<Travel> {
        if let Some(mut travel) = self.travels.remove(entity_id) {
            travel.status = TravelStatus::Cancelled;

            // Return to starting location
            self.positions.insert(
                entity_id.clone(),
                EntityPosition::AtLocation(travel.from.clone()),
            );

            Some(travel)
        } else {
            None
        }
    }

    /// Pause travel
    pub fn pause_travel(&mut self, entity_id: &EntityId) -> bool {
        if let Some(travel) = self.travels.get_mut(entity_id) {
            travel.pause();
            true
        } else {
            false
        }
    }

    /// Resume travel
    pub fn resume_travel(&mut self, entity_id: &EntityId) -> bool {
        if let Some(travel) = self.travels.get_mut(entity_id) {
            travel.resume();
            true
        } else {
            false
        }
    }

    /// Check if location is discovered
    pub fn is_discovered(&self, location_id: &LocationId) -> bool {
        self.discovered_locations.contains(location_id)
    }

    /// Discover a location (for fog of war)
    pub fn discover_location(&mut self, location_id: impl Into<String>) {
        self.discovered_locations.insert(location_id.into());
    }

    /// Get discovered locations
    pub fn discovered_locations(&self) -> &HashSet<LocationId> {
        &self.discovered_locations
    }

    /// Get travel history for entity
    pub fn get_history(&self, entity_id: &EntityId) -> Option<&Vec<LocationId>> {
        self.travel_history.get(entity_id)
    }

    /// Clear all state
    pub fn clear(&mut self) {
        self.travels.clear();
        self.positions.clear();
        self.discovered_locations.clear();
        self.travel_history.clear();
    }

    /// Get number of active travels
    pub fn active_travel_count(&self) -> usize {
        self.travels.len()
    }
}

/// Entity position (static or traveling)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityPosition {
    /// At a specific location
    AtLocation(LocationId),

    /// Currently traveling
    Traveling(TravelId),
}

/// Travel completion result
#[derive(Clone, Debug)]
pub struct TravelCompleted {
    pub entity_id: EntityId,
    pub travel_id: TravelId,
    pub destination: LocationId,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = WorldMapState::new();
        assert_eq!(state.active_travel_count(), 0);
        assert_eq!(state.discovered_locations().len(), 0);
    }

    #[test]
    fn test_start_travel() {
        let mut state = WorldMapState::new();

        let travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0, 10.0,
        );

        let travel_id = state.start_travel(travel);

        assert_eq!(travel_id, "travel_1");
        assert_eq!(state.active_travel_count(), 1);
        assert!(state.get_travel(&"player_1".to_string()).is_some());
    }

    #[test]
    fn test_update_travels() {
        let mut state = WorldMapState::new();

        let travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0,
            10.0, // 10 seconds total
        );

        state.start_travel(travel);

        // Update 5 seconds
        let completed = state.update_travels(5.0);
        assert!(completed.is_empty());
        assert_eq!(state.active_travel_count(), 1);

        // Complete travel
        let completed = state.update_travels(5.0);
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].entity_id, "player_1");
        assert_eq!(completed[0].destination, "city_b");
        assert_eq!(state.active_travel_count(), 0);
    }

    #[test]
    fn test_set_position() {
        let mut state = WorldMapState::new();

        state.set_position("player_1", "city_a");

        let position = state.get_position(&"player_1".to_string()).unwrap();
        assert_eq!(position, &EntityPosition::AtLocation("city_a".to_string()));

        assert!(state.is_discovered(&"city_a".to_string()));
    }

    #[test]
    fn test_cancel_travel() {
        let mut state = WorldMapState::new();

        let travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0, 10.0,
        );

        state.start_travel(travel);
        assert_eq!(state.active_travel_count(), 1);

        let cancelled = state.cancel_travel(&"player_1".to_string());
        assert!(cancelled.is_some());
        assert_eq!(cancelled.unwrap().status, TravelStatus::Cancelled);
        assert_eq!(state.active_travel_count(), 0);

        // Should be back at starting location
        let position = state.get_position(&"player_1".to_string()).unwrap();
        assert_eq!(position, &EntityPosition::AtLocation("city_a".to_string()));
    }

    #[test]
    fn test_pause_resume_travel() {
        let mut state = WorldMapState::new();

        let travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0, 10.0,
        );

        state.start_travel(travel);

        // Pause
        assert!(state.pause_travel(&"player_1".to_string()));
        let travel = state.get_travel(&"player_1".to_string()).unwrap();
        assert_eq!(travel.status, TravelStatus::Paused);

        // Resume
        assert!(state.resume_travel(&"player_1".to_string()));
        let travel = state.get_travel(&"player_1".to_string()).unwrap();
        assert_eq!(travel.status, TravelStatus::InProgress);
    }

    #[test]
    fn test_travel_history() {
        let mut state = WorldMapState::new();

        state.set_position("player_1", "city_a");

        let travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0, 10.0,
        );

        state.start_travel(travel);
        state.update_travels(10.0); // Complete travel

        let history = state.get_history(&"player_1".to_string()).unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], "city_a");
        assert_eq!(history[1], "city_b");
    }

    #[test]
    fn test_discover_location() {
        let mut state = WorldMapState::new();

        assert!(!state.is_discovered(&"city_a".to_string()));

        state.discover_location("city_a");

        assert!(state.is_discovered(&"city_a".to_string()));
        assert_eq!(state.discovered_locations().len(), 1);
    }

    #[test]
    fn test_clear() {
        let mut state = WorldMapState::new();

        state.set_position("player_1", "city_a");
        state.discover_location("city_b");

        assert!(!state.positions.is_empty());
        assert!(!state.discovered_locations.is_empty());

        state.clear();

        assert!(state.positions.is_empty());
        assert!(state.discovered_locations.is_empty());
        assert_eq!(state.active_travel_count(), 0);
    }
}
