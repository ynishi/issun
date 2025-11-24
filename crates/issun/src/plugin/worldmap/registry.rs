//! World map registry (static definitions)

use super::types::*;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// World map registry (static definitions)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorldMapRegistry {
    /// All locations on the map
    locations: HashMap<LocationId, Location>,

    /// All routes between locations
    routes: HashMap<RouteId, Route>,

    /// Adjacency list for pathfinding (location_id -> connected routes)
    #[serde(skip)]
    adjacency: HashMap<LocationId, Vec<RouteId>>,
}

impl Resource for WorldMapRegistry {}

impl WorldMapRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new location
    ///
    /// # Arguments
    ///
    /// * `location` - Location to register
    ///
    /// # Returns
    ///
    /// Location ID
    pub fn register_location(&mut self, location: Location) -> LocationId {
        let id = location.id.clone();
        self.locations.insert(id.clone(), location);
        id
    }

    /// Register a new route
    ///
    /// # Arguments
    ///
    /// * `route` - Route to register
    ///
    /// # Returns
    ///
    /// Route ID
    pub fn register_route(&mut self, route: Route) -> RouteId {
        let id = route.id.clone();

        // Update adjacency list
        self.adjacency
            .entry(route.from.clone())
            .or_default()
            .push(id.clone());

        // Bidirectional route
        if route.bidirectional {
            self.adjacency
                .entry(route.to.clone())
                .or_default()
                .push(id.clone());
        }

        self.routes.insert(id.clone(), route);
        id
    }

    /// Get location by ID
    pub fn get_location(&self, id: &LocationId) -> Option<&Location> {
        self.locations.get(id)
    }

    /// Get mutable location by ID
    pub fn get_location_mut(&mut self, id: &LocationId) -> Option<&mut Location> {
        self.locations.get_mut(id)
    }

    /// Get route by ID
    pub fn get_route(&self, id: &RouteId) -> Option<&Route> {
        self.routes.get(id)
    }

    /// Get mutable route by ID
    pub fn get_route_mut(&mut self, id: &RouteId) -> Option<&mut Route> {
        self.routes.get_mut(id)
    }

    /// Get all routes from a location
    pub fn get_routes_from(&self, location_id: &LocationId) -> Vec<&Route> {
        if let Some(route_ids) = self.adjacency.get(location_id) {
            route_ids
                .iter()
                .filter_map(|id| self.routes.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get all locations
    pub fn locations(&self) -> &HashMap<LocationId, Location> {
        &self.locations
    }

    /// Get all routes
    pub fn routes(&self) -> &HashMap<RouteId, Route> {
        &self.routes
    }

    /// Calculate distance between two locations
    pub fn calculate_distance(&self, from: &LocationId, to: &LocationId) -> Option<f32> {
        let from_loc = self.get_location(from)?;
        let to_loc = self.get_location(to)?;
        Some(from_loc.position.distance_to(&to_loc.position))
    }

    /// Check if route exists between locations
    pub fn has_route(&self, from: &LocationId, to: &LocationId) -> bool {
        self.get_routes_from(from)
            .iter()
            .any(|r| &r.to == to || (r.bidirectional && &r.from == to))
    }

    /// Remove a location
    ///
    /// # Arguments
    ///
    /// * `id` - Location ID to remove
    ///
    /// # Returns
    ///
    /// Removed location, or None if not found
    pub fn remove_location(&mut self, id: &LocationId) -> Option<Location> {
        // Remove from adjacency list
        self.adjacency.remove(id);

        // Remove location
        self.locations.remove(id)
    }

    /// Remove a route
    ///
    /// # Arguments
    ///
    /// * `id` - Route ID to remove
    ///
    /// # Returns
    ///
    /// Removed route, or None if not found
    pub fn remove_route(&mut self, id: &RouteId) -> Option<Route> {
        if let Some(route) = self.routes.remove(id) {
            // Remove from adjacency list
            if let Some(routes) = self.adjacency.get_mut(&route.from) {
                routes.retain(|rid| rid != id);
            }

            if route.bidirectional {
                if let Some(routes) = self.adjacency.get_mut(&route.to) {
                    routes.retain(|rid| rid != id);
                }
            }

            Some(route)
        } else {
            None
        }
    }

    /// Rebuild adjacency list (call after deserialization)
    pub fn rebuild_adjacency(&mut self) {
        self.adjacency.clear();

        for (route_id, route) in &self.routes {
            self.adjacency
                .entry(route.from.clone())
                .or_default()
                .push(route_id.clone());

            if route.bidirectional {
                self.adjacency
                    .entry(route.to.clone())
                    .or_default()
                    .push(route_id.clone());
            }
        }
    }

    /// Get number of locations
    pub fn location_count(&self) -> usize {
        self.locations.len()
    }

    /// Get number of routes
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new() {
        let registry = WorldMapRegistry::new();
        assert_eq!(registry.location_count(), 0);
        assert_eq!(registry.route_count(), 0);
    }

    #[test]
    fn test_register_location() {
        let mut registry = WorldMapRegistry::new();

        let location =
            Location::new("city_1", "Test City").with_position(Position::new(10.0, 20.0));

        let id = registry.register_location(location);

        assert_eq!(id, "city_1");
        assert_eq!(registry.location_count(), 1);
        assert!(registry.get_location(&id).is_some());
    }

    #[test]
    fn test_register_route() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(Location::new("city_a", "City A"));
        registry.register_location(Location::new("city_b", "City B"));

        let route = Route::new("route_1", "city_a", "city_b");
        let id = registry.register_route(route);

        assert_eq!(id, "route_1");
        assert_eq!(registry.route_count(), 1);
        assert!(registry.get_route(&id).is_some());
    }

    #[test]
    fn test_get_routes_from() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(Location::new("city_a", "City A"));
        registry.register_location(Location::new("city_b", "City B"));
        registry.register_location(Location::new("city_c", "City C"));

        registry.register_route(Route::new("route_1", "city_a", "city_b"));
        registry.register_route(Route::new("route_2", "city_a", "city_c"));

        let routes = registry.get_routes_from(&"city_a".to_string());
        assert_eq!(routes.len(), 2);
    }

    #[test]
    fn test_calculate_distance() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(
            Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_b", "City B").with_position(Position::new(3.0, 4.0)),
        );

        let distance = registry
            .calculate_distance(&"city_a".to_string(), &"city_b".to_string())
            .unwrap();

        assert_eq!(distance, 5.0);
    }

    #[test]
    fn test_has_route() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(Location::new("city_a", "City A"));
        registry.register_location(Location::new("city_b", "City B"));
        registry.register_location(Location::new("city_c", "City C"));

        registry.register_route(Route::new("route_1", "city_a", "city_b"));

        assert!(registry.has_route(&"city_a".to_string(), &"city_b".to_string()));
        assert!(registry.has_route(&"city_b".to_string(), &"city_a".to_string())); // Bidirectional
        assert!(!registry.has_route(&"city_a".to_string(), &"city_c".to_string()));
    }

    #[test]
    fn test_remove_location() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(Location::new("city_a", "City A"));
        assert_eq!(registry.location_count(), 1);

        let removed = registry.remove_location(&"city_a".to_string());
        assert!(removed.is_some());
        assert_eq!(registry.location_count(), 0);
    }

    #[test]
    fn test_remove_route() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(Location::new("city_a", "City A"));
        registry.register_location(Location::new("city_b", "City B"));
        registry.register_route(Route::new("route_1", "city_a", "city_b"));

        assert_eq!(registry.route_count(), 1);

        let removed = registry.remove_route(&"route_1".to_string());
        assert!(removed.is_some());
        assert_eq!(registry.route_count(), 0);
    }

    #[test]
    fn test_rebuild_adjacency() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(Location::new("city_a", "City A"));
        registry.register_location(Location::new("city_b", "City B"));
        registry.register_route(Route::new("route_1", "city_a", "city_b"));

        // Clear adjacency
        registry.adjacency.clear();
        assert_eq!(registry.get_routes_from(&"city_a".to_string()).len(), 0);

        // Rebuild
        registry.rebuild_adjacency();
        assert_eq!(registry.get_routes_from(&"city_a".to_string()).len(), 1);
    }
}
