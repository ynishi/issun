//! World map system for orchestrating travel

use super::config::WorldMapConfig;
use super::hook::WorldMapHook;
use super::registry::WorldMapRegistry;
use super::service::WorldMapService;
use super::state::WorldMapState;
use super::types::*;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// World map system (event-driven travel orchestration)
#[derive(Clone)]
#[allow(dead_code)]
pub struct WorldMapSystem {
    hook: Arc<dyn WorldMapHook>,
    service: WorldMapService,
}

#[async_trait]
impl System for WorldMapSystem {
    fn name(&self) -> &'static str {
        "issun:worldmap_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl WorldMapSystem {
    /// Create a new world map system
    pub fn new(hook: Arc<dyn WorldMapHook>) -> Self {
        Self {
            hook,
            service: WorldMapService,
        }
    }

    /// Update all active travels
    ///
    /// # Arguments
    ///
    /// * `state` - World map state
    /// * `registry` - World map registry
    /// * `config` - World map configuration
    /// * `delta_time` - Time elapsed since last update (seconds)
    ///
    /// # Returns
    ///
    /// Vector of travel events
    pub async fn update(
        &mut self,
        state: &mut WorldMapState,
        registry: &WorldMapRegistry,
        config: &WorldMapConfig,
        delta_time: f32,
    ) -> Vec<TravelEvent> {
        let mut events = Vec::new();

        // Update all travels
        let completed = state.update_travels(delta_time);

        // Process completions
        for complete in completed {
            // Notify hook
            self.hook
                .on_travel_completed(&complete.entity_id, &complete.destination)
                .await;

            events.push(TravelEvent::Arrived {
                entity_id: complete.entity_id.clone(),
                location_id: complete.destination.clone(),
            });

            // Discover location (if fog of war enabled)
            if config.enable_fog_of_war {
                events.push(TravelEvent::LocationDiscovered {
                    entity_id: complete.entity_id,
                    location_id: complete.destination,
                });
            }
        }

        // Check for encounters
        let active_travels: Vec<(EntityId, Travel)> = state
            .travels()
            .iter()
            .map(|(id, travel)| (id.clone(), travel.clone()))
            .collect();

        for (entity_id, travel) in active_travels {
            if travel.status != TravelStatus::InProgress {
                continue;
            }

            if let Some(route) = registry.get_route(&travel.route_id) {
                // Check if encounter should trigger
                if self.should_check_encounter(&travel, route, config) {
                    if let Some(encounter) = self
                        .hook
                        .generate_encounter(&entity_id, &travel, route)
                        .await
                    {
                        // Add encounter to travel
                        if let Some(t) = state.get_travel_mut(&entity_id) {
                            t.encounters.push(encounter.id.clone());
                            t.status = TravelStatus::Interrupted;
                        }

                        events.push(TravelEvent::EncounterTriggered {
                            entity_id,
                            encounter,
                        });
                    }
                }
            }
        }

        events
    }

