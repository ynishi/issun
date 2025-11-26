//! Save/Load resources

use bevy::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

use super::components::SaveMetadata;

/// Global configuration for save/load system
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct SaveLoadConfig {
    /// Save directory (default: "./saves")
    pub save_directory: String,

    /// Auto-save enabled
    pub enable_auto_save: bool,

    /// Auto-save period in days (default: 1)
    pub auto_save_period: u32,

    /// Max save slots (default: 10)
    pub max_save_slots: usize,

    /// Quicksave slot name (default: "quicksave")
    pub quicksave_slot: String,
}

impl Default for SaveLoadConfig {
    fn default() -> Self {
        Self {
            save_directory: "./saves".to_string(),
            enable_auto_save: false,
            auto_save_period: 1,
            max_save_slots: 10,
            quicksave_slot: "quicksave".to_string(),
        }
    }
}

impl SaveLoadConfig {
    /// Get full path for a save slot
    pub fn slot_path(&self, slot_name: &str) -> PathBuf {
        PathBuf::from(&self.save_directory).join(format!("{}.ron", slot_name))
    }

    /// Ensure save directory exists
    pub fn ensure_save_directory(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.save_directory)
    }
}

/// Information about a save slot
#[derive(Clone, Debug, Reflect)]
#[reflect(opaque)]
pub struct SaveSlotInfo {
    pub slot_name: String,
    pub file_path: String,
    pub metadata: SaveMetadata,
    pub file_size: u64,
}

impl SaveSlotInfo {
    /// Create save slot info from file path
    pub fn from_file(file_path: impl Into<PathBuf>) -> std::io::Result<Self> {
        let path: PathBuf = file_path.into();
        let slot_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Try to read metadata from file
        let content = std::fs::read_to_string(&path)?;
        let metadata = ron::from_str::<SaveMetadata>(&content)
            .unwrap_or_else(|_| SaveMetadata::new(&slot_name));

        let file_size = std::fs::metadata(&path)?.len();

        Ok(Self {
            slot_name,
            file_path: path.to_string_lossy().to_string(),
            metadata,
            file_size,
        })
    }
}

/// Track available save slots
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct SaveSlotRegistry {
    /// Available save slots with metadata
    #[reflect(ignore)]
    slots: HashMap<String, SaveSlotInfo>,
}

impl SaveSlotRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Add or update a save slot
    pub fn add_slot(&mut self, slot_info: SaveSlotInfo) {
        self.slots.insert(slot_info.slot_name.clone(), slot_info);
    }

    /// Remove a save slot
    pub fn remove_slot(&mut self, slot_name: &str) {
        self.slots.remove(slot_name);
    }

    /// Get save slot info
    pub fn get_slot(&self, slot_name: &str) -> Option<&SaveSlotInfo> {
        self.slots.get(slot_name)
    }

    /// Get all save slots
    pub fn all_slots(&self) -> Vec<&SaveSlotInfo> {
        self.slots.values().collect()
    }

    /// Check if slot exists
    pub fn has_slot(&self, slot_name: &str) -> bool {
        self.slots.contains_key(slot_name)
    }

    /// Refresh registry from disk
    pub fn refresh_from_disk(&mut self, config: &SaveLoadConfig) -> std::io::Result<()> {
        self.slots.clear();

        // Ensure directory exists
        config.ensure_save_directory()?;

        // Scan save directory
        let save_dir = PathBuf::from(&config.save_directory);
        if !save_dir.exists() {
            return Ok(());
        }

        for entry in std::fs::read_dir(save_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Only .ron files
            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Ok(slot_info) = SaveSlotInfo::from_file(&path) {
                    self.add_slot(slot_info);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = SaveLoadConfig::default();
        assert_eq!(config.save_directory, "./saves");
        assert!(!config.enable_auto_save);
        assert_eq!(config.auto_save_period, 1);
        assert_eq!(config.max_save_slots, 10);
        assert_eq!(config.quicksave_slot, "quicksave");
    }

    #[test]
    fn test_config_slot_path() {
        let config = SaveLoadConfig::default();
        let path = config.slot_path("slot_1");
        assert_eq!(path, PathBuf::from("./saves/slot_1.ron"));
    }

    #[test]
    fn test_save_slot_registry() {
        let mut registry = SaveSlotRegistry::new();

        let slot_info = SaveSlotInfo {
            slot_name: "slot_1".to_string(),
            file_path: "./saves/slot_1.ron".to_string(),
            metadata: SaveMetadata::new("slot_1"),
            file_size: 1024,
        };

        registry.add_slot(slot_info);
        assert!(registry.has_slot("slot_1"));
        assert_eq!(registry.all_slots().len(), 1);

        registry.remove_slot("slot_1");
        assert!(!registry.has_slot("slot_1"));
    }
}
