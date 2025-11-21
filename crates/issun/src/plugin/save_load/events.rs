//! Save/Load plugin events

use crate::event::Event;
use crate::storage::save_data::SaveMetadata;
use serde::{Deserialize, Serialize};

// ============================================================================
// Command Events (request actions)
// ============================================================================

/// Request to save game state to a slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveGameRequested {
    /// Save slot name
    pub slot: String,
    /// Optional custom save label/description
    pub label: Option<String>,
}

impl Event for SaveGameRequested {}

/// Request to load game state from a slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadGameRequested {
    /// Save slot name to load from
    pub slot: String,
}

impl Event for LoadGameRequested {}

/// Request to delete a save slot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteSaveRequested {
    /// Save slot name to delete
    pub slot: String,
}

impl Event for DeleteSaveRequested {}

/// Request to list all available saves
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListSavesRequested;

impl Event for ListSavesRequested {}

/// Request to get metadata for a specific save
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetSaveMetadataRequested {
    /// Save slot name
    pub slot: String,
}

impl Event for GetSaveMetadataRequested {}

/// Request to create an auto-save
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveRequested {
    /// Optional reason for auto-save (e.g., "level_complete", "checkpoint")
    pub reason: Option<String>,
}

impl Event for AutoSaveRequested {}

// ============================================================================
// State Events (notify state changes)
// ============================================================================

/// Game state has been saved successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameSaved {
    /// The save slot used
    pub slot: String,
    /// Metadata of the saved game
    pub metadata: SaveMetadata,
    /// Optional label/description
    pub label: Option<String>,
}

impl Event for GameSaved {}

/// Game state has been loaded successfully
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameLoaded {
    /// The save slot loaded from
    pub slot: String,
    /// Metadata of the loaded game
    pub metadata: SaveMetadata,
}

impl Event for GameLoaded {}

/// A save slot has been deleted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveDeleted {
    /// The deleted save slot name
    pub slot: String,
}

impl Event for SaveDeleted {}

/// List of available saves retrieved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavesListed {
    /// List of save metadata
    pub saves: Vec<SaveMetadata>,
}

impl Event for SavesListed {}

/// Save metadata retrieved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveMetadataRetrieved {
    /// The save slot queried
    pub slot: String,
    /// Retrieved metadata
    pub metadata: SaveMetadata,
}

impl Event for SaveMetadataRetrieved {}

/// Auto-save completed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveCompleted {
    /// The auto-save slot used
    pub slot: String,
    /// Metadata of the auto-saved game
    pub metadata: SaveMetadata,
    /// Reason for the auto-save
    pub reason: Option<String>,
}

impl Event for AutoSaveCompleted {}

/// Save/Load operation failed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveLoadFailed {
    /// Type of operation that failed
    pub operation: String,
    /// The slot involved (if applicable)
    pub slot: Option<String>,
    /// Error message
    pub error: String,
}

impl Event for SaveLoadFailed {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_game_requested() {
        let event = SaveGameRequested {
            slot: "player_save_1".to_string(),
            label: Some("Chapter 3 Complete".to_string()),
        };
        assert_eq!(event.slot, "player_save_1");
        assert_eq!(event.label, Some("Chapter 3 Complete".to_string()));
    }

    #[test]
    fn test_load_game_requested() {
        let event = LoadGameRequested {
            slot: "player_save_1".to_string(),
        };
        assert_eq!(event.slot, "player_save_1");
    }

    #[test]
    fn test_auto_save_requested() {
        let event = AutoSaveRequested {
            reason: Some("checkpoint".to_string()),
        };
        assert_eq!(event.reason, Some("checkpoint".to_string()));
    }

    #[test]
    fn test_game_saved() {
        let metadata = SaveMetadata {
            version: 1,
            slot: "test_slot".to_string(),
            timestamp: std::time::SystemTime::now(),
            size_bytes: 1024,
        };
        let event = GameSaved {
            slot: "test_slot".to_string(),
            metadata: metadata.clone(),
            label: None,
        };
        assert_eq!(event.slot, "test_slot");
        assert_eq!(event.metadata.slot, metadata.slot);
    }

    #[test]
    fn test_save_load_failed() {
        let event = SaveLoadFailed {
            operation: "save".to_string(),
            slot: Some("test_slot".to_string()),
            error: "Disk full".to_string(),
        };
        assert_eq!(event.operation, "save");
        assert_eq!(event.slot, Some("test_slot".to_string()));
        assert_eq!(event.error, "Disk full");
    }
}
