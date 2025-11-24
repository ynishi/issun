//! Pure logic service for world map calculations

use super::registry::WorldMapRegistry;
use super::types::*;
use crate::service::Service;
use async_trait::async_trait;
use std::any::Any;
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Pure world map calculation service
#[derive(Clone, Default)]
pub struct WorldMapService;

#[async_trait]
impl Service for WorldMapService {
    fn name(&self) -> &'static str {
        "issun:worldmap_service"
    }

    fn clone_box(&self) -> Box<dyn Service> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl WorldMapService {
    /// Calculate travel duration
    ///
    /// # Arguments
    ///
    /// * `distance` - Distance to travel
    /// * `speed` - Base travel speed
    /// * `terrain_multiplier` - Terrain speed modifier
    ///
    /// # Returns
    ///
    /// Travel duration in seconds
    pub fn calculate_duration(distance: f32, speed: f32, terrain_multiplier: f32) -> f32 {
        if speed * terrain_multiplier > 0.0 {
            distance / (speed * terrain_multiplier)
        } else {
            f32::MAX
        }
    }

    /// Find shortest path between two locations (Dijkstra's algorithm)
    ///
    /// # Arguments
    ///
    /// * `from` - Starting location
    /// * `to` - Destination location
    /// * `registry` - World map registry
    ///
    /// # Returns
    ///
    /// Vector of route IDs representing the path, or None if no path exists
    pub fn find_path(
        from: &LocationId,
        to: &LocationId,
        registry: &WorldMapRegistry,
    ) -> Option<Vec<RouteId>> {
        // Early exit if from == to
        if from == to {
            return Some(Vec::new());
        }

        // Priority queue for Dijkstra (distance, location_id)
        let mut queue: BinaryHeap<PathNode> = BinaryHeap::new();

        // Distance from start
        let mut distances: HashMap<LocationId, f32> = HashMap::new();

        // Previous route in optimal path
        let mut previous: HashMap<LocationId, RouteId> = HashMap::new();

        // Visited set
        let mut visited: HashSet<LocationId> = HashSet::new();

        // Initialize
        distances.insert(from.clone(), 0.0);
        queue.push(PathNode {
            distance: 0.0,
            location_id: from.clone(),
        });

        while let Some(PathNode {
            distance: current_dist,
            location_id: current,
        }) = queue.pop()
        {
            // Skip if already visited
            if visited.contains(&current) {
                continue;
            }

            // Mark as visited
            visited.insert(current.clone());

            // Found destination
            if &current == to {
                break;
            }

            // Check all neighbors
            let routes = registry.get_routes_from(&current);
            for route in routes {
                // Determine neighbor
                let neighbor = if route.from == current {
                    &route.to
                } else if route.bidirectional {
                    &route.from
                } else {
                    continue;
                };

                // Skip if visited
                if visited.contains(neighbor) {
                    continue;
                }

                // Calculate distance
                let edge_dist = if let Some(explicit) = route.distance {
                    explicit
                } else if let (Some(from_loc), Some(to_loc)) = (
                    registry.get_location(&route.from),
                    registry.get_location(&route.to),
                ) {
                    from_loc.position.distance_to(&to_loc.position)
                } else {
                    continue;
                };

                let new_dist = current_dist + edge_dist;
                let neighbor_dist = *distances.get(neighbor).unwrap_or(&f32::MAX);

                // Update if shorter path found
                if new_dist < neighbor_dist {
                    distances.insert(neighbor.clone(), new_dist);
                    previous.insert(neighbor.clone(), route.id.clone());
                    queue.push(PathNode {
                        distance: new_dist,
                        location_id: neighbor.clone(),
                    });
                }
            }
        }

        // Reconstruct path
        if !previous.contains_key(to) {
            return None; // No path found
        }

        let mut path = Vec::new();
        let mut current = to.clone();

        while let Some(route_id) = previous.get(&current) {
            path.push(route_id.clone());

            let route = registry.get_route(route_id)?;
            current = if route.to == current {
                route.from.clone()
            } else {
                route.to.clone()
            };

            if &current == from {
                break;
            }
        }

        path.reverse();
        Some(path)
    }

    /// Calculate encounter probability based on progress
    ///
    /// # Arguments
    ///
    /// * `progress` - Current travel progress (0.0 - 1.0)
    /// * `danger_level` - Route danger level (0.0 - 1.0)
    /// * `base_rate` - Base encounter rate
    /// * `danger_multiplier` - Danger level multiplier
    ///
    /// # Returns
    ///
    /// Encounter probability (0.0 - 1.0)
    pub fn calculate_encounter_chance(
        progress: f32,
        danger_level: f32,
        base_rate: f32,
        danger_multiplier: f32,
    ) -> f32 {
        // Higher danger = more encounters
        (base_rate * (1.0 + danger_level * danger_multiplier) * progress).min(1.0)
    }

    /// Calculate total path distance
    ///
    /// # Arguments
    ///
    /// * `path` - Vector of route IDs
    /// * `registry` - World map registry
    ///
    /// # Returns
    ///
    /// Total distance
    pub fn calculate_path_distance(path: &[RouteId], registry: &WorldMapRegistry) -> f32 {
        path.iter()
            .filter_map(|route_id| {
                let route = registry.get_route(route_id)?;
                if let Some(dist) = route.distance {
                    Some(dist)
                } else {
                    let from_loc = registry.get_location(&route.from)?;
                    let to_loc = registry.get_location(&route.to)?;
                    Some(from_loc.position.distance_to(&to_loc.position))
                }
            })
            .sum()
    }
}

