//! Save/Load systems

use bevy::prelude::*;
use moonshine_save::load::TriggerLoad;
use moonshine_save::prelude::{LoadWorld, SaveWorld};
use moonshine_save::save::TriggerSave;

use super::components::SaveMetadata;
use super::events::*;
use super::resources::{SaveLoadConfig, SaveSlotInfo, SaveSlotRegistry};

use crate::plugins::time::DayChanged;

// ============================================================================
// IssunSet::Input - Auto-Save
// ============================================================================

/// Trigger auto-saves based on configuration
///
/// Listens for DayChanged events and creates auto-saves at configured intervals
pub fn auto_save_system(
    mut commands: Commands,
    mut messages: MessageReader<DayChanged>,
    config: Res<SaveLoadConfig>,
) {
    if !config.enable_auto_save {
        return;
    }

    for msg in messages.read() {
        // Check if it's time for an auto-save
        if msg.day % config.auto_save_period == 0 {
            info!("Auto-save triggered on day {}", msg.day);

            commands.write_message(SaveRequested {
                slot_name: "auto_save".to_string(),
                metadata: Some(SaveMetadata::new("auto_save").with_game_day(msg.day)),
            });
        }
    }
}

// ============================================================================
// IssunSet::Logic - Request Processing
// ============================================================================

/// Process save requests
pub fn process_save_requests(
    mut commands: Commands,
    mut messages: MessageReader<SaveRequested>,
    config: Res<SaveLoadConfig>,
) {
    for request in messages.read() {
        info!("Processing save request for slot: {}", request.slot_name);

        // Ensure save directory exists
        if let Err(e) = config.ensure_save_directory() {
            error!("Failed to create save directory: {}", e);
            commands.write_message(SaveFailed {
                slot_name: request.slot_name.clone(),
                error: format!("Failed to create save directory: {}", e),
            });
            continue;
        }

        // Resolve file path
        let file_path = config.slot_path(&request.slot_name);

        // Create or use provided metadata
        let metadata = request
            .metadata
            .clone()
            .unwrap_or_else(|| SaveMetadata::new(&request.slot_name));

        // Spawn a metadata entity (will be saved)
        commands.spawn(metadata.clone());

        // Trigger moonshine_save
        commands.trigger_save(SaveWorld::default_into_file(&file_path));

        // Note: Actual file write happens asynchronously
        // We'll verify completion in verify_save_completion system
        info!("Save triggered for: {}", file_path.display());
    }
}

/// Process load requests
pub fn process_load_requests(
    mut commands: Commands,
    mut messages: MessageReader<LoadRequested>,
    config: Res<SaveLoadConfig>,
    registry: Res<SaveSlotRegistry>,
) {
    for request in messages.read() {
        info!("Processing load request for slot: {}", request.slot_name);

        // Verify slot exists
        if !registry.has_slot(&request.slot_name) {
            warn!("Save slot not found: {}", request.slot_name);
            commands.write_message(LoadFailed {
                slot_name: request.slot_name.clone(),
                error: format!("Save slot '{}' not found", request.slot_name),
            });
            continue;
        }

        // Resolve file path
        let file_path = config.slot_path(&request.slot_name);

        // Verify file exists
        if !file_path.exists() {
            error!("Save file not found: {}", file_path.display());
            commands.write_message(LoadFailed {
                slot_name: request.slot_name.clone(),
                error: format!("Save file not found: {}", file_path.display()),
            });
            continue;
        }

        // Trigger moonshine_save
        commands.trigger_load(LoadWorld::default_from_file(&file_path));

        // Note: Actual file read and entity spawning happens asynchronously
        // We'll verify completion in verify_load_completion system
        info!("Load triggered for: {}", file_path.display());
    }
}

/// Process delete requests
pub fn process_delete_requests(
    _commands: Commands,
    mut messages: MessageReader<DeleteSaveRequested>,
    config: Res<SaveLoadConfig>,
    mut registry: ResMut<SaveSlotRegistry>,
) {
    for request in messages.read() {
        info!("Processing delete request for slot: {}", request.slot_name);

        // Resolve file path
        let file_path = config.slot_path(&request.slot_name);

        // Delete file
        match std::fs::remove_file(&file_path) {
            Ok(_) => {
                info!("Deleted save file: {}", file_path.display());
                registry.remove_slot(&request.slot_name);
            }
            Err(e) => {
                error!("Failed to delete save file: {}", e);
            }
        }
    }
}

