//! Time plugin configuration

use crate::resources::Resource;

/// Configuration for time management plugin
///
/// This configuration determines how time flows in the game.
///
/// # Example
///
/// ```ignore
/// use issun::plugin::time::{TimeConfig, BuiltInTimePlugin};
///
/// let config = TimeConfig {
///     initial_day: 1,
///     actions_per_day: 5,
/// };
///
/// let plugin = BuiltInTimePlugin::new(config);
/// ```
#[derive(Debug, Clone)]
pub struct TimeConfig {
    /// Starting day number (default: 1)
    pub initial_day: u32,
    /// Number of action points available per day (default: 3)
    pub actions_per_day: u32,
}

impl Resource for TimeConfig {}

impl Default for TimeConfig {
    fn default() -> Self {
        Self {
            initial_day: 1,
            actions_per_day: 3,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TimeConfig::default();
        assert_eq!(config.initial_day, 1);
        assert_eq!(config.actions_per_day, 3);
    }

    #[test]
    fn test_custom_config() {
        let config = TimeConfig {
            initial_day: 10,
            actions_per_day: 5,
        };
        assert_eq!(config.initial_day, 10);
        assert_eq!(config.actions_per_day, 5);
    }

    #[test]
    fn test_config_clone() {
        let config1 = TimeConfig {
            initial_day: 1,
            actions_per_day: 3,
        };
        let config2 = config1.clone();
        assert_eq!(config1.initial_day, config2.initial_day);
        assert_eq!(config1.actions_per_day, config2.actions_per_day);
    }
}
