//! Save/Load messages

use bevy::prelude::*;

use super::components::SaveMetadata;
use super::resources::SaveSlotInfo;

// ============================================================================
// Command Messages (Requests)
// ============================================================================

/// Request to save game
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SaveRequested {
    pub slot_name: String,
    pub metadata: Option<SaveMetadata>,
}

/// Request to load game
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct LoadRequested {
    pub slot_name: String,
}

/// Request to delete save
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct DeleteSaveRequested {
    pub slot_name: String,
}

/// Request to list available saves
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct ListSavesRequested;

// ============================================================================
// State Messages (Notifications)
// ============================================================================

/// Save succeeded
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SaveCompleted {
    pub slot_name: String,
    pub file_path: String,
    pub metadata: SaveMetadata,
}

/// Load succeeded
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct LoadCompleted {
    pub slot_name: String,
    pub metadata: SaveMetadata,
}

/// Save failed
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SaveFailed {
    pub slot_name: String,
    pub error: String,
}

/// Load failed
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct LoadFailed {
    pub slot_name: String,
    pub error: String,
}

/// List of available saves
#[derive(Message, Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SavesListed {
    #[reflect(ignore)]
    pub slots: Vec<SaveSlotInfo>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_save_requested() {
        let message = SaveRequested {
            slot_name: "slot_1".to_string(),
            metadata: None,
        };
        assert_eq!(message.slot_name, "slot_1");
    }

    #[test]
    fn test_load_requested() {
        let message = LoadRequested {
            slot_name: "slot_1".to_string(),
        };
        assert_eq!(message.slot_name, "slot_1");
    }

    #[test]
    fn test_delete_save_requested() {
        let message = DeleteSaveRequested {
            slot_name: "slot_1".to_string(),
        };
        assert_eq!(message.slot_name, "slot_1");
    }
}
