//! MOD System Plugin for ISSUN integration

use crate::context::ResourceContext;
use crate::engine::ModBridgeSystem;
use crate::event::EventBus;
use crate::modding::events::*;
use crate::modding::{ModEventSystem, ModHandle, ModLoader, PluginAction};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use crate::system::System;
use async_trait::async_trait;
use std::any::Any;

/// MOD System Plugin
///
/// Registers the MOD loading infrastructure into the ISSUN engine.
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::modding::{ModSystemPlugin};
/// use issun_mod_rhai::RhaiLoader;
///
/// let game = GameBuilder::new()
///     .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))?
///     .build()
///     .await?;
/// ```
#[derive(Default)]
pub struct ModSystemPlugin {
    loader: Option<Box<dyn ModLoader>>,
}

impl ModSystemPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the backend loader (Rhai or Wasm)
    pub fn with_loader(mut self, loader: impl ModLoader + 'static) -> Self {
        self.loader = Some(Box::new(loader));
        self
    }
}

#[async_trait]
impl Plugin for ModSystemPlugin {
    fn name(&self) -> &'static str {
        "mod_system"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_resource(ModSystemConfig::default());

        if let Some(loader) = &self.loader {
            builder.register_runtime_state(ModLoaderState {
                loader: loader.clone_box(),
                loaded_mods: Vec::new(),
            });
        }

        // Register all four systems
        builder.register_system(Box::new(ModLoadSystem));
        builder.register_system(Box::new(PluginControlSystem));
        builder.register_system(Box::new(ModEventSystem::new()));
        builder.register_system(Box::new(ModBridgeSystem::new()));
    }
}

/// Configuration for MOD system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModSystemConfig {
    pub mod_dir: String,
    pub hot_reload: bool,
    pub auto_load: bool,
}

impl Default for ModSystemConfig {
    fn default() -> Self {
        Self {
            mod_dir: "mods".to_string(),
            hot_reload: false,
            auto_load: true,
        }
    }
}

impl crate::resources::Resource for ModSystemConfig {}

/// Runtime state for MOD system
pub struct ModLoaderState {
    pub loader: Box<dyn ModLoader>,
    pub loaded_mods: Vec<ModHandle>,
}

/// System for loading and managing MODs
///
/// Processes `ModLoadRequested` and `ModUnloadRequested` events,
/// delegates to the configured `ModLoader`, and publishes result events.
struct ModLoadSystem;

impl ModLoadSystem {
    /// Update method using ResourceContext (Modern API)
    ///
    /// This method is the recommended way to update the system.
    #[allow(dead_code)]
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Step 1: Collect load requests
        let load_requests: Vec<ModLoadRequested> = {
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                event_bus
                    .reader::<ModLoadRequested>()
                    .iter()
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Step 2: Collect unload requests
        let unload_requests: Vec<ModUnloadRequested> = {
            if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
                event_bus
                    .reader::<ModUnloadRequested>()
                    .iter()
                    .cloned()
                    .collect()
            } else {
                Vec::new()
            }
        };

        // Step 3: Process load requests
        let mut load_results = Vec::new();
        if !load_requests.is_empty() {
            if let Some(mut loader_state) = resources.get_mut::<ModLoaderState>().await {
                for request in load_requests {
                    match loader_state.loader.load(&request.path) {
                        Ok(handle) => {
                            println!(
                                "[MOD System] Loaded MOD: {} v{}",
                                handle.metadata.name, handle.metadata.version
                            );
                            loader_state.loaded_mods.push(handle.clone());
                            load_results.push(Ok(handle));
                        }
                        Err(e) => {
                            eprintln!("[MOD System] Failed to load MOD {:?}: {}", request.path, e);
                            load_results.push(Err((request.path, e.to_string())));
                        }
                    }
                }
            }
        }

