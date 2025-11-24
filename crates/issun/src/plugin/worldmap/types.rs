//! Core types for WorldMapPlugin

use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Unique identifier for world locations
pub type LocationId = String;

/// Unique identifier for routes between locations
pub type RouteId = String;

/// Unique identifier for travel instances
pub type TravelId = String;

/// Entity ID (for players, NPCs, caravans)
pub type EntityId = String;

/// Scene ID (for seamless transitions)
pub type SceneId = String;

/// Encounter ID
pub type EncounterId = String;

/// A location on the world map (static definition)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    /// Unique identifier
    pub id: LocationId,

    /// Display name
    pub name: String,

    /// World coordinates (for distance calculation and visualization)
    pub position: Position,

    /// Optional scene to transition to when arriving
    pub scene_id: Option<SceneId>,

    /// Location type (City, Dungeon, Wilderness, etc.)
    pub location_type: LocationType,

    /// Game-specific metadata
    #[serde(default)]
    pub metadata: serde_json::Value,
}

impl Location {
    /// Create a new location
    pub fn new(id: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            position: Position::default(),
            scene_id: None,
            location_type: LocationType::City,
            metadata: serde_json::Value::Null,
        }
    }

    /// Set position
    pub fn with_position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Set scene ID
    pub fn with_scene(mut self, scene_id: impl Into<String>) -> Self {
        self.scene_id = Some(scene_id.into());
        self
    }

    /// Set location type
    pub fn with_type(mut self, location_type: LocationType) -> Self {
        self.location_type = location_type;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// 2D world coordinates
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Position {
    /// Create a new position
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    /// Calculate Euclidean distance
    pub fn distance_to(&self, other: &Position) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        (dx * dx + dy * dy).sqrt()
    }
}

impl Default for Position {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Location types
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    City,
    Town,
    Village,
    Dungeon,
    Fortress,
    Wilderness,
    Port,
    Ruins,
    Custom(String),
}

/// A route connecting two locations
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Route {
    /// Unique identifier
    pub id: RouteId,

    /// Starting location
    pub from: LocationId,

    /// Destination location
    pub to: LocationId,

    /// Terrain type (affects travel speed)
    pub terrain: TerrainType,

    /// Danger level (0.0 = safe, 1.0 = very dangerous)
    pub danger_level: f32,

    /// Distance (can override Position-based calculation)
    pub distance: Option<f32>,

    /// Is bidirectional (can travel both ways)
    pub bidirectional: bool,
}

impl Route {
    /// Create a new route
    pub fn new(id: impl Into<String>, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            from: from.into(),
            to: to.into(),
            terrain: TerrainType::Road,
            danger_level: 0.0,
            distance: None,
            bidirectional: true,
        }
    }

    /// Set terrain type
    pub fn with_terrain(mut self, terrain: TerrainType) -> Self {
        self.terrain = terrain;
        self
    }

    /// Set danger level
    pub fn with_danger(mut self, danger_level: f32) -> Self {
        self.danger_level = danger_level.clamp(0.0, 1.0);
        self
    }

    /// Set explicit distance
    pub fn with_distance(mut self, distance: f32) -> Self {
        self.distance = Some(distance);
        self
    }

    /// Set bidirectional
    pub fn with_bidirectional(mut self, bidirectional: bool) -> Self {
        self.bidirectional = bidirectional;
        self
    }
}

/// Terrain types (affects travel speed)
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TerrainType {
    Road,      // 1.0x speed (baseline)
    Plains,    // 0.9x speed
    Forest,    // 0.7x speed
    Mountains, // 0.5x speed
    Desert,    // 0.6x speed
    Swamp,     // 0.5x speed
    Sea,       // 1.2x speed (ship travel)
    Custom(String),
}