/// Node for Dijkstra's algorithm priority queue
#[derive(Clone, Debug, PartialEq)]
struct PathNode {
    distance: f32,
    location_id: LocationId,
}

impl Eq for PathNode {}

impl PartialOrd for PathNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PathNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order for min-heap (BinaryHeap is max-heap by default)
        other
            .distance
            .partial_cmp(&self.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_duration() {
        let duration = WorldMapService::calculate_duration(100.0, 10.0, 1.0);
        assert_eq!(duration, 10.0);

        let duration = WorldMapService::calculate_duration(100.0, 10.0, 0.5);
        assert_eq!(duration, 20.0);

        let duration = WorldMapService::calculate_duration(100.0, 0.0, 1.0);
        assert_eq!(duration, f32::MAX);
    }

    #[test]
    fn test_find_path_direct() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(
            Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_b", "City B").with_position(Position::new(100.0, 0.0)),
        );

        registry.register_route(Route::new("route_1", "city_a", "city_b"));

        let path =
            WorldMapService::find_path(&"city_a".to_string(), &"city_b".to_string(), &registry);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 1);
        assert_eq!(path[0], "route_1");
    }

    #[test]
    fn test_find_path_multi_hop() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(
            Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_b", "City B").with_position(Position::new(50.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_c", "City C").with_position(Position::new(100.0, 0.0)),
        );

        registry.register_route(Route::new("route_1", "city_a", "city_b"));
        registry.register_route(Route::new("route_2", "city_b", "city_c"));

        let path =
            WorldMapService::find_path(&"city_a".to_string(), &"city_c".to_string(), &registry);

        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(path.len(), 2);
        assert_eq!(path[0], "route_1");
        assert_eq!(path[1], "route_2");
    }

    #[test]
    fn test_find_path_no_route() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(Location::new("city_a", "City A"));
        registry.register_location(Location::new("city_b", "City B"));

        let path =
            WorldMapService::find_path(&"city_a".to_string(), &"city_b".to_string(), &registry);

        assert!(path.is_none());
    }

    #[test]
    fn test_find_path_same_location() {
        let mut registry = WorldMapRegistry::new();
        registry.register_location(Location::new("city_a", "City A"));

        let path =
            WorldMapService::find_path(&"city_a".to_string(), &"city_a".to_string(), &registry);

        assert!(path.is_some());
        assert_eq!(path.unwrap().len(), 0);
    }

    #[test]
    fn test_calculate_encounter_chance() {
        let chance = WorldMapService::calculate_encounter_chance(0.5, 0.5, 0.1, 2.0);
        // 0.1 * (1.0 + 0.5 * 2.0) * 0.5 = 0.1 * 2.0 * 0.5 = 0.1
        assert_eq!(chance, 0.1);

        let chance = WorldMapService::calculate_encounter_chance(1.0, 1.0, 0.5, 2.0);
        // 0.5 * (1.0 + 1.0 * 2.0) * 1.0 = 0.5 * 3.0 * 1.0 = 1.5 -> clamped to 1.0
        assert_eq!(chance, 1.0);
    }

    #[test]
    fn test_calculate_path_distance() {
        let mut registry = WorldMapRegistry::new();

        registry.register_location(
            Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_b", "City B").with_position(Position::new(50.0, 0.0)),
        );
        registry.register_location(
            Location::new("city_c", "City C").with_position(Position::new(100.0, 0.0)),
        );

        registry.register_route(Route::new("route_1", "city_a", "city_b").with_distance(50.0));
        registry.register_route(Route::new("route_2", "city_b", "city_c").with_distance(50.0));

        let path = vec!["route_1".to_string(), "route_2".to_string()];
        let distance = WorldMapService::calculate_path_distance(&path, &registry);

        assert_eq!(distance, 100.0);
    }
}
