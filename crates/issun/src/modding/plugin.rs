//! MOD System Plugin for ISSUN integration

use crate::modding::{ModLoader, ModHandle};
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use crate::system::System;
use crate::context::Context;
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
/// use issun::modding::{ModSystemPlugin, RhaiLoader};
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

        builder.register_system(Box::new(ModLoadSystem));
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
struct ModLoadSystem;

#[async_trait]
impl System for ModLoadSystem {
    fn name(&self) -> &'static str {
        "mod_load_system"
    }

    async fn update(&mut self, _ctx: &mut Context) {
        // TODO: Implementation will poll for ModLoadRequested events
        // and call loader.load() for each request
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
