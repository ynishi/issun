//! Game builder for ISSUN

use crate::context::ResourceContext;
use crate::error::{IssunError, Result};
use crate::plugin::{Plugin, PluginBuilder};
use crate::service::Service;
use crate::system::System;
use std::any::TypeId;
use std::collections::{HashMap, HashSet};

/// Game builder for composing plugins and configuring the game
pub struct GameBuilder {
    plugins: Vec<Box<dyn Plugin>>,
    plugin_names: HashSet<String>,
    runtime_resources: HashMap<TypeId, Box<dyn RuntimeResourceEntry>>,
    extra_services: Vec<Box<dyn Service>>,
    extra_systems: Vec<Box<dyn System>>,
}

impl GameBuilder {
    /// Create a new game builder
    pub fn new() -> Self {
        Self {
            plugins: Vec::new(),
            plugin_names: HashSet::new(),
            runtime_resources: HashMap::new(),
            extra_services: Vec::new(),
            extra_systems: Vec::new(),
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

    /// Register a mutable runtime resource
    ///
    /// These resources live in `ResourceContext` and can be mutated by systems.
    pub fn with_resource<T: 'static + Send + Sync>(mut self, resource: T) -> Self {
        self.runtime_resources
            .insert(TypeId::of::<T>(), Box::new(resource));
        self
    }

    /// Register an additional stateless service
    pub fn with_service(mut self, service: impl Service + 'static) -> Self {
        self.extra_services.push(Box::new(service));
        self
    }

    /// Register an additional system
    pub fn with_system(mut self, system: impl System + 'static) -> Self {
        self.extra_systems.push(Box::new(system));
        self
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

        // Combine services/systems from plugins and manual registrations
        let mut all_services = plugin_builder.services;
        all_services.extend(self.extra_services.into_iter());

        let mut all_systems = plugin_builder.systems;
        all_systems.extend(self.extra_systems.into_iter());

        // Legacy context (for backward compatibility)
        let mut context = crate::context::Context::new();

        // New contexts
        let mut resource_context = crate::context::ResourceContext::new();
        let mut service_context = crate::context::ServiceContext::new();
        let mut system_context = crate::context::SystemContext::new();

        // Register services in both contexts (cloned for new architecture)
        for service in all_services {
            let cloned = service.as_ref().clone_box();
            context.register_service(service);
            service_context.register(cloned);
        }

        // Register systems into SystemContext
        for system in all_systems {
            system_context.register_boxed(system);
        }

        // Register runtime resources
        for (_, entry) in self.runtime_resources.into_iter() {
            entry.insert(&mut resource_context);
        }

        // Register resources from plugins (read-only data)
        *context.resources_mut() = plugin_builder.resources;

        Ok(Game {
            resources: resource_context,
            services: service_context,
            systems: system_context,
            context,
            entities: plugin_builder.entities,
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

/// Trait object wrapper to insert concrete resources into ResourceContext
trait RuntimeResourceEntry: Send {
    fn insert(self: Box<Self>, ctx: &mut ResourceContext);
}

impl<T: 'static + Send + Sync> RuntimeResourceEntry for T {
    fn insert(self: Box<Self>, ctx: &mut ResourceContext) {
        ctx.insert(*self);
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

/// Game instance with partitioned contexts (Proposal C)
///
/// This struct holds the three specialized contexts:
/// - `resources`: Global shared state (Resources)
/// - `services`: Stateless domain logic (Services)
/// - `systems`: Stateful orchestration (Systems)
pub struct Game {
    /// Resource context - global shared state
    pub resources: crate::context::ResourceContext,
    /// Service context - stateless domain logic
    pub services: crate::context::ServiceContext,
    /// System context - stateful orchestration
    pub systems: crate::context::SystemContext,

    /// Legacy: Old unified context (deprecated)
    /// Kept for backward compatibility. Will be removed in future versions.
    #[deprecated(since = "0.2.0", note = "Use resources, services, and systems instead")]
    pub context: crate::context::Context,

    /// Registered entities from plugins
    pub entities: HashMap<String, Box<dyn crate::entity::Entity>>,
    /// Registered assets from plugins
    pub assets: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl Game {
    /// Get the legacy game context (deprecated)
    #[deprecated(
        since = "0.2.0",
        note = "Use resources(), services(), or systems() instead"
    )]
    pub fn context(&self) -> &crate::context::Context {
        &self.context
    }

    /// Get mutable reference to legacy game context (deprecated)
    #[deprecated(
        since = "0.2.0",
        note = "Use resources_mut(), services_mut(), or systems_mut() instead"
    )]
    pub fn context_mut(&mut self) -> &mut crate::context::Context {
        &mut self.context
    }

    /// Get reference to resource context
    pub fn resources(&self) -> &crate::context::ResourceContext {
        &self.resources
    }

    /// Get mutable reference to resource context
    pub fn resources_mut(&mut self) -> &mut crate::context::ResourceContext {
        &mut self.resources
    }

    /// Get reference to service context
    pub fn services(&self) -> &crate::context::ServiceContext {
        &self.services
    }

    /// Get reference to system context
    pub fn systems(&self) -> &crate::context::SystemContext {
        &self.systems
    }

    /// Get mutable reference to system context
    pub fn systems_mut(&mut self) -> &mut crate::context::SystemContext {
        &mut self.systems
    }

    /// Run the game (TODO: implement game loop)
    pub fn run(self) -> Result<()> {
        // TODO: Implement game loop
        println!(
            "Game running with {} services and {} systems...",
            self.services.len(),
            self.systems.len()
        );
        Ok(())
    }
}
