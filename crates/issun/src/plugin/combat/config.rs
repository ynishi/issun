//! Combat system configuration (ReadOnly)

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for combat system (ReadOnly)
///
/// This is a config loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatConfig {
    /// Enable combat log
    pub enable_log: bool,

    /// Max log entries to keep per battle
    pub max_log_entries: usize,

    /// Score awarded per enemy defeated
    pub score_per_enemy: u32,
}

impl Resource for CombatConfig {}

impl Default for CombatConfig {
    fn default() -> Self {
        Self {
            enable_log: true,
            max_log_entries: 100,
            score_per_enemy: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = CombatConfig::default();
        assert!(config.enable_log);
        assert_eq!(config.max_log_entries, 100);
        assert_eq!(config.score_per_enemy, 10);
    }

    #[test]
    fn test_custom_config() {
        let config = CombatConfig {
            enable_log: false,
            max_log_entries: 50,
            score_per_enemy: 20,
        };
        assert!(!config.enable_log);
        assert_eq!(config.max_log_entries, 50);
        assert_eq!(config.score_per_enemy, 20);
    }
}
