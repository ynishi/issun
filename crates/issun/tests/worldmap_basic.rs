//! Basic integration tests for WorldMapPlugin
//!
//! These tests verify the core functionality through public APIs.

use issun::plugin::worldmap::{
    Location, LocationType, Position, Route, TerrainType, WorldMapConfig, WorldMapPlugin,
    WorldMapRegistry, WorldMapService, WorldMapState,
};

#[test]
fn test_registry_setup() {
    let mut registry = WorldMapRegistry::new();

    // Register locations
    registry.register_location(
        Location::new("start_town", "Starting Town")
            .with_position(Position::new(0.0, 0.0))
            .with_type(LocationType::Town),
    );

    registry.register_location(
        Location::new("forest_village", "Forest Village")
            .with_position(Position::new(50.0, 0.0))
            .with_type(LocationType::Village),
    );

    // Register route
    registry.register_route(
        Route::new("route_1", "start_town", "forest_village")
            .with_terrain(TerrainType::Road)
            .with_danger(0.1),
    );

    // Verify setup
    assert_eq!(registry.location_count(), 2);
    assert_eq!(registry.route_count(), 1);

    let location = registry.get_location(&"start_town".to_string()).unwrap();
    assert_eq!(location.name, "Starting Town");

    let route = registry.get_route(&"route_1".to_string()).unwrap();
    assert_eq!(route.from, "start_town");
    assert_eq!(route.to, "forest_village");
}

#[test]
fn test_pathfinding() {
    let mut registry = WorldMapRegistry::new();

    // Create linear path: A -> B -> C
    registry.register_location(
        Location::new("city_a", "City A").with_position(Position::new(0.0, 0.0)),
    );
    registry.register_location(
        Location::new("city_b", "City B").with_position(Position::new(100.0, 0.0)),
    );
    registry.register_location(
        Location::new("city_c", "City C").with_position(Position::new(200.0, 0.0)),
    );

    registry.register_route(Route::new("ab", "city_a", "city_b"));
    registry.register_route(Route::new("bc", "city_b", "city_c"));

    // Find path from A to C
    let path = WorldMapService::find_path(&"city_a".to_string(), &"city_c".to_string(), &registry);

    assert!(path.is_some());
    let path = path.unwrap();
    assert_eq!(path.len(), 2);
    assert_eq!(path[0], "ab");
    assert_eq!(path[1], "bc");

    // Calculate path distance
    let distance = WorldMapService::calculate_path_distance(&path, &registry);
    assert_eq!(distance, 200.0); // 100 + 100
}

#[test]
fn test_travel_state_management() {
    let mut state = WorldMapState::new();

    // Set initial position
    state.set_position("player", "city_a");

    let position = state.get_position(&"player".to_string()).unwrap();
    match position {
        issun::plugin::worldmap::EntityPosition::AtLocation(loc) => {
            assert_eq!(loc, "city_a");
        }
        _ => panic!("Expected AtLocation"),
    }

    // Verify location is discovered
    assert!(state.is_discovered(&"city_a".to_string()));
}

#[test]
fn test_config_builder() {
    let config = WorldMapConfig::default()
        .with_travel_speed(20.0)
        .with_encounter_rate(0.3)
        .with_fog_of_war(true)
        .with_terrain_modifiers(false);

    assert_eq!(config.default_travel_speed, 20.0);
    assert_eq!(config.base_encounter_rate, 0.3);
    assert!(config.enable_fog_of_war);
    assert!(!config.enable_terrain_modifiers);
}

#[test]
fn test_plugin_creation() {
    // Plugin creation should not panic
    let _plugin = WorldMapPlugin::new();
}

#[test]
fn test_plugin_with_custom_config() {
    let config = WorldMapConfig::default()
        .with_travel_speed(15.0)
        .with_fog_of_war(true);

    // Plugin should accept custom config without panicking
    let _plugin = WorldMapPlugin::new().with_config(config);
}

#[test]
fn test_terrain_speed_modifiers() {
    assert_eq!(TerrainType::Road.speed_multiplier(), 1.0);
    assert_eq!(TerrainType::Plains.speed_multiplier(), 0.9);
    assert_eq!(TerrainType::Forest.speed_multiplier(), 0.7);
    assert_eq!(TerrainType::Mountains.speed_multiplier(), 0.5);
    assert_eq!(TerrainType::Desert.speed_multiplier(), 0.6);
    assert_eq!(TerrainType::Swamp.speed_multiplier(), 0.5);
    assert_eq!(TerrainType::Sea.speed_multiplier(), 1.2);
}