    /// Start a new travel
    ///
    /// # Arguments
    ///
    /// * `state` - World map state
    /// * `registry` - World map registry
    /// * `config` - World map configuration
    /// * `entity_id` - Entity to travel
    /// * `from` - Starting location
    /// * `to` - Destination location
    ///
    /// # Returns
    ///
    /// Travel ID, or error
    pub async fn start_travel(
        &mut self,
        state: &mut WorldMapState,
        registry: &WorldMapRegistry,
        config: &WorldMapConfig,
        entity_id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Result<TravelId, TravelError> {
        let entity_id = entity_id.into();
        let from = from.into();
        let to = to.into();

        // Validate with hook
        self.hook
            .can_start_travel(&entity_id, &from, &to)
            .await
            .map_err(|_| TravelError::ValidationFailed)?;

        // Validate locations exist
        let from_loc = registry
            .get_location(&from)
            .ok_or(TravelError::LocationNotFound)?;
        let to_loc = registry
            .get_location(&to)
            .ok_or(TravelError::LocationNotFound)?;

        // Find direct route
        let routes = registry.get_routes_from(&from);
        let route = routes
            .iter()
            .find(|r| r.to == to || (r.bidirectional && r.from == to))
            .ok_or(TravelError::NoRouteExists)?;

        // Calculate distance
        let distance = route
            .distance
            .unwrap_or_else(|| from_loc.position.distance_to(&to_loc.position));

        // Calculate speed
        let base_speed = config.default_travel_speed;
        let terrain_multiplier = if config.enable_terrain_modifiers {
            route.terrain.speed_multiplier()
        } else {
            1.0
        };
        let hook_multiplier = self.hook.calculate_speed_modifier(&entity_id, route).await;
        let speed = base_speed * terrain_multiplier * hook_multiplier;

        if speed <= 0.0 {
            return Err(TravelError::InvalidSpeed);
        }

        // Create travel
        let travel_id = format!(
            "travel_{}_{}",
            entity_id,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        let travel = Travel::new(
            travel_id.clone(),
            entity_id.clone(),
            route.id.clone(),
            from,
            to,
            distance,
            speed,
        );

        // Notify hook
        self.hook.on_travel_started(&entity_id, &travel).await;

        // Register travel
        state.start_travel(travel);

        Ok(travel_id)
    }

    /// Cancel travel
    ///
    /// # Arguments
    ///
    /// * `state` - World map state
    /// * `entity_id` - Entity to cancel travel for
    ///
    /// # Returns
    ///
    /// Cancelled travel, or None if not traveling
    pub async fn cancel_travel(
        &mut self,
        state: &mut WorldMapState,
        entity_id: &str,
    ) -> Option<Travel> {
        if let Some(travel) = state.cancel_travel(&entity_id.to_string()) {
            // Notify hook
            self.hook.on_travel_cancelled(entity_id, &travel).await;
            Some(travel)
        } else {
            None
        }
    }

    /// Pause travel
    ///
    /// # Arguments
    ///
    /// * `state` - World map state
    /// * `entity_id` - Entity to pause travel for
    ///
    /// # Returns
    ///
    /// True if paused, false if not traveling
    pub async fn pause_travel(&mut self, state: &mut WorldMapState, entity_id: &str) -> bool {
        if state.pause_travel(&entity_id.to_string()) {
            if let Some(travel) = state.get_travel(&entity_id.to_string()) {
                self.hook.on_travel_paused(entity_id, travel).await;
            }
            true
        } else {
            false
        }
    }

    /// Resume travel
    ///
    /// # Arguments
    ///
    /// * `state` - World map state
    /// * `entity_id` - Entity to resume travel for
    ///
    /// # Returns
    ///
    /// True if resumed, false if not paused
    pub async fn resume_travel(&mut self, state: &mut WorldMapState, entity_id: &str) -> bool {
        if state.resume_travel(&entity_id.to_string()) {
            if let Some(travel) = state.get_travel(&entity_id.to_string()) {
                self.hook.on_travel_resumed(entity_id, travel).await;
            }
            true
        } else {
            false
        }
    }

    /// Teleport entity to a location
    ///
    /// # Arguments
    ///
    /// * `state` - World map state
    /// * `entity_id` - Entity to teleport
    /// * `location_id` - Destination location
    pub fn teleport(
        &mut self,
        state: &mut WorldMapState,
        entity_id: impl Into<String>,
        location_id: impl Into<String>,
    ) {
        state.set_position(entity_id, location_id);
    }

    /// Check if encounter should be evaluated
    fn should_check_encounter(
        &self,
        travel: &Travel,
        route: &Route,
        config: &WorldMapConfig,
    ) -> bool {
        // Don't check if too early in journey
        if travel.progress < 0.1 {
            return false;
        }

        // Don't check if route is safe
        if route.danger_level <= 0.0 {
            return false;
        }

        // Simple probability check
        // In a real implementation, this would check elapsed time since last encounter
        let encounter_chance = WorldMapService::calculate_encounter_chance(
            travel.progress,
            route.danger_level,
            config.base_encounter_rate,
            config.danger_multiplier,
        );

        // Simple random check (game should inject RNG via Hook)
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < encounter_chance * 0.01 // Reduce frequency for testing
    }
}

/// Travel events
#[derive(Clone, Debug)]
pub enum TravelEvent {
    Arrived {
        entity_id: EntityId,
        location_id: LocationId,
    },
    EncounterTriggered {
        entity_id: EntityId,
        encounter: Encounter,
    },
    LocationDiscovered {
        entity_id: EntityId,
        location_id: LocationId,
    },
}

/// Travel errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TravelError {
    LocationNotFound,
    NoRouteExists,
    InvalidSpeed,
    ValidationFailed,
}

impl std::fmt::Display for TravelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TravelError::LocationNotFound => write!(f, "Location not found"),
            TravelError::NoRouteExists => write!(f, "No route exists between locations"),
            TravelError::InvalidSpeed => write!(f, "Invalid travel speed"),
            TravelError::ValidationFailed => write!(f, "Travel validation failed"),
        }
    }
}

