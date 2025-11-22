use super::hook::{DefaultRumorHook, RumorHook};
use super::models::{RumorRegistry, RumorState};
use super::service::RumorService;
use super::system::RumorSystem;
use issun::plugin::PluginBuilderExt;
use issun::prelude::*;
use std::sync::Arc;

/// Rumor plugin for managing rumors and their effects
pub struct RumorPlugin {
    hook: Arc<dyn RumorHook>,
}

impl RumorPlugin {
    pub fn new() -> Self {
        Self {
            hook: Arc::new(DefaultRumorHook),
        }
    }

    pub fn with_hook(mut self, hook: Arc<dyn RumorHook>) -> Self {
        self.hook = hook;
        self
    }
}

impl Default for RumorPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl Plugin for RumorPlugin {
    fn name(&self) -> &'static str {
        "rumor_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        builder.register_service(Box::new(RumorService));
        builder.register_system(Box::new(RumorSystem::new(self.hook.clone())));
        // Note: RumorRegistry and RumorState will be registered manually in main.rs
        // because they don't implement Resource/RuntimeState traits
    }
}
