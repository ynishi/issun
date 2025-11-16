//! Save repository trait for ISSUN

use crate::error::Result;
use crate::storage::save_data::{SaveData, SaveMetadata};

/// Save repository trait for game persistence
///
/// Implementors provide different storage backends (JSON, RON, etc.)
pub trait SaveRepository: Send + Sync {
    /// Save game data
    fn save(&self, data: &SaveData) -> Result<()>;

    /// Load game data
    fn load(&self, slot: &str) -> Result<SaveData>;

    /// List all available saves
    fn list_saves(&self) -> Result<Vec<SaveMetadata>>;

    /// Delete a save slot
    fn delete(&self, slot: &str) -> Result<()>;

    /// Check if a save exists
    fn exists(&self, slot: &str) -> bool;

    /// Get save metadata without loading full data
    fn get_metadata(&self, slot: &str) -> Result<SaveMetadata>;
}
