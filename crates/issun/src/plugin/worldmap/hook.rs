//! Hook for game-specific world map behavior

use super::types::*;
use async_trait::async_trait;

/// Hook for game-specific world map behavior
#[async_trait]
pub trait WorldMapHook: Send + Sync {
    /// Called when travel starts
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity starting travel
    /// * `travel` - Travel instance
    async fn on_travel_started(&self, _entity_id: &str, _travel: &Travel) {}

    /// Called when travel completes
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity that completed travel
    /// * `location_id` - Destination location
    async fn on_travel_completed(&self, _entity_id: &str, _location_id: &str) {}

    /// Called when travel is cancelled
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity that cancelled travel
    /// * `travel` - Cancelled travel instance
    async fn on_travel_cancelled(&self, _entity_id: &str, _travel: &Travel) {}

    /// Called when travel is paused
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity that paused travel
    /// * `travel` - Paused travel instance
    async fn on_travel_paused(&self, _entity_id: &str, _travel: &Travel) {}

    /// Called when travel is resumed
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity that resumed travel
    /// * `travel` - Resumed travel instance
    async fn on_travel_resumed(&self, _entity_id: &str, _travel: &Travel) {}

    /// Generate random encounter during travel (return None for no encounter)
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity traveling
    /// * `travel` - Current travel state
    /// * `route` - Route being traveled
    ///
    /// # Returns
    ///
    /// Optional encounter, or None if no encounter should occur
    async fn generate_encounter(
        &self,
        _entity_id: &str,
        _travel: &Travel,
        _route: &Route,
    ) -> Option<Encounter> {
        None
    }

    /// Calculate custom travel speed modifier (default 1.0)
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity traveling
    /// * `route` - Route being traveled
    ///
    /// # Returns
    ///
    /// Speed multiplier (1.0 = normal)
    async fn calculate_speed_modifier(&self, _entity_id: &str, _route: &Route) -> f32 {
        1.0
    }

    /// Validate if entity can start travel (check resources, permissions, etc.)
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity requesting travel
    /// * `from` - Starting location
    /// * `to` - Destination location
    ///
    /// # Returns
    ///
    /// Ok(()) if travel is allowed, Err(reason) otherwise
    async fn can_start_travel(
        &self,
        _entity_id: &str,
        _from: &str,
        _to: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called when location is discovered (fog of war)
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity that discovered the location
    /// * `location_id` - Discovered location
    async fn on_location_discovered(&self, _entity_id: &str, _location_id: &str) {}

    /// Calculate travel cost (for economy integration)
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity traveling
    /// * `route` - Route being traveled
    /// * `distance` - Travel distance
    ///
    /// # Returns
    ///
    /// Travel cost (currency or resource units)
    async fn calculate_travel_cost(&self, _entity_id: &str, _route: &Route, _distance: f32) -> f32 {
        0.0
    }
}

/// Default no-op hook
pub struct DefaultWorldMapHook;

#[async_trait]
impl WorldMapHook for DefaultWorldMapHook {}
