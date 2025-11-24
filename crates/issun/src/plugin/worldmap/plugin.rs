//! WorldMapPlugin implementation

use super::config::WorldMapConfig;
use super::hook::{DefaultWorldMapHook, WorldMapHook};
use super::registry::WorldMapRegistry;
use super::service::WorldMapService;
use super::state::WorldMapState;
use super::system::WorldMapSystem;
use crate::Plugin;
use std::sync::Arc;

/// World map plugin for spatial navigation and travel simulation
///
/// This plugin provides world map functionality with:
/// - Location and route management
/// - Travel simulation with time/distance mechanics
/// - Encounter system during travel
/// - Fog of war support
/// - Scene integration for seamless transitions
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Generate encounters during travel
/// - Calculate travel costs
/// - Validate travel permissions
/// - React to travel events
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::worldmap::{WorldMapPlugin, Location, Route, Position};
///
/// let game = GameBuilder::new()
///     .with_plugin(WorldMapPlugin::new())
///     .build()
///     .await?;
///
/// // Register locations
/// let mut worldmap = game.get_plugin_mut::<WorldMapPlugin>().unwrap();
/// worldmap.registry_mut().register_location(
///     Location::new("city_a", "City A")
///         .with_position(Position::new(0.0, 0.0))
/// );
/// worldmap.registry_mut().register_location(
///     Location::new("city_b", "City B")
///         .with_position(Position::new(100.0, 0.0))
/// );
///
/// // Register route
/// worldmap.registry_mut().register_route(
///     Route::new("route_1", "city_a", "city_b")
/// );
///
/// // Start travel
/// worldmap.system_mut().start_travel(
///     worldmap.state_mut(),
///     worldmap.registry(),
///     worldmap.config(),
///     "player_1",
///     "city_a",
///     "city_b",
/// ).await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:worldmap")]
pub struct WorldMapPlugin {
    #[plugin(skip)]
    hook: Arc<dyn WorldMapHook>,

    #[plugin(resource)]
    config: WorldMapConfig,

    #[plugin(resource)]
    registry: WorldMapRegistry,

    #[plugin(runtime_state)]
    state: WorldMapState,

    #[plugin(service)]
    service: WorldMapService,

    #[plugin(system)]
    system: WorldMapSystem,
}

impl WorldMapPlugin {
    /// Create a new world map plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultWorldMapHook);
        Self {
            hook: hook.clone(),
            config: WorldMapConfig::default(),
            registry: WorldMapRegistry::new(),
            state: WorldMapState::new(),
            service: WorldMapService,
            system: WorldMapSystem::new(hook),
        }
    }

    /// Add a custom hook for world map behavior
    ///
    /// The hook will be called when:
    /// - Travel starts/completes/cancels (`on_travel_*`)
    /// - Encounters need to be generated (`generate_encounter`)
    /// - Travel permissions need validation (`can_start_travel`)
    /// - Speed modifiers need calculation (`calculate_speed_modifier`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of WorldMapHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::worldmap::{WorldMapPlugin, WorldMapHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl WorldMapHook for MyHook {
    ///     async fn on_travel_completed(&self, entity_id: &str, location_id: &str) {
    ///         println!("Entity {} arrived at {}", entity_id, location_id);
    ///     }
    /// }
    ///
    /// let plugin = WorldMapPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: WorldMapHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = WorldMapSystem::new(hook);
        self
    }

    /// Set custom world map configuration
    ///
    /// # Arguments
    ///
    /// * `config` - World map configuration
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::worldmap::{WorldMapPlugin, WorldMapConfig};
    ///
    /// let config = WorldMapConfig::default()
    ///     .with_travel_speed(20.0)
    ///     .with_encounter_rate(0.2)
    ///     .with_fog_of_war(true);
    ///
    /// let plugin = WorldMapPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: WorldMapConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for WorldMapPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Plugin;

    #[test]
    fn test_plugin_creation() {
        let plugin = WorldMapPlugin::new();
        assert_eq!(plugin.name(), "issun:worldmap");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl WorldMapHook for CustomHook {}

        let plugin = WorldMapPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:worldmap");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = WorldMapConfig::default()
            .with_travel_speed(20.0)
            .with_encounter_rate(0.2)
            .with_fog_of_war(true);

        let plugin = WorldMapPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "issun:worldmap");
    }
}
