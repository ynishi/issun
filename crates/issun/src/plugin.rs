//! Plugin system for ISSUN
//!
//! Plugins allow you to compose game systems in a modular way.

use async_trait::async_trait;

/// Plugin trait for system composition
#[async_trait]
pub trait Plugin: Send + Sync {
    /// Unique identifier for this plugin
    fn name(&self) -> &'static str;

    /// Register plugin components with the GameBuilder
    fn build(&self, builder: &mut dyn PluginBuilder);

    /// List of plugins this plugin depends on
    fn dependencies(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Initialize plugin (called before build)
    async fn initialize(&mut self) {}
}

/// Builder interface for plugins to register components
pub trait PluginBuilder {
    // TODO: Add methods for registering services, assets, scenes
}