impl TerrainType {
    /// Get speed multiplier for this terrain
    pub fn speed_multiplier(&self) -> f32 {
        match self {
            TerrainType::Road => 1.0,
            TerrainType::Plains => 0.9,
            TerrainType::Forest => 0.7,
            TerrainType::Mountains => 0.5,
            TerrainType::Desert => 0.6,
            TerrainType::Swamp => 0.5,
            TerrainType::Sea => 1.2,
            TerrainType::Custom(_) => 1.0,
        }
    }
}

/// Active travel instance (runtime state)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Travel {
    /// Unique identifier
    pub id: TravelId,

    /// Entity performing the travel (player, NPC, caravan)
    pub entity_id: EntityId,

    /// Route being traveled
    pub route_id: RouteId,

    /// Starting location
    pub from: LocationId,

    /// Destination location
    pub to: LocationId,

    /// Travel progress (0.0 = start, 1.0 = arrived)
    pub progress: f32,

    /// Total travel duration (based on distance and speed)
    pub total_duration: f32,

    /// Time elapsed (seconds)
    pub elapsed_time: f32,

    /// Base speed (units per second)
    pub speed: f32,

    /// Current status
    pub status: TravelStatus,

    /// Encounters experienced during this journey
    pub encounters: Vec<EncounterId>,

    /// Started at (for progress tracking)
    #[serde(skip)]
    pub started_at: Option<Instant>,
}

impl Travel {
    /// Create a new travel instance
    pub fn new(
        id: impl Into<String>,
        entity_id: impl Into<String>,
        route_id: impl Into<String>,
        from: impl Into<String>,
        to: impl Into<String>,
        distance: f32,
        speed: f32,
    ) -> Self {
        let total_duration = if speed > 0.0 {
            distance / speed
        } else {
            f32::MAX
        };

        Self {
            id: id.into(),
            entity_id: entity_id.into(),
            route_id: route_id.into(),
            from: from.into(),
            to: to.into(),
            progress: 0.0,
            total_duration,
            elapsed_time: 0.0,
            speed,
            status: TravelStatus::InProgress,
            encounters: Vec::new(),
            started_at: Some(Instant::now()),
        }
    }

    /// Update travel progress
    pub fn update(&mut self, delta_time: f32) {
        if self.status != TravelStatus::InProgress {
            return;
        }

        self.elapsed_time += delta_time;
        self.progress = (self.elapsed_time / self.total_duration).min(1.0);

        if self.progress >= 1.0 {
            self.status = TravelStatus::Arrived;
        }
    }

    /// Get current world position (interpolated)
    pub fn current_position(&self, from_pos: &Position, to_pos: &Position) -> Position {
        Position {
            x: from_pos.x + (to_pos.x - from_pos.x) * self.progress,
            y: from_pos.y + (to_pos.y - from_pos.y) * self.progress,
        }
    }

    /// Pause travel
    pub fn pause(&mut self) {
        if self.status == TravelStatus::InProgress {
            self.status = TravelStatus::Paused;
        }
    }

    /// Resume travel
    pub fn resume(&mut self) {
        if self.status == TravelStatus::Paused {
            self.status = TravelStatus::InProgress;
            self.started_at = Some(Instant::now());
        }
    }

    /// Cancel travel
    pub fn cancel(&mut self) {
        self.status = TravelStatus::Cancelled;
    }
}

/// Travel status
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TravelStatus {
    /// Currently traveling
    InProgress,

    /// Paused (by player or event)
    Paused,

    /// Arrived at destination
    Arrived,

    /// Interrupted (by encounter, combat, etc.)
    Interrupted,

    /// Cancelled by player
    Cancelled,
}

/// Encounter during travel
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Encounter {
    /// Unique identifier
    pub id: EncounterId,

    /// Encounter type (game-specific)
    pub encounter_type: String,

    /// Progress point where encounter occurred (0.0 - 1.0)
    pub progress: f32,

    /// Was resolved (or still active)
    pub resolved: bool,

    /// Game-specific data
    #[serde(default)]
    pub data: serde_json::Value,
}

impl Encounter {
    /// Create a new encounter
    pub fn new(id: impl Into<String>, encounter_type: impl Into<String>, progress: f32) -> Self {
        Self {
            id: id.into(),
            encounter_type: encounter_type.into(),
            progress,
            resolved: false,
            data: serde_json::Value::Null,
        }
    }

