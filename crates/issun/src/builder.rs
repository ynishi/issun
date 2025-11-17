//! Game builder for ISSUN

use crate::error::{IssunError, Result};
use crate::plugin::{Plugin, PluginBuilder};
use std::collections::{HashMap, HashSet};

/// Game builder for composing plugins and configuring the game
pub struct GameBuilder {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_names: HashSet<String>,
}

impl GameBuilder {
    /// Create a new game builder
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_names: HashSet::new(),
        }
    }

    /// Register a plugin
    pub fn with_plugin(mut self, plugin: impl Plugin + 'static) -> Result<Self> {
        let name = plugin.name().to_string();

        // Check for duplicate plugins
        if self.plugin_names.contains(&name) {
            return Err(IssunError::Plugin(format!(
                "Plugin '{}' already registered",
                name
            )));
        }

        self.plugin_names.insert(name);
        self.plugins.push(Box::new(plugin));
        Ok(self)
    }

    /// Build and run the game
    pub async fn build(mut self) -> Result<Game> {
        // Initialize plugins first
        for plugin in &mut self.plugins {
            plugin.initialize().await;
        }

        // Resolve dependencies (creates indices, not references)
        let sorted_indices = self.resolve_dependency_order()?;

        // Build plugins in dependency order
        let mut plugin_builder = DefaultPluginBuilder::new();
        for idx in sorted_indices {
            self.plugins[idx].build(&mut plugin_builder);
        }

        // Create context and register all services
        let mut context = crate::context::Context::new();
        for service in plugin_builder.services {
            context.register_service(service);
        }

        // Register resources from plugins
        *context.resources_mut() = plugin_builder.resources;

        Ok(Game {
            context,
            entities: plugin_builder.entities,
            systems: plugin_builder.systems,
            assets: plugin_builder.assets,
        })
    }

    /// Resolve plugin dependencies and return indices in topological order
    fn resolve_dependency_order(&self) -> Result<Vec<usize>> {
        let mut sorted_indices = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        for (idx, _) in self.plugins.iter().enumerate() {
            self.visit_plugin_index(idx, &mut visited, &mut visiting, &mut sorted_indices)?;
        }

        Ok(sorted_indices)
    }

    fn visit_plugin_index(
        &self,
        idx: usize,
        visited: &mut HashSet<usize>,
        visiting: &mut HashSet<usize>,
        sorted: &mut Vec<usize>,
    ) -> Result<()> {
        if visited.contains(&idx) {
            return Ok(());
        }

        if visiting.contains(&idx) {
            let name = self.plugins[idx].name().to_string();
            return Err(IssunError::CircularDependency(vec![name]));
        }

        visiting.insert(idx);

        let plugin = &self.plugins[idx];
        let name = plugin.name().to_string();

        // Visit dependencies first
        for dep_name in plugin.dependencies() {
            let dep_idx = self
                .plugins
                .iter()
                .position(|p| p.name() == dep_name)
                .ok_or_else(|| IssunError::PluginDependency {
                    plugin: name.clone(),
                    dependency: dep_name.to_string(),
                })?;

            self.visit_plugin_index(dep_idx, visited, visiting, sorted)?;
        }

        visiting.remove(&idx);
        visited.insert(idx);
        sorted.push(idx);

        Ok(())
    }
}

impl Default for GameBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Default plugin builder implementation
struct DefaultPluginBuilder {
    entities: HashMap<String, Box<dyn crate::entity::Entity>>,
    services: Vec<Box<dyn crate::service::Service>>,
    systems: Vec<Box<dyn crate::system::System>>,
    // scenes: Removed - use SceneDirector instead
    assets: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
    resources: crate::resources::Resources,
}

impl DefaultPluginBuilder {
    fn new() -> Self {
        Self {
            entities: HashMap::new(),
            services: Vec::new(),
            systems: Vec::new(),
            assets: HashMap::new(),
            resources: crate::resources::Resources::default(),
        }
    }
}

impl PluginBuilder for DefaultPluginBuilder {
    fn register_entity(&mut self, name: &str, entity: Box<dyn crate::entity::Entity>) {
        self.entities.insert(name.to_string(), entity);
    }

    fn register_service(&mut self, service: Box<dyn crate::service::Service>) {
        self.services.push(service);
    }

    fn register_system(&mut self, system: Box<dyn crate::system::System>) {
        self.systems.push(system);
    }

    // register_scene removed - use SceneDirector instead

    fn register_asset(&mut self, name: &str, asset: Box<dyn std::any::Any + Send + Sync>) {
        self.assets.insert(name.to_string(), asset);
    }

    fn resources_mut(&mut self) -> &mut crate::resources::Resources {
        &mut self.resources
    }
}

/// Game instance
pub struct Game {
    /// Game context with registered services
    pub context: crate::context::Context,
    /// Registered entities from plugins
    pub entities: HashMap<String, Box<dyn crate::entity::Entity>>,
    /// Registered systems from plugins (Application Logic)
    pub systems: Vec<Box<dyn crate::system::System>>,
    // scenes: Removed - use SceneDirector instead
    /// Registered assets from plugins
    pub assets: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Game {
    /// Get the game context
    pub fn context(&self) -> &crate::context::Context {
        &self.context
    }

    /// Get mutable reference to game context
    pub fn context_mut(&mut self) -> &mut crate::context::Context {
        &mut self.context
    }

    /// Run the game (TODO: implement game loop)
    pub fn run(self) -> Result<()> {
        // TODO: Implement game loop
        println!(
            "Game running with {} registered services...",
            self.context.service_count()
        );
        Ok(())
    }
}
