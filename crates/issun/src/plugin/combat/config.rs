//! Combat system configuration (ReadOnly)

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for combat system
///
/// This config can be modified at runtime by MODs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatConfig {
    /// Enable/disable combat system (MOD-controllable)
    pub enabled: bool,

    /// Default maximum HP for combatants (MOD-controllable)
    pub default_max_hp: u32,

    /// Difficulty multiplier (MOD-controllable)
    pub difficulty_multiplier: f32,

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
            enabled: true,
            default_max_hp: 100,
            difficulty_multiplier: 1.0,
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
        assert!(config.enabled);
        assert_eq!(config.default_max_hp, 100);
        assert_eq!(config.difficulty_multiplier, 1.0);
        assert!(config.enable_log);
        assert_eq!(config.max_log_entries, 100);
        assert_eq!(config.score_per_enemy, 10);
    }

    #[test]
    fn test_custom_config() {
        let config = CombatConfig {
            enabled: false,
            default_max_hp: 50,
            difficulty_multiplier: 2.0,
            enable_log: false,
            max_log_entries: 50,
            score_per_enemy: 20,
        };
        assert!(!config.enabled);
        assert_eq!(config.default_max_hp, 50);
        assert_eq!(config.difficulty_multiplier, 2.0);
        assert!(!config.enable_log);
        assert_eq!(config.max_log_entries, 50);
        assert_eq!(config.score_per_enemy, 20);
    }
}
