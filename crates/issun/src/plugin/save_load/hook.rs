//! Save/Load plugin hooks for customization

use crate::context::ResourceContext;
use crate::storage::save_data::{SaveData, SaveMetadata};
use async_trait::async_trait;

/// Hook trait for customizing save/load behavior
///
/// Implementors can react to save/load events and add custom logic:
/// - Data validation before saving
/// - Custom compression/encryption
/// - Cloud backup integration
/// - Save file versioning
/// - Progress tracking
#[async_trait]
pub trait SaveLoadHook: Send + Sync {
    /// Called before a save operation begins
    ///
    /// This allows you to:
    /// - Validate the save data
    /// - Modify or preprocess the data
    /// - Add custom metadata
    /// - Perform security checks
    ///
    /// Return `false` to abort the save operation.
    ///
    /// # Arguments
    ///
    /// * `save_data` - The save data that will be written
    /// * `resources` - Access to game resources
    async fn before_save(
        &self,
        _save_data: &mut SaveData,
        _resources: &mut ResourceContext,
    ) -> bool {
        true // Allow save by default
    }

    /// Called after a save operation completes successfully
    ///
    /// Use this to:
    /// - Trigger cloud backup
    /// - Update achievement progress
    /// - Log save events
    /// - Clean up old auto-saves
    ///
    /// # Arguments
    ///
    /// * `save_data` - The save data that was written
    /// * `metadata` - Metadata of the saved file
    /// * `resources` - Access to game resources
    async fn after_save(
        &self,
        _save_data: &SaveData,
        _metadata: &SaveMetadata,
        _resources: &mut ResourceContext,
    ) {
        // Default: no action
    }

    /// Called before a load operation begins
    ///
    /// This allows you to:
    /// - Perform security validation
    /// - Check save file integrity
    /// - Handle version migration
    ///
    /// Return `false` to abort the load operation.
    ///
    /// # Arguments
    ///
    /// * `slot` - The save slot being loaded
    /// * `metadata` - Metadata of the save file
    /// * `resources` - Access to game resources
    async fn before_load(
        &self,
        _slot: &str,
        _metadata: &SaveMetadata,
        _resources: &mut ResourceContext,
    ) -> bool {
        true // Allow load by default
    }

    /// Called after a load operation completes successfully
    ///
    /// Use this to:
    /// - Apply save data to game state
    /// - Trigger post-load events
    /// - Update UI state
    /// - Log load events
    ///
    /// # Arguments
    ///
    /// * `save_data` - The loaded save data
    /// * `resources` - Access to game resources
    async fn after_load(&self, _save_data: &SaveData, _resources: &mut ResourceContext) {
        // Default: no action
    }

    /// Called when a save operation fails
    ///
    /// Use this to:
    /// - Log error details
    /// - Retry with different settings
    /// - Show user-friendly error messages
    ///
    /// # Arguments
    ///
    /// * `slot` - The save slot that failed
    /// * `error` - The error that occurred
    /// * `resources` - Access to game resources
    async fn on_save_failed(&self, _slot: &str, _error: &str, _resources: &mut ResourceContext) {
        // Default: no action
    }

    /// Called when a load operation fails
    ///
    /// Use this to:
    /// - Handle corrupted saves
    /// - Suggest recovery options
    /// - Log error details
    ///
    /// # Arguments
    ///
    /// * `slot` - The save slot that failed to load
    /// * `error` - The error that occurred
    /// * `resources` - Access to game resources
    async fn on_load_failed(&self, _slot: &str, _error: &str, _resources: &mut ResourceContext) {
        // Default: no action
    }

    /// Called when auto-save triggers
    ///
    /// Use this to:
    /// - Customize auto-save slot naming
    /// - Decide whether to auto-save based on game state
    /// - Manage auto-save frequency
    ///
    /// Return the slot name to use for auto-save, or None to skip.
    ///
    /// # Arguments
    ///
    /// * `reason` - Optional reason for auto-save
    /// * `resources` - Access to game resources
    async fn on_auto_save(
        &self,
        reason: Option<&str>,
        _resources: &ResourceContext,
    ) -> Option<String> {
        // Default auto-save slot naming
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        match reason {
            Some(r) => Some(format!("auto_{}_{}", r, timestamp)),
            None => Some(format!("auto_{}", timestamp)),
        }
    }
}

/// Default no-op implementation of SaveLoadHook
///
/// This implementation does nothing and allows all operations.
/// Use this as a base when you only want to customize specific hooks.
pub struct DefaultSaveLoadHook;

#[async_trait]
impl SaveLoadHook for DefaultSaveLoadHook {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::save_data::{SaveData, SaveMetadata};
    use std::time::SystemTime;

    #[tokio::test]
    async fn test_default_hook_allows_operations() {
        let hook = DefaultSaveLoadHook;
        let mut resources = ResourceContext::new();

        // Test save operations
        let mut save_data = SaveData::new("test", serde_json::Value::Null);
        assert!(hook.before_save(&mut save_data, &mut resources).await);

        let metadata = SaveMetadata {
            version: 1,
            slot: "test".to_string(),
            timestamp: SystemTime::now(),
            size_bytes: 100,
        };
        hook.after_save(&save_data, &metadata, &mut resources).await;

        // Test load operations
        assert!(hook.before_load("test", &metadata, &mut resources).await);
        hook.after_load(&save_data, &mut resources).await;

        // Test error handling
        hook.on_save_failed("test", "error", &mut resources).await;
        hook.on_load_failed("test", "error", &mut resources).await;

        // Test auto-save
        let auto_slot = hook.on_auto_save(Some("checkpoint"), &resources).await;
        assert!(auto_slot.is_some());
        assert!(auto_slot.unwrap().contains("auto_checkpoint"));
    }
}
