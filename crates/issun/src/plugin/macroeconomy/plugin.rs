//! Plugin definition for macroeconomy

use super::config::MacroeconomyConfig;
use super::resources::EconomicMetrics;
use super::state::MacroeconomyState;

/// Macroeconomy plugin
///
/// Provides macro-economic indicators by aggregating data from
/// market, accounting, generation, and economy plugins.
#[derive(Debug, Clone)]
pub struct MacroeconomyPlugin {
    config: MacroeconomyConfig,
}

impl MacroeconomyPlugin {
    /// Create a new macroeconomy plugin with default configuration
    pub fn new() -> Self {
        Self {
            config: MacroeconomyConfig::default(),
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: MacroeconomyConfig) -> Self {
        self.config = config;
        self
    }

    /// Set update interval
    pub fn with_update_interval(mut self, interval: u64) -> Self {
        self.config.update_interval = interval;
        self
    }

    /// Get configuration
    pub fn config(&self) -> &MacroeconomyConfig {
        &self.config
    }

    /// Get initial state
    pub fn initial_state() -> MacroeconomyState {
        MacroeconomyState::default()
    }

    /// Get initial metrics
    pub fn initial_metrics() -> EconomicMetrics {
        EconomicMetrics::default()
    }
}

impl Default for MacroeconomyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let plugin = MacroeconomyPlugin::new();
        assert_eq!(plugin.config().update_interval, 10);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = MacroeconomyConfig {
            update_interval: 5,
            ..Default::default()
        };

        let plugin = MacroeconomyPlugin::new().with_config(config);
        assert_eq!(plugin.config().update_interval, 5);
    }

    #[test]
    fn test_plugin_with_update_interval() {
        let plugin = MacroeconomyPlugin::new().with_update_interval(20);
        assert_eq!(plugin.config().update_interval, 20);
    }

    #[test]
    fn test_initial_state() {
        let state = MacroeconomyPlugin::initial_state();
        assert_eq!(state.indicators.last_update, 0);
    }

    #[test]
    fn test_initial_metrics() {
        let metrics = MacroeconomyPlugin::initial_metrics();
        assert_eq!(metrics.current_tick, 0);
    }
}
