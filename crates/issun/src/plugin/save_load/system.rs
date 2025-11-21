//! Save/Load plugin system implementation

use super::events::*;
use super::hook::SaveLoadHook;
use super::plugin::{SaveFormat, SaveLoadConfig};
use crate::context::{Context, ResourceContext, ServiceContext};
use crate::error::{IssunError, Result};
use crate::event::EventBus;
use crate::storage::json_repository::JsonSaveRepository;
use crate::storage::repository::SaveRepository;
use crate::storage::ron_repository::RonSaveRepository;
use crate::storage::save_data::SaveData;
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;
use std::sync::Arc;

/// System that handles save/load operations
///
/// This system processes save/load events and coordinates with:
/// - SaveRepository for actual file operations
/// - SaveLoadHook for customization
/// - EventBus for publishing state events
/// - ResourceContext for game state access
pub struct SaveLoadSystem {
    hook: Arc<dyn SaveLoadHook>,
    config: SaveLoadConfig,
    repository: Option<Arc<dyn SaveRepository>>,
}

impl SaveLoadSystem {
    /// Create a new SaveLoadSystem with the given hook and config
    pub fn new(hook: Arc<dyn SaveLoadHook>, config: SaveLoadConfig) -> Self {
        Self {
            hook,
            config,
            repository: None,
        }
    }

    /// Process all save/load events
    pub async fn process_events(
        &mut self,
        _services: &ServiceContext,
        resources: &mut ResourceContext,
    ) {
        // Ensure repository is initialized
        if self.repository.is_none() {
            if let Err(e) = self.initialize_repository().await {
                eprintln!("Failed to initialize save repository: {}", e);
                return;
            }
        }

        self.process_save_requests(resources).await;
        self.process_load_requests(resources).await;
        self.process_delete_requests(resources).await;
        self.process_list_saves_requests(resources).await;
        self.process_metadata_requests(resources).await;
        self.process_auto_save_requests(resources).await;
    }

    /// Initialize the save repository based on config
    async fn initialize_repository(&mut self) -> Result<()> {
        let repository: Arc<dyn SaveRepository> = match self.config.format {
            SaveFormat::Json => {
                Arc::new(JsonSaveRepository::new(&self.config.save_directory).await?)
            }
            SaveFormat::Ron => Arc::new(RonSaveRepository::new(&self.config.save_directory).await?),
        };
        self.repository = Some(repository);
        Ok(())
    }

    /// Get the repository, ensuring it's initialized
    fn get_repository(&self) -> Result<Arc<dyn SaveRepository>> {
        self.repository
            .clone()
            .ok_or_else(|| IssunError::Plugin("Save repository not initialized".to_string()))
    }