/// Process list requests
pub fn process_list_requests(
    mut commands: Commands,
    mut messages: MessageReader<ListSavesRequested>,
    config: Res<SaveLoadConfig>,
    mut registry: ResMut<SaveSlotRegistry>,
) {
    for _request in messages.read() {
        info!("Processing list saves request");

        // Refresh registry from disk
        if let Err(e) = registry.refresh_from_disk(&config) {
            error!("Failed to refresh save slot registry: {}", e);
            commands.write_message(SavesListed { slots: Vec::new() });
            continue;
        }

        // Return list of slots
        let slots = registry.all_slots().iter().map(|s| (*s).clone()).collect();
        commands.write_message(SavesListed { slots });
    }
}

/// Verify save completion and publish success/failure messages
///
/// This system runs after save operations to check if files were written successfully
pub fn verify_save_completion(
    mut commands: Commands,
    config: Res<SaveLoadConfig>,
    mut registry: ResMut<SaveSlotRegistry>,
    metadata_query: Query<(Entity, &SaveMetadata), Added<SaveMetadata>>,
) {
    for (entity, metadata) in metadata_query.iter() {
        let file_path = config.slot_path(&metadata.slot_name);

        // Check if file exists (indicates successful save)
        if file_path.exists() {
            // Create slot info
            match SaveSlotInfo::from_file(&file_path) {
                Ok(slot_info) => {
                    registry.add_slot(slot_info.clone());

                    commands.write_message(SaveCompleted {
                        slot_name: metadata.slot_name.clone(),
                        file_path: file_path.to_string_lossy().to_string(),
                        metadata: metadata.clone(),
                    });

                    info!("Save completed successfully: {}", metadata.slot_name);
                }
                Err(e) => {
                    error!("Failed to create slot info: {}", e);
                    commands.write_message(SaveFailed {
                        slot_name: metadata.slot_name.clone(),
                        error: format!("Failed to create slot info: {}", e),
                    });
                }
            }
        }

        // Despawn metadata entity (it was saved to file)
        commands.entity(entity).despawn();
    }
}

/// Verify load completion and publish success/failure messages
///
/// This system runs after load operations to check if entities were loaded successfully
pub fn verify_load_completion(
    mut commands: Commands,
    metadata_query: Query<&SaveMetadata, Added<SaveMetadata>>,
) {
    for metadata in metadata_query.iter() {
        commands.write_message(LoadCompleted {
            slot_name: metadata.slot_name.clone(),
            metadata: metadata.clone(),
        });

        info!("Load completed successfully: {}", metadata.slot_name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::IssunCorePlugin;
    use tempfile::TempDir;

    // Helper to create test app
    fn setup_test_app() -> (App, TempDir) {
        let temp_dir = TempDir::new().unwrap();

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.add_plugins(IssunCorePlugin);

        // Add resources
        let config = SaveLoadConfig {
            save_directory: temp_dir.path().to_string_lossy().to_string(),
            ..Default::default()
        };
        app.insert_resource(config);
        app.insert_resource(SaveSlotRegistry::new());

        // Add messages
        app.add_message::<SaveRequested>();
        app.add_message::<LoadRequested>();
        app.add_message::<DeleteSaveRequested>();
        app.add_message::<ListSavesRequested>();
        app.add_message::<SaveCompleted>();
        app.add_message::<LoadCompleted>();
        app.add_message::<SaveFailed>();
        app.add_message::<LoadFailed>();
        app.add_message::<SavesListed>();
        app.add_message::<DayChanged>();

        // Register types
        app.register_type::<SaveMetadata>();

        // Add systems
        app.add_systems(
            Update,
            (
                process_save_requests,
                process_load_requests,
                process_delete_requests,
                process_list_requests,
                verify_save_completion,
                verify_load_completion,
            )
                .chain(),
        );

        // Add moonshine_save observers
        app.add_observer(moonshine_save::prelude::save_on_default_event);
        app.add_observer(moonshine_save::prelude::load_on_default_event);

        (app, temp_dir)
    }

    #[test]
    fn test_process_list_requests() {
        let (mut app, _temp_dir) = setup_test_app();

        // Request list
        app.world_mut().write_message(ListSavesRequested);
        app.update();

        // Check response
        let mut messages = app.world_mut().resource_mut::<Messages<SavesListed>>();
        let events: Vec<_> = messages.drain().collect();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].slots.len(), 0); // No saves yet
    }

    #[test]
    fn test_process_delete_requests() {
        let (mut app, temp_dir) = setup_test_app();

        // Create a dummy save file
        let save_path = temp_dir.path().join("test_slot.ron");
        std::fs::write(&save_path, "dummy content").unwrap();

        // Request delete
        app.world_mut().write_message(DeleteSaveRequested {
            slot_name: "test_slot".to_string(),
        });
        app.update();

        // Verify file deleted
        assert!(!save_path.exists());
    }
}