impl std::error::Error for TravelError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::worldmap::hook::DefaultWorldMapHook;

    #[tokio::test]
    async fn test_start_travel() {
        let mut registry = WorldMapRegistry::new();
        let mut state = WorldMapState::new();
        let config = WorldMapConfig::default();

        // Setup map
        registry.register_location(
            Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_b", "City B").with_position(Position::new(100.0, 0.0)),
        );
        registry.register_route(Route::new("route_1", "city_a", "city_b"));

        let hook = Arc::new(DefaultWorldMapHook);
        let mut system = WorldMapSystem::new(hook);

        let travel_id = system
            .start_travel(
                &mut state, &registry, &config, "player_1", "city_a", "city_b",
            )
            .await
            .unwrap();

        assert!(!travel_id.is_empty());
        assert_eq!(state.active_travel_count(), 1);
    }

    #[tokio::test]
    async fn test_start_travel_no_route() {
        let mut registry = WorldMapRegistry::new();
        let mut state = WorldMapState::new();
        let config = WorldMapConfig::default();

        registry.register_location(Location::new("city_a", "City A"));
        registry.register_location(Location::new("city_b", "City B"));

        let hook = Arc::new(DefaultWorldMapHook);
        let mut system = WorldMapSystem::new(hook);

        let result = system
            .start_travel(
                &mut state, &registry, &config, "player_1", "city_a", "city_b",
            )
            .await;

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), TravelError::NoRouteExists);
    }

    #[tokio::test]
    async fn test_cancel_travel() {
        let mut registry = WorldMapRegistry::new();
        let mut state = WorldMapState::new();
        let config = WorldMapConfig::default();

        registry.register_location(
            Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_b", "City B").with_position(Position::new(100.0, 0.0)),
        );
        registry.register_route(Route::new("route_1", "city_a", "city_b"));

        let hook = Arc::new(DefaultWorldMapHook);
        let mut system = WorldMapSystem::new(hook);

        system
            .start_travel(
                &mut state, &registry, &config, "player_1", "city_a", "city_b",
            )
            .await
            .unwrap();

        let cancelled = system.cancel_travel(&mut state, "player_1").await;
        assert!(cancelled.is_some());
        assert_eq!(state.active_travel_count(), 0);
    }

    #[tokio::test]
    async fn test_update_completes_travel() {
        let mut registry = WorldMapRegistry::new();
        let mut state = WorldMapState::new();
        let config = WorldMapConfig::default();

        registry.register_location(
            Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_b", "City B").with_position(Position::new(100.0, 0.0)),
        );
        registry.register_route(Route::new("route_1", "city_a", "city_b"));

        let hook = Arc::new(DefaultWorldMapHook);
        let mut system = WorldMapSystem::new(hook);

        system
            .start_travel(
                &mut state, &registry, &config, "player_1", "city_a", "city_b",
            )
            .await
            .unwrap();

        // Update enough to complete travel (100 distance / 10 speed = 10 seconds)
        let events = system.update(&mut state, &registry, &config, 10.0).await;

        assert_eq!(state.active_travel_count(), 0);
        assert!(!events.is_empty());
    }
}
