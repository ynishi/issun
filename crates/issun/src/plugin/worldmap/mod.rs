//! World map plugin for spatial navigation and travel simulation
//!
//! # Overview
//!
//! The WorldMap plugin manages large-scale world maps with travel simulation,
//! spatial positioning, and encounter systems. It separates static world geography
//! from dynamic travel state, enabling rich exploration gameplay with time/distance mechanics.
//!
//! # Features
//!
//! - **Graph-Based Spatial Model**: Locations connected by routes with terrain effects
//! - **Time-Based Travel Simulation**: Progress tracking with speed modifiers
//! - **Dynamic Encounter System**: Probabilistic encounters during travel
//! - **Fog of War Support**: Discover locations as you travel
//! - **Scene Integration**: Seamless transitions between world map and scenes
//! - **Pathfinding**: Dijkstra-based shortest path calculation
//!
//! # Example
//!
//! ```ignore
//! use issun::prelude::*;
//! use issun::plugin::worldmap::{WorldMapPlugin, Location, Route, Position, TerrainType};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let game = GameBuilder::new()
//!         .with_plugin(WorldMapPlugin::new())
//!         .build()
//!         .await?;
//!
//!     // Setup world map
//!     let worldmap = game.get_plugin_mut::<WorldMapPlugin>().unwrap();
//!
//!     // Register locations
//!     worldmap.registry_mut().register_location(
//!         Location::new("city_a", "City A")
//!             .with_position(Position::new(0.0, 0.0))
//!             .with_type(LocationType::City)
//!     );
//!
//!     worldmap.registry_mut().register_location(
//!         Location::new("city_b", "City B")
//!             .with_position(Position::new(100.0, 0.0))
//!             .with_type(LocationType::Town)
//!     );
//!
//!     // Register route
//!     worldmap.registry_mut().register_route(
//!         Route::new("route_1", "city_a", "city_b")
//!             .with_terrain(TerrainType::Road)
//!             .with_danger(0.3)
//!     );
//!
//!     // Start travel
//!     let travel_id = worldmap.system_mut().start_travel(
//!         worldmap.state_mut(),
//!         worldmap.registry(),
//!         worldmap.config(),
//!         "player_1",
//!         "city_a",
//!         "city_b",
//!     ).await?;
//!
//!     println!("Travel started: {}", travel_id);
//!
//!     // Game loop
//!     loop {
//!         let delta_time = 1.0 / 60.0; // 60 FPS
//!
//!         let worldmap = game.get_plugin_mut::<WorldMapPlugin>().unwrap();
//!         let events = worldmap.system_mut().update(
//!             worldmap.state_mut(),
//!             worldmap.registry(),
//!             worldmap.config(),
//!             delta_time,
//!         ).await;
//!
//!         for event in events {
//!             match event {
//!                 TravelEvent::Arrived { entity_id, location_id } => {
//!                     println!("{} arrived at {}", entity_id, location_id);
//!                     break;
//!                 }
//!                 TravelEvent::EncounterTriggered { entity_id, encounter } => {
//!                     println!("{} encountered: {}", entity_id, encounter.encounter_type);
//!                 }
//!                 _ => {}
//!             }
//!         }
//!
//!         // Check if travel complete
//!         if worldmap.state().active_travel_count() == 0 {
//!             break;
//!         }
//!
//!         tokio::time::sleep(tokio::time::Duration::from_millis(16)).await;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! # Performance
//!
//! - **Pathfinding**: O(E log V) with Dijkstra's algorithm
//! - **Travel Update**: O(n) where n = active travels
//! - **Encounter Check**: O(n) amortized over time
//!
//! # Architecture
//!
//! ```text
//! WorldMapPlugin
//! ├── Config (WorldMapConfig) - Global settings
//! ├── Registry (WorldMapRegistry) - Locations & routes
//! ├── State (WorldMapState) - Active travels & positions
//! ├── Service (WorldMapService) - Pure pathfinding logic
//! ├── System (WorldMapSystem) - Travel orchestration
//! └── Hook (WorldMapHook) - Customization points
//! ```

pub mod config;
pub mod events;
pub mod hook;
pub mod plugin;
pub mod registry;
pub mod service;
pub mod state;
pub mod system;
pub mod types;

// Re-exports
pub use config::WorldMapConfig;
pub use events::*;
pub use hook::{DefaultWorldMapHook, WorldMapHook};
pub use plugin::WorldMapPlugin;
pub use registry::WorldMapRegistry;
pub use service::WorldMapService;
pub use state::{EntityPosition, TravelCompleted, WorldMapState};
pub use system::{TravelError, TravelEvent, WorldMapSystem};
pub use types::{
    Encounter, EncounterId, EntityId, Location, LocationId, LocationType, Position, Route, RouteId,
    SceneId, TerrainType, Travel, TravelId, TravelStatus,
};
