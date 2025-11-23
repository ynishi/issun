//! Configuration for OrganizationSuitePlugin

use serde::{Deserialize, Serialize};

/// OrganizationSuitePlugin configuration (ReadOnly Resource)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgSuiteConfig {
    /// Enable/disable automatic transitions based on conditions
    pub enable_auto_transition: bool,

    /// How often to check transition conditions (every N ticks)
    pub transition_check_interval: u32,

    /// Log transitions to output
    pub log_transitions: bool,
}

impl Default for OrgSuiteConfig {
    fn default() -> Self {
        Self {
            enable_auto_transition: true,
            transition_check_interval: 1, // Check every tick
            log_transitions: true,
        }
    }
}

impl OrgSuiteConfig {
    /// Create new config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder: Enable/disable automatic transitions
    pub fn with_auto_transition(mut self, enabled: bool) -> Self {
        self.enable_auto_transition = enabled;
        self
    }

    /// Builder: Set transition check interval
    pub fn with_check_interval(mut self, interval: u32) -> Self {
        self.transition_check_interval = interval;
        self
    }

    /// Builder: Enable/disable transition logging
    pub fn with_logging(mut self, enabled: bool) -> Self {
        self.log_transitions = enabled;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OrgSuiteConfig::default();
        assert_eq!(config.enable_auto_transition, true);
        assert_eq!(config.transition_check_interval, 1);
        assert_eq!(config.log_transitions, true);
    }

    #[test]
    fn test_builder_pattern() {
        let config = OrgSuiteConfig::new()
            .with_auto_transition(false)
            .with_check_interval(10)
            .with_logging(false);

        assert_eq!(config.enable_auto_transition, false);
        assert_eq!(config.transition_check_interval, 10);
        assert_eq!(config.log_transitions, false);
    }

    #[test]
    fn test_serde_roundtrip() {
        let config = OrgSuiteConfig::new().with_check_interval(5);

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: OrgSuiteConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.transition_check_interval, deserialized.transition_check_interval);
    }
}
