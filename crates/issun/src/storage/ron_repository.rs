//! RON-based save repository

use crate::error::{IssunError, Result};
use crate::storage::repository::SaveRepository;
use crate::storage::save_data::{SaveData, SaveMetadata};
use std::fs;
use std::path::{Path, PathBuf};

/// RON-based save repository
pub struct RonSaveRepository {
    save_dir: PathBuf,
}

impl RonSaveRepository {
    /// Create a new RON repository
    pub fn new(save_dir: impl AsRef<Path>) -> Result<Self> {
        let save_dir = save_dir.as_ref().to_path_buf();
        
        // Create directory if it doesn't exist
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir)?;
        }

        Ok(Self { save_dir })
    }

    /// Get the path for a save slot
    fn slot_path(&self, slot: &str) -> PathBuf {
        self.save_dir.join(format!("{}.ron", slot))
    }
}

impl SaveRepository for RonSaveRepository {
    fn save(&self, data: &SaveData) -> Result<()> {
        let path = self.slot_path(&data.slot);
        let ron = ron::ser::to_string_pretty(data, ron::ser::PrettyConfig::default())
            .map_err(|e| IssunError::Serialization(e.to_string()))?;
        
        fs::write(path, ron)?;
        Ok(())
    }

    fn load(&self, slot: &str) -> Result<SaveData> {
        let path = self.slot_path(slot);
        
        if !path.exists() {
            return Err(IssunError::AssetLoad(format!(
                "Save slot '{}' not found",
                slot
            )));
        }

        let ron = fs::read_to_string(path)?;
        let data: SaveData = ron::from_str(&ron)
            .map_err(|e| IssunError::Serialization(e.to_string()))?;
        
        Ok(data)
    }

    fn list_saves(&self) -> Result<Vec<SaveMetadata>> {
        let mut saves = Vec::new();

        for entry in fs::read_dir(&self.save_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Some(slot) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(metadata) = self.get_metadata(slot) {
                        saves.push(metadata);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    fn delete(&self, slot: &str) -> Result<()> {
        let path = self.slot_path(slot);
        
        if path.exists() {
            fs::remove_file(path)?;
        }

        Ok(())
    }

    fn exists(&self, slot: &str) -> bool {
        self.slot_path(slot).exists()
    }

    fn get_metadata(&self, slot: &str) -> Result<SaveMetadata> {
        let path = self.slot_path(slot);
        
        if !path.exists() {
            return Err(IssunError::AssetLoad(format!(
                "Save slot '{}' not found",
                slot
            )));
        }

        let file_size = fs::metadata(&path)?.len();
        let ron = fs::read_to_string(path)?;
        let data: SaveData = ron::from_str(&ron)
            .map_err(|e| IssunError::Serialization(e.to_string()))?;

        Ok(SaveMetadata::from_save_data(&data, file_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let repo = RonSaveRepository::new(temp_dir.path()).unwrap();

        let data = SaveData::new("test_slot", serde_json::json!({"score": 100}));
        repo.save(&data).unwrap();

        let loaded = repo.load("test_slot").unwrap();
        assert_eq!(loaded.slot, "test_slot");
    }

    #[test]
    fn test_exists() {
        let temp_dir = TempDir::new().unwrap();
        let repo = RonSaveRepository::new(temp_dir.path()).unwrap();

        assert!(!repo.exists("nonexistent"));

        let data = SaveData::new("test_slot", serde_json::json!({"score": 100}));
        repo.save(&data).unwrap();

        assert!(repo.exists("test_slot"));
    }
}