#[test]
fn test_position_distance_calculation() {
    let pos1 = Position::new(0.0, 0.0);
    let pos2 = Position::new(3.0, 4.0);

    // 3-4-5 right triangle
    assert_eq!(pos1.distance_to(&pos2), 5.0);

    let pos3 = Position::new(0.0, 0.0);
    let pos4 = Position::new(100.0, 0.0);

    // Horizontal distance
    assert_eq!(pos3.distance_to(&pos4), 100.0);
}

#[test]
fn test_route_builder() {
    let route = Route::new("test_route", "start", "end")
        .with_terrain(TerrainType::Mountains)
        .with_danger(0.7)
        .with_distance(150.0)
        .with_bidirectional(false);

    assert_eq!(route.id, "test_route");
    assert_eq!(route.from, "start");
    assert_eq!(route.to, "end");
    assert_eq!(route.terrain, TerrainType::Mountains);
    assert_eq!(route.danger_level, 0.7);
    assert_eq!(route.distance, Some(150.0));
    assert!(!route.bidirectional);
}

#[test]
fn test_location_builder() {
    let location = Location::new("fortress", "Mountain Fortress")
        .with_position(Position::new(100.0, 50.0))
        .with_type(LocationType::Fortress)
        .with_scene("fortress_scene");

    assert_eq!(location.id, "fortress");
    assert_eq!(location.name, "Mountain Fortress");
    assert_eq!(location.position, Position::new(100.0, 50.0));
    assert_eq!(location.location_type, LocationType::Fortress);
    assert_eq!(location.scene_id, Some("fortress_scene".to_string()));
}

#[test]
fn test_registry_query_operations() {
    let mut registry = WorldMapRegistry::new();

    registry.register_location(Location::new("a", "A").with_position(Position::new(0.0, 0.0)));
    registry.register_location(Location::new("b", "B").with_position(Position::new(100.0, 0.0)));
    registry.register_location(Location::new("c", "C").with_position(Position::new(200.0, 0.0)));

    registry.register_route(Route::new("ab", "a", "b"));
    registry.register_route(Route::new("ac", "a", "c"));
    registry.register_route(Route::new("bc", "b", "c"));

    // Get routes from a location
    let routes_from_a = registry.get_routes_from(&"a".to_string());
    assert_eq!(routes_from_a.len(), 2);

    // Check if route exists
    assert!(registry.has_route(&"a".to_string(), &"b".to_string()));
    assert!(registry.has_route(&"a".to_string(), &"c".to_string()));

    // Calculate distance between locations
    let distance = registry
        .calculate_distance(&"a".to_string(), &"b".to_string())
        .unwrap();
    assert_eq!(distance, 100.0);
}

#[test]
fn test_registry_modification() {
    let mut registry = WorldMapRegistry::new();

    // Add location
    registry.register_location(Location::new("temp", "Temporary"));
    assert_eq!(registry.location_count(), 1);

    // Remove location
    let removed = registry.remove_location(&"temp".to_string());
    assert!(removed.is_some());
    assert_eq!(registry.location_count(), 0);

    // Add and remove route
    registry.register_location(Location::new("a", "A"));
    registry.register_location(Location::new("b", "B"));
    registry.register_route(Route::new("ab", "a", "b"));
    assert_eq!(registry.route_count(), 1);

    let removed_route = registry.remove_route(&"ab".to_string());
    assert!(removed_route.is_some());
    assert_eq!(registry.route_count(), 0);
}

#[test]
fn test_no_path_exists() {
    let mut registry = WorldMapRegistry::new();

    // Create two disconnected locations
    registry.register_location(Location::new("isolated_a", "Isolated A"));
    registry.register_location(Location::new("isolated_b", "Isolated B"));

    let path = WorldMapService::find_path(
        &"isolated_a".to_string(),
        &"isolated_b".to_string(),
        &registry,
    );

    assert!(path.is_none());
}

#[test]
fn test_same_location_path() {
    let mut registry = WorldMapRegistry::new();
    registry.register_location(Location::new("city", "City"));

    let path = WorldMapService::find_path(&"city".to_string(), &"city".to_string(), &registry);

    assert!(path.is_some());
    assert_eq!(path.unwrap().len(), 0); // Empty path for same location
}
