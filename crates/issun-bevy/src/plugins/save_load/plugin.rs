//! Save/Load plugin definition

use bevy::prelude::*;

use super::components::SaveMetadata;
use super::events::*;
use super::resources::{SaveLoadConfig, SaveSlotInfo, SaveSlotRegistry};
use super::systems::*;

use crate::IssunSet;

/// Save/Load plugin (wrapper around moonshine_save)
///
/// Provides ergonomic save/load commands using moonshine_save library:
/// - Declarative persistence (`#[require(Save)]` on components)
/// - Type-safe serialization (Reflect-based)
/// - Entity reference mapping (automatic)
/// - Model/View separation (save logic, not visuals)
/// - Slot management (multiple save files)
/// - Error handling and notifications
///
/// # Example
///
/// ```ignore
/// use bevy::prelude::*;
/// use issun_bevy::plugins::save_load::{SaveLoadPlugin, SaveLoadConfig};
///
/// App::new()
///     .add_plugins(SaveLoadPlugin::default())
///     .run();
///
/// // Or with custom config
/// App::new()
///     .add_plugins(SaveLoadPlugin::default().with_config(SaveLoadConfig {
///         save_directory: "./my_saves".into(),
///         enable_auto_save: true,
///         auto_save_period: 1,
///         max_save_slots: 20,
///         quicksave_slot: "quicksave".into(),
///     }))
///     .run();
/// ```
#[derive(Default)]
pub struct SaveLoadPlugin {
    pub config: SaveLoadConfig,
}

impl SaveLoadPlugin {
    /// Create plugin with custom config
    pub fn with_config(mut self, config: SaveLoadConfig) -> Self {
        self.config = config;
        self
    }

    /// Enable auto-save
    pub fn with_auto_save(mut self, period_days: u32) -> Self {
        self.config.enable_auto_save = true;
        self.config.auto_save_period = period_days;
        self
    }

    /// Set save directory
    pub fn with_save_directory(mut self, dir: impl Into<String>) -> Self {
        self.config.save_directory = dir.into();
        self
    }

    /// Set max save slots
    pub fn with_max_slots(mut self, max_slots: usize) -> Self {
        self.config.max_save_slots = max_slots;
        self
    }
}

impl Plugin for SaveLoadPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(self.config.clone());
        app.insert_resource(SaveSlotRegistry::new());

        // Messages (Bevy 0.17: buffered events)
        app.add_message::<SaveRequested>()
            .add_message::<LoadRequested>()
            .add_message::<DeleteSaveRequested>()
            .add_message::<ListSavesRequested>()
            .add_message::<SaveCompleted>()
            .add_message::<LoadCompleted>()
            .add_message::<SaveFailed>()
            .add_message::<LoadFailed>()
            .add_message::<SavesListed>();

        // Type registration (⚠️ CRITICAL: All types must be registered)
        // Resources
        app.register_type::<SaveLoadConfig>()
            .register_type::<SaveSlotRegistry>();

        // Components
        app.register_type::<SaveMetadata>();

        // Types
        app.register_type::<SaveSlotInfo>();

        // Messages
        app.register_type::<SaveRequested>()
            .register_type::<LoadRequested>()
            .register_type::<DeleteSaveRequested>()
            .register_type::<ListSavesRequested>()
            .register_type::<SaveCompleted>()
            .register_type::<LoadCompleted>()
            .register_type::<SaveFailed>()
            .register_type::<LoadFailed>()
            .register_type::<SavesListed>();

        // Systems (using IssunSet from core plugin)
        app.add_systems(
            Update,
            (
                // IssunSet::Input - Auto-save
                auto_save_system.in_set(IssunSet::Input),
            ),
        );

        app.add_systems(
            Update,
            (
                // IssunSet::Logic - Request processing (chained order)
                process_save_requests,
                process_load_requests,
                process_delete_requests,
                process_list_requests,
            )
                .chain()
                .in_set(IssunSet::Logic),
        );

        app.add_systems(
            Update,
            (
                // IssunSet::PostLogic - Verification
                verify_save_completion,
                verify_load_completion,
            )
                .chain()
                .in_set(IssunSet::PostLogic),
        );

        // ⚠️ CRITICAL: Register moonshine_save observers
        app.add_observer(moonshine_save::prelude::save_on_default_event);
        app.add_observer(moonshine_save::prelude::load_on_default_event);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_default() {
        let plugin = SaveLoadPlugin::default();
        assert_eq!(plugin.config.save_directory, "./saves");
        assert!(!plugin.config.enable_auto_save);
    }

    #[test]
    fn test_plugin_with_config() {
        let config = SaveLoadConfig {
            save_directory: "./my_saves".to_string(),
            enable_auto_save: true,
            auto_save_period: 2,
            max_save_slots: 20,
            quicksave_slot: "quick".to_string(),
        };

        let plugin = SaveLoadPlugin::default().with_config(config);
        assert_eq!(plugin.config.save_directory, "./my_saves");
        assert!(plugin.config.enable_auto_save);
        assert_eq!(plugin.config.auto_save_period, 2);
    }

    #[test]
    fn test_plugin_builder() {
        let plugin = SaveLoadPlugin::default()
            .with_save_directory("./test_saves")
            .with_auto_save(3)
            .with_max_slots(15);

        assert_eq!(plugin.config.save_directory, "./test_saves");
        assert!(plugin.config.enable_auto_save);
        assert_eq!(plugin.config.auto_save_period, 3);
        assert_eq!(plugin.config.max_save_slots, 15);
    }

    #[test]
    fn test_plugin_builds() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(crate::IssunCorePlugin);
        app.add_plugins(SaveLoadPlugin::default());

        // Verify resources are registered
        assert!(app.world().contains_resource::<SaveLoadConfig>());
        assert!(app.world().contains_resource::<SaveSlotRegistry>());
    }
}
