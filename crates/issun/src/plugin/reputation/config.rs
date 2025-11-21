//! Reputation system configuration

use super::types::ReputationThreshold;
use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for reputation system (ReadOnly)
///
/// This is an asset/config that is loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationConfig {
    /// Default score for new entries
    pub default_score: f32,

    /// Optional score range (min, max)
    pub score_range: Option<(f32, f32)>,

    /// Enable automatic score clamping
    pub auto_clamp: bool,

    /// Enable score decay over time
    pub enable_decay: bool,

    /// Decay rate per time unit (e.g., per day/turn)
    pub decay_rate: f32,

    /// Optional thresholds for semantic levels
    #[serde(default)]
    pub thresholds: Vec<ReputationThreshold>,
}

impl Default for ReputationConfig {
    fn default() -> Self {
        Self {
            default_score: 0.0,
            score_range: None,
            auto_clamp: false,
            enable_decay: false,
            decay_rate: 0.0,
            thresholds: Vec::new(),
        }
    }
}

impl Resource for ReputationConfig {}

impl ReputationConfig {
    /// Get threshold for a given score
    pub fn get_threshold(&self, score: f32) -> Option<&ReputationThreshold> {
        self.thresholds.iter().find(|t| t.contains(score))
    }

    /// Add a threshold
    pub fn add_threshold(&mut self, threshold: ReputationThreshold) {
        self.thresholds.push(threshold);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ReputationConfig::default();
        assert_eq!(config.default_score, 0.0);
        assert_eq!(config.score_range, None);
        assert!(!config.auto_clamp);
        assert!(!config.enable_decay);
        assert_eq!(config.decay_rate, 0.0);
        assert!(config.thresholds.is_empty());
    }

    #[test]
    fn test_custom_config() {
        let config = ReputationConfig {
            default_score: 50.0,
            score_range: Some((-100.0, 100.0)),
            auto_clamp: true,
            enable_decay: true,
            decay_rate: 0.1,
            thresholds: Vec::new(),
        };
        assert_eq!(config.default_score, 50.0);
        assert_eq!(config.score_range, Some((-100.0, 100.0)));
        assert!(config.auto_clamp);
        assert!(config.enable_decay);
        assert_eq!(config.decay_rate, 0.1);
    }
}
