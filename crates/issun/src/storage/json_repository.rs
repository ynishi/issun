//! JSON-based save repository

use crate::error::{IssunError, Result};
use crate::storage::repository::SaveRepository;
use crate::storage::save_data::{SaveData, SaveMetadata};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::fs;

/// JSON-based save repository
pub struct JsonSaveRepository {
    save_dir: PathBuf,
}

impl JsonSaveRepository {
    /// Create a new JSON repository
    pub async fn new(save_dir: impl AsRef<Path>) -> Result<Self> {
        let save_dir = save_dir.as_ref().to_path_buf();

        // Create directory if it doesn't exist
        if !save_dir.exists() {
            fs::create_dir_all(&save_dir).await?;
        }

        Ok(Self { save_dir })
    }

    /// Get the path for a save slot
    fn slot_path(&self, slot: &str) -> PathBuf {
        self.save_dir.join(format!("{}.json", slot))
    }
}

#[async_trait]
impl SaveRepository for JsonSaveRepository {
    async fn save(&self, data: &SaveData) -> Result<()> {
        let path = self.slot_path(&data.slot);
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| IssunError::Serialization(e.to_string()))?;

        fs::write(path, json).await?;
        Ok(())
    }

    async fn load(&self, slot: &str) -> Result<SaveData> {
        let path = self.slot_path(slot);

        if !path.exists() {
            return Err(IssunError::AssetLoad(format!(
                "Save slot '{}' not found",
                slot
            )));
        }

        let json = fs::read_to_string(path).await?;
        let data: SaveData =
            serde_json::from_str(&json).map_err(|e| IssunError::Serialization(e.to_string()))?;

        Ok(data)
    }

    async fn list_saves(&self) -> Result<Vec<SaveMetadata>> {
        let mut saves = Vec::new();
        let mut read_dir = fs::read_dir(&self.save_dir).await?;

        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(slot) = path.file_stem().and_then(|s| s.to_str()) {
                    if let Ok(metadata) = self.get_metadata(slot).await {
                        saves.push(metadata);
                    }
                }
            }
        }

        // Sort by timestamp (newest first)
        saves.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(saves)
    }

    async fn delete(&self, slot: &str) -> Result<()> {
        let path = self.slot_path(slot);

        if path.exists() {
            fs::remove_file(path).await?;
        }

        Ok(())
    }

    async fn exists(&self, slot: &str) -> bool {
        self.slot_path(slot).exists()
    }

    async fn get_metadata(&self, slot: &str) -> Result<SaveMetadata> {
        let path = self.slot_path(slot);

        if !path.exists() {
            return Err(IssunError::AssetLoad(format!(
                "Save slot '{}' not found",
                slot
            )));
        }

        let file_size = fs::metadata(&path).await?.len();
        let json = fs::read_to_string(path).await?;
        let data: SaveData =
            serde_json::from_str(&json).map_err(|e| IssunError::Serialization(e.to_string()))?;

        Ok(SaveMetadata::from_save_data(&data, file_size))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonSaveRepository::new(temp_dir.path()).await.unwrap();

        let data = SaveData::new("test_slot", serde_json::json!({"score": 100}));
        repo.save(&data).await.unwrap();

        let loaded = repo.load("test_slot").await.unwrap();
        assert_eq!(loaded.slot, "test_slot");
    }

    #[tokio::test]
    async fn test_list_saves() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonSaveRepository::new(temp_dir.path()).await.unwrap();

        let data1 = SaveData::new("slot1", serde_json::json!({"score": 100}));
        let data2 = SaveData::new("slot2", serde_json::json!({"score": 200}));

        repo.save(&data1).await.unwrap();
        repo.save(&data2).await.unwrap();

        let saves = repo.list_saves().await.unwrap();
        assert_eq!(saves.len(), 2);
    }

    #[tokio::test]
    async fn test_delete() {
        let temp_dir = TempDir::new().unwrap();
        let repo = JsonSaveRepository::new(temp_dir.path()).await.unwrap();

        let data = SaveData::new("test_slot", serde_json::json!({"score": 100}));
        repo.save(&data).await.unwrap();
        assert!(repo.exists("test_slot").await);

        repo.delete("test_slot").await.unwrap();
        assert!(!repo.exists("test_slot").await);
    }
}
