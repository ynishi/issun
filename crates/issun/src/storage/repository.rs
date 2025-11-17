//! Save repository trait for ISSUN

use crate::error::Result;
use crate::storage::save_data::{SaveData, SaveMetadata};
use async_trait::async_trait;

/// Save repository trait for game persistence
///
/// Implementors provide different storage backends (JSON, RON, etc.)
#[async_trait]
pub trait SaveRepository: Send + Sync {
    /// Save game data
    async fn save(&self, data: &SaveData) -> Result<()>;

    /// Load game data
    async fn load(&self, slot: &str) -> Result<SaveData>;

    /// List all available saves
    async fn list_saves(&self) -> Result<Vec<SaveMetadata>>;

    /// Delete a save slot
    async fn delete(&self, slot: &str) -> Result<()>;

    /// Check if a save exists
    async fn exists(&self, slot: &str) -> bool;

    /// Get save metadata without loading full data
    async fn get_metadata(&self, slot: &str) -> Result<SaveMetadata>;
}
