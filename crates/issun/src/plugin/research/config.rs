//! Research system configuration (ReadOnly)

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Progress model for research projects
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProgressModel {
    /// Fixed progress per turn (e.g., 0.1 per turn = 10 turns)
    TurnBased,

    /// Real-time progress (requires GameTimer plugin)
    TimeBased,

    /// Manual progress updates via events
    Manual,
}

impl Default for ProgressModel {
    fn default() -> Self {
        Self::TurnBased
    }
}

/// Configuration for research system (ReadOnly)
///
/// This is a config/asset loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    /// Allow multiple projects to be researched simultaneously
    pub allow_parallel_research: bool,

    /// Maximum number of parallel research slots
    pub max_parallel_slots: usize,

    /// Progress model (turn-based, time-based, manual)
    pub progress_model: ProgressModel,

    /// Auto-advance progress each turn/tick
    pub auto_advance: bool,

    /// Base progress per turn (when auto_advance = true)
    pub base_progress_per_turn: f32,
}

impl Resource for ResearchConfig {}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            allow_parallel_research: false,
            max_parallel_slots: 1,
            progress_model: ProgressModel::TurnBased,
            auto_advance: true,
            base_progress_per_turn: 0.1, // 10 turns by default
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ResearchConfig::default();
        assert!(!config.allow_parallel_research);
        assert_eq!(config.max_parallel_slots, 1);
        assert_eq!(config.progress_model, ProgressModel::TurnBased);
        assert!(config.auto_advance);
        assert_eq!(config.base_progress_per_turn, 0.1);
    }

    #[test]
    fn test_progress_model_default() {
        assert_eq!(ProgressModel::default(), ProgressModel::TurnBased);
    }
}