    /// Process save game requests
    async fn process_save_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<SaveGameRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            if let Err(e) = self.handle_save_request(&request, resources).await {
                let error_event = SaveLoadFailed {
                    operation: "save".to_string(),
                    slot: Some(request.slot.clone()),
                    error: e.to_string(),
                };
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(error_event);
                }
                self.hook
                    .on_save_failed(&request.slot, &e.to_string(), resources)
                    .await;
            }
        }
    }

    /// Process load game requests
    async fn process_load_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<LoadGameRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            if let Err(e) = self.handle_load_request(&request, resources).await {
                let error_event = SaveLoadFailed {
                    operation: "load".to_string(),
                    slot: Some(request.slot.clone()),
                    error: e.to_string(),
                };
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(error_event);
                }
                self.hook
                    .on_load_failed(&request.slot, &e.to_string(), resources)
                    .await;
            }
        }
    }

    /// Process delete save requests
    async fn process_delete_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<DeleteSaveRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            if let Err(e) = self.handle_delete_request(&request, resources).await {
                let error_event = SaveLoadFailed {
                    operation: "delete".to_string(),
                    slot: Some(request.slot.clone()),
                    error: e.to_string(),
                };
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(error_event);
                }
            }
        }
    }

    /// Process list saves requests
    async fn process_list_saves_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<ListSavesRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for _request in requests {
            if let Err(e) = self.handle_list_saves_request(resources).await {
                let error_event = SaveLoadFailed {
                    operation: "list_saves".to_string(),
                    slot: None,
                    error: e.to_string(),
                };
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(error_event);
                }
            }
        }
    }

    /// Process metadata requests
    async fn process_metadata_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<GetSaveMetadataRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            if let Err(e) = self.handle_metadata_request(&request, resources).await {
                let error_event = SaveLoadFailed {
                    operation: "get_metadata".to_string(),
                    slot: Some(request.slot.clone()),
                    error: e.to_string(),
                };
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(error_event);
                }
            }
        }
    }

    /// Process auto-save requests
    async fn process_auto_save_requests(&mut self, resources: &mut ResourceContext) {
        let requests = {
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                let reader = bus.reader::<AutoSaveRequested>();
                reader.iter().cloned().collect::<Vec<_>>()
            } else {
                Vec::new()
            }
        };

        for request in requests {
            if let Err(e) = self.handle_auto_save_request(&request, resources).await {
                let error_event = SaveLoadFailed {
                    operation: "auto_save".to_string(),
                    slot: None,
                    error: e.to_string(),
                };
                if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                    bus.publish(error_event);
                }
            }
        }
    }

    async fn handle_save_request(
        &self,
        event: &SaveGameRequested,
        resources: &mut ResourceContext,
    ) -> Result<()> {
        let repository = self.get_repository()?;

        // Create save data from current game state
        // Note: In a real implementation, you'd serialize the actual game state
        let game_state_json = serde_json::json!({
            "slot": event.slot,
            "label": event.label,
            "timestamp": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        let mut save_data = SaveData::new(&event.slot, game_state_json);

        // Call before_save hook
        if !self.hook.before_save(&mut save_data, resources).await {
            return Err(IssunError::Plugin(
                "Save operation cancelled by hook".to_string(),
            ));
        }

        // Perform the save
        repository.save(&save_data).await?;

        // Get metadata for the saved file
        let metadata = repository.get_metadata(&event.slot).await?;

        // Call after_save hook
        self.hook.after_save(&save_data, &metadata, resources).await;

        // Publish success event
        let success_event = GameSaved {
            slot: event.slot.clone(),
            metadata,
            label: event.label.clone(),
        };
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(success_event);
        }

        Ok(())
    }

    async fn handle_load_request(
        &self,
        event: &LoadGameRequested,
        resources: &mut ResourceContext,
    ) -> Result<()> {
        let repository = self.get_repository()?;

        // Get metadata first
        let metadata = repository.get_metadata(&event.slot).await?;

        // Call before_load hook
        if !self
            .hook
            .before_load(&event.slot, &metadata, resources)
            .await
        {
            return Err(IssunError::Plugin(
                "Load operation cancelled by hook".to_string(),
            ));
        }

        // Load the save data
        let save_data = repository.load(&event.slot).await?;

        // Apply loaded data to game state (simplified)
        // In a real implementation, you'd deserialize and apply the actual game state

        // Call after_load hook
        self.hook.after_load(&save_data, resources).await;

        // Publish success event
        let success_event = GameLoaded {
            slot: event.slot.clone(),
            metadata,
        };
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(success_event);
        }

        Ok(())
    }

    async fn handle_delete_request(
        &self,
        event: &DeleteSaveRequested,
        resources: &mut ResourceContext,
    ) -> Result<()> {
        let repository = self.get_repository()?;

        // Delete the save
        repository.delete(&event.slot).await?;

        // Publish success event
        let success_event = SaveDeleted {
            slot: event.slot.clone(),
        };
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(success_event);
        }

        Ok(())
    }

    async fn handle_list_saves_request(&self, resources: &mut ResourceContext) -> Result<()> {
        let repository = self.get_repository()?;

        // List all saves
        let saves = repository.list_saves().await?;

        // Publish result event
        let result_event = SavesListed { saves };
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(result_event);
        }

        Ok(())
    }

    async fn handle_metadata_request(
        &self,
        event: &GetSaveMetadataRequested,
        resources: &mut ResourceContext,
    ) -> Result<()> {
        let repository = self.get_repository()?;

        // Get metadata
        let metadata = repository.get_metadata(&event.slot).await?;

        // Publish result event
        let result_event = SaveMetadataRetrieved {
            slot: event.slot.clone(),
            metadata,
        };
        if let Some(mut bus) = resources.get_mut::<EventBus>().await {
            bus.publish(result_event);
        }

        Ok(())
    }

    async fn handle_auto_save_request(
        &self,
        event: &AutoSaveRequested,
        resources: &mut ResourceContext,
    ) -> Result<()> {
        // Ask hook for auto-save slot name
        let slot = self
            .hook
            .on_auto_save(event.reason.as_deref(), resources)
            .await;

        if let Some(slot) = slot {
            // Perform auto-save
            let save_request = SaveGameRequested {
                slot: slot.clone(),
                label: event.reason.clone(),
            };

            // Handle the save (reuse existing logic)
            self.handle_save_request(&save_request, resources).await?;

            // Get metadata for the auto-saved file
            let repository = self.get_repository()?;
            let metadata = repository.get_metadata(&slot).await?;

            // Publish auto-save success event
            let auto_save_event = AutoSaveCompleted {
                slot,
                metadata,
                reason: event.reason.clone(),
            };
            if let Some(mut bus) = resources.get_mut::<EventBus>().await {
                bus.publish(auto_save_event);
            }
        }

        Ok(())
    }
}

#[async_trait]
impl System for SaveLoadSystem {
    fn name(&self) -> &'static str {
        "save_load_system"
    }

    async fn initialize(&mut self, _ctx: &mut Context) {
        if let Err(e) = self.initialize_repository().await {
            eprintln!("Failed to initialize save repository: {}", e);
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::save_load::hook::DefaultSaveLoadHook;

    #[test]
    fn test_system_creation() {
        let hook = Arc::new(DefaultSaveLoadHook);
        let config = SaveLoadConfig::default();
        let system = SaveLoadSystem::new(hook, config);
        assert_eq!(system.name(), "save_load_system");
    }
}