    /// Set encounter data
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = data;
        self
    }

    /// Resolve the encounter
    pub fn resolve(&mut self) {
        self.resolved = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_distance() {
        let pos1 = Position::new(0.0, 0.0);
        let pos2 = Position::new(3.0, 4.0);

        assert_eq!(pos1.distance_to(&pos2), 5.0);
    }

    #[test]
    fn test_location_builder() {
        let location = Location::new("city_1", "Test City")
            .with_position(Position::new(10.0, 20.0))
            .with_type(LocationType::City)
            .with_scene("city_1_scene");

        assert_eq!(location.id, "city_1");
        assert_eq!(location.name, "Test City");
        assert_eq!(location.position, Position::new(10.0, 20.0));
        assert_eq!(location.scene_id, Some("city_1_scene".to_string()));
        assert_eq!(location.location_type, LocationType::City);
    }

    #[test]
    fn test_route_builder() {
        let route = Route::new("route_1", "city_a", "city_b")
            .with_terrain(TerrainType::Mountains)
            .with_danger(0.7)
            .with_distance(100.0)
            .with_bidirectional(false);

        assert_eq!(route.id, "route_1");
        assert_eq!(route.from, "city_a");
        assert_eq!(route.to, "city_b");
        assert_eq!(route.terrain, TerrainType::Mountains);
        assert_eq!(route.danger_level, 0.7);
        assert_eq!(route.distance, Some(100.0));
        assert!(!route.bidirectional);
    }

    #[test]
    fn test_terrain_speed_multiplier() {
        assert_eq!(TerrainType::Road.speed_multiplier(), 1.0);
        assert_eq!(TerrainType::Mountains.speed_multiplier(), 0.5);
        assert_eq!(TerrainType::Sea.speed_multiplier(), 1.2);
    }

    #[test]
    fn test_travel_update() {
        let mut travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0,
            10.0, // 10 units/sec = 10 seconds total
        );

        assert_eq!(travel.progress, 0.0);
        assert_eq!(travel.status, TravelStatus::InProgress);

        // Update 5 seconds
        travel.update(5.0);
        assert_eq!(travel.progress, 0.5);
        assert_eq!(travel.status, TravelStatus::InProgress);

        // Complete travel
        travel.update(5.0);
        assert_eq!(travel.progress, 1.0);
        assert_eq!(travel.status, TravelStatus::Arrived);
    }

    #[test]
    fn test_travel_current_position() {
        let travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0, 10.0,
        );

        let from = Position::new(0.0, 0.0);
        let to = Position::new(100.0, 0.0);

        // At start
        assert_eq!(travel.current_position(&from, &to), Position::new(0.0, 0.0));

        // At 50% progress
        let mut halfway = travel.clone();
        halfway.progress = 0.5;
        assert_eq!(
            halfway.current_position(&from, &to),
            Position::new(50.0, 0.0)
        );
    }

    #[test]
    fn test_travel_pause_resume() {
        let mut travel = Travel::new(
            "travel_1", "player_1", "route_1", "city_a", "city_b", 100.0, 10.0,
        );

        assert_eq!(travel.status, TravelStatus::InProgress);

        travel.pause();
        assert_eq!(travel.status, TravelStatus::Paused);

        // Update should not progress when paused
        travel.update(5.0);
        assert_eq!(travel.progress, 0.0);

        travel.resume();
        assert_eq!(travel.status, TravelStatus::InProgress);
    }

    #[test]
    fn test_encounter_creation() {
        let encounter = Encounter::new("enc_1", "bandits", 0.5)
            .with_data(serde_json::json!({"enemy_count": 3}));

        assert_eq!(encounter.id, "enc_1");
        assert_eq!(encounter.encounter_type, "bandits");
        assert_eq!(encounter.progress, 0.5);
        assert!(!encounter.resolved);
    }
}
