//! Loot system configuration (ReadOnly)

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for loot system (ReadOnly)
///
/// This is a config loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootConfig {
    /// Global drop rate multiplier (applies to all drops)
    pub global_drop_multiplier: f32,
}

impl Resource for LootConfig {}

impl Default for LootConfig {
    fn default() -> Self {
        Self {
            global_drop_multiplier: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LootConfig::default();
        assert_eq!(config.global_drop_multiplier, 1.0);
    }

    #[test]
    fn test_custom_config() {
        let config = LootConfig {
            global_drop_multiplier: 1.5,
        };
        assert_eq!(config.global_drop_multiplier, 1.5);
    }
}
