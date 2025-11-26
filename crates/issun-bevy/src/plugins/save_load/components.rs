//! Save/Load components

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

// Re-export moonshine_save markers
pub use moonshine_save::prelude::{Save, Unload};

/// Per-save metadata
///
/// This component is attached to a root entity in the save file
/// to store metadata about the save (timestamp, version, etc.)
#[derive(Component, Clone, Debug, Reflect, Serialize, Deserialize)]
#[reflect(Component)]
pub struct SaveMetadata {
    /// Save slot name (e.g., "slot_1", "auto_save", "quicksave")
    pub slot_name: String,

    /// Timestamp (Unix seconds)
    pub timestamp: u64,

    /// ISSUN version (e.g., "0.6.0")
    pub version: String,

    /// Game day (from TimePlugin, if available)
    pub game_day: u32,

    /// Custom metadata (JSON string)
    pub custom: String,
}

impl SaveMetadata {
    /// Create new save metadata
    pub fn new(slot_name: impl Into<String>) -> Self {
        Self {
            slot_name: slot_name.into(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            game_day: 0,
            custom: String::new(),
        }
    }

    /// Set game day
    pub fn with_game_day(mut self, game_day: u32) -> Self {
        self.game_day = game_day;
        self
    }

    /// Set custom metadata
    pub fn with_custom(mut self, custom: impl Into<String>) -> Self {
        self.custom = custom.into();
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_metadata_creation() {
        let metadata = SaveMetadata::new("slot_1");
        assert_eq!(metadata.slot_name, "slot_1");
        assert_eq!(metadata.version, env!("CARGO_PKG_VERSION"));
        assert!(metadata.timestamp > 0);
    }

    #[test]
    fn test_save_metadata_builder() {
        let metadata = SaveMetadata::new("slot_1")
            .with_game_day(42)
            .with_custom(r#"{"player": "Hero"}"#);

        assert_eq!(metadata.game_day, 42);
        assert_eq!(metadata.custom, r#"{"player": "Hero"}"#);
    }
}