        // Publish load results
        if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
            for result in load_results {
                match result {
                    Ok(handle) => {
                        event_bus.publish(ModLoadedEvent { handle });
                    }
                    Err((path, error)) => {
                        event_bus.publish(ModLoadFailedEvent { path, error });
                    }
                }
            }
        }

        // Step 4: Process unload requests
        let mut unload_results: Vec<Result<String, ()>> = Vec::new();
        if !unload_requests.is_empty() {
            if let Some(mut loader_state) = resources.get_mut::<ModLoaderState>().await {
                for request in unload_requests {
                    // Find the MOD handle
                    if let Some(pos) = loader_state
                        .loaded_mods
                        .iter()
                        .position(|h| h.id == request.mod_id)
                    {
                        let handle = loader_state.loaded_mods.remove(pos);

                        match loader_state.loader.unload(&handle) {
                            Ok(_) => {
                                println!("[MOD System] Unloaded MOD: {}", request.mod_id);
                                unload_results.push(Ok(request.mod_id));
                            }
                            Err(e) => {
                                eprintln!(
                                    "[MOD System] Failed to unload MOD {}: {}",
                                    request.mod_id, e
                                );
                                // Re-add to list on failure
                                loader_state.loaded_mods.push(handle);
                            }
                        }
                    } else {
                        eprintln!("[MOD System] MOD '{}' not found", request.mod_id);
                    }
                }
            }
        }

        // Publish unload results
        if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
            for result in unload_results {
                if let Ok(mod_id) = result {
                    event_bus.publish(ModUnloadedEvent { mod_id });
                }
            }
        }
    }
}

#[async_trait]
impl System for ModLoadSystem {
    fn name(&self) -> &'static str {
        "mod_load_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// System for processing plugin control commands from MODs
///
/// Drains commands from `ModLoader`, publishes `PluginControlRequested` events,
/// and then publishes specific events for each action type.
struct PluginControlSystem;

impl PluginControlSystem {
    /// Update method using ResourceContext (Modern API)
    ///
    /// This method is the recommended way to update the system.
    #[allow(dead_code)]
    pub async fn update_resources(&mut self, resources: &mut ResourceContext) {
        // Step 1: Drain commands from loader
        let commands = {
            if let Some(mut loader_state) = resources.get_mut::<ModLoaderState>().await {
                loader_state.loader.drain_commands()
            } else {
                Vec::new()
            }
        };

        if commands.is_empty() {
            return;
        }

        // Step 2 & 3: Publish all events
        if let Some(mut event_bus) = resources.get_mut::<EventBus>().await {
            // Publish PluginControlRequested events
            for command in &commands {
                event_bus.publish(PluginControlRequested {
                    control: command.clone(),
                    source_mod: None, // TODO: Track source MOD
                });
            }

            // Publish specific action events
            for command in commands {
                match &command.action {
                    PluginAction::Enable => {
                        println!("[MOD System] Enabling plugin: {}", command.plugin_name);
                        event_bus.publish(PluginEnabledEvent {
                            plugin_name: command.plugin_name.clone(),
                        });
                    }
                    PluginAction::Disable => {
                        println!("[MOD System] Disabling plugin: {}", command.plugin_name);
                        event_bus.publish(PluginDisabledEvent {
                            plugin_name: command.plugin_name.clone(),
                        });
                    }
                    PluginAction::SetParameter { key, value } => {
                        println!(
                            "[MOD System] Setting {}.{} = {:?}",
                            command.plugin_name, key, value
                        );
                        event_bus.publish(PluginParameterChangedEvent {
                            plugin_name: command.plugin_name.clone(),
                            key: key.clone(),
                            value: value.clone(),
                        });
                    }
                    PluginAction::TriggerHook { hook_name, data } => {
                        println!(
                            "[MOD System] Triggering hook: {}.{}",
                            command.plugin_name, hook_name
                        );
                        event_bus.publish(PluginHookTriggeredEvent {
                            plugin_name: command.plugin_name.clone(),
                            hook_name: hook_name.clone(),
                            data: data.clone(),
                        });
                    }
                }
            }
        }
    }
}

#[async_trait]
impl System for PluginControlSystem {
    fn name(&self) -> &'static str {
        "plugin_control_system"
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
