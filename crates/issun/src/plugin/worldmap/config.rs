//! Configuration for WorldMapPlugin

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// World map plugin configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorldMapConfig {
    /// Default travel speed (units per second)
    pub default_travel_speed: f32,

    /// Enable terrain speed modifiers
    pub enable_terrain_modifiers: bool,

    /// Base encounter chance per distance unit (0.0 - 1.0)
    pub base_encounter_rate: f32,

    /// Multiplier for danger level effect on encounters
    pub danger_multiplier: f32,

    /// Minimum time between encounters (seconds)
    pub min_encounter_interval: f32,

    /// Enable fog of war (unvisited locations hidden)
    pub enable_fog_of_war: bool,

    /// Auto-transition to scene when arriving at location
    pub auto_scene_transition: bool,
}

impl Default for WorldMapConfig {
    fn default() -> Self {
        Self {
            default_travel_speed: 10.0, // 10 units/second
            enable_terrain_modifiers: true,
            base_encounter_rate: 0.1,
            danger_multiplier: 2.0,
            min_encounter_interval: 30.0, // 30 seconds
            enable_fog_of_war: false,
            auto_scene_transition: true,
        }
    }
}

impl WorldMapConfig {
    /// Create config with custom travel speed
    pub fn with_travel_speed(mut self, speed: f32) -> Self {
        self.default_travel_speed = speed;
        self
    }

    /// Create config with custom encounter rate
    pub fn with_encounter_rate(mut self, rate: f32) -> Self {
        self.base_encounter_rate = rate.clamp(0.0, 1.0);
        self
    }

    /// Create config with custom danger multiplier
    pub fn with_danger_multiplier(mut self, multiplier: f32) -> Self {
        self.danger_multiplier = multiplier;
        self
    }

    /// Create config with custom min encounter interval
    pub fn with_min_encounter_interval(mut self, interval: f32) -> Self {
        self.min_encounter_interval = interval;
        self
    }

    /// Enable/disable terrain modifiers
    pub fn with_terrain_modifiers(mut self, enabled: bool) -> Self {
        self.enable_terrain_modifiers = enabled;
        self
    }

    /// Enable/disable fog of war
    pub fn with_fog_of_war(mut self, enabled: bool) -> Self {
        self.enable_fog_of_war = enabled;
        self
    }

    /// Enable/disable auto scene transition
    pub fn with_auto_scene_transition(mut self, enabled: bool) -> Self {
        self.auto_scene_transition = enabled;
        self
    }
}

impl Resource for WorldMapConfig {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = WorldMapConfig::default();

        assert_eq!(config.default_travel_speed, 10.0);
        assert!(config.enable_terrain_modifiers);
        assert_eq!(config.base_encounter_rate, 0.1);
        assert_eq!(config.danger_multiplier, 2.0);
        assert_eq!(config.min_encounter_interval, 30.0);
        assert!(!config.enable_fog_of_war);
        assert!(config.auto_scene_transition);
    }

    #[test]
    fn test_config_builder() {
        let config = WorldMapConfig::default()
            .with_travel_speed(20.0)
            .with_encounter_rate(0.2)
            .with_danger_multiplier(3.0)
            .with_min_encounter_interval(60.0)
            .with_terrain_modifiers(false)
            .with_fog_of_war(true)
            .with_auto_scene_transition(false);

        assert_eq!(config.default_travel_speed, 20.0);
        assert!(!config.enable_terrain_modifiers);
        assert_eq!(config.base_encounter_rate, 0.2);
        assert_eq!(config.danger_multiplier, 3.0);
        assert_eq!(config.min_encounter_interval, 60.0);
        assert!(config.enable_fog_of_war);
        assert!(!config.auto_scene_transition);
    }

    #[test]
    fn test_encounter_rate_clamping() {
        let config = WorldMapConfig::default().with_encounter_rate(1.5);
        assert_eq!(config.base_encounter_rate, 1.0);

        let config = WorldMapConfig::default().with_encounter_rate(-0.5);
        assert_eq!(config.base_encounter_rate, 0.0);
    }
}
