//! Plugin system for ISSUN
//!
//! Plugins allow you to compose game systems in a modular way.
//!
//! # Built-in Plugins (Future)
//!
//! ISSUN will provide optional built-in plugins for common game systems:
//! - `TurnBasedCombatPlugin`: Turn-based combat with damage calculation, combat log
//! - `InventoryPlugin`: Item management, equipment system
//! - `LootPlugin`: Drop generation, rarity system
//! - `DungeonPlugin`: Floor progression, room generation
//! - `BuffPlugin`: Buff/debuff management
//!
//! # Usage
//!
//! ```ignore
//! use issun::plugin::{Plugin, TurnBasedCombatPlugin};
//!
//! let game = GameBuilder::new()
//!     .add_plugin(TurnBasedCombatPlugin::default())
//!     .build();
//! ```

use async_trait::async_trait;

// Built-in plugins
pub mod combat;
pub mod dungeon;
pub mod inventory;
pub mod loot;
pub mod room_buff;

// Re-exports for convenience
pub use combat::{
    CombatLogEntry,
    CombatResult,
    // Service
    CombatService,
    // Engine
    CombatSystem,
    // Types
    Combatant,
    DamageResult,
    TurnBasedCombatConfig,
    // Plugin
    TurnBasedCombatPlugin,
};

pub use inventory::{
    // Service
    InventoryService,
    // Types
    Item,
    // Plugin
    InventoryPlugin,
};

pub use loot::{
    // Service
    LootService,
    // Types
    DropConfig,
    Rarity,
    // Plugin
    LootPlugin,
};

pub use dungeon::{
    // Service
    DungeonService,
    // System
    DungeonSystem,
    // Types
    Connection,
    ConnectionPattern,
    DungeonConfig,
    DungeonState,
    RoomId,
    // Plugin
    DungeonPlugin,
};

pub use room_buff::{
    // Service
    BuffService,
    // System
    BuffSystem,
    // Types
    ActiveBuff,
    ActiveBuffs,
    BuffConfig,
    BuffDuration,
    BuffEffect,
    RoomBuffDatabase,
    // Plugin
    RoomBuffPlugin,
};

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
    /// Register an entity type
    fn register_entity(&mut self, name: &str, entity: Box<dyn crate::entity::Entity>);

    /// Register a service (Domain Service - pure functions)
    fn register_service(&mut self, service: Box<dyn crate::service::Service>);

    /// Register a system (Application Logic - orchestration)
    fn register_system(&mut self, system: Box<dyn crate::system::System>);

    /// Register a scene
    fn register_scene(&mut self, name: &str, scene: Box<dyn crate::scene::Scene>);

    /// Register an asset
    fn register_asset(&mut self, name: &str, asset: Box<dyn std::any::Any + Send + Sync>);

    /// Get mutable access to the resources registry (internal use)
    fn resources_mut(&mut self) -> &mut crate::resources::Resources;
}

/// Extension trait for PluginBuilder with generic methods
pub trait PluginBuilderExt: PluginBuilder {
    /// Register a resource (read-only global data)
    ///
    /// Resources are type-based and accessible from Systems and Scenes.
    /// Use this to register configuration, asset databases, or lookup tables.
    ///
    /// # Example
    ///
    /// ```ignore
    /// builder.register_resource(DungeonConfig {
    ///     total_floors: 5,
    ///     rooms_per_floor: 3,
    /// });
    /// ```
    fn register_resource<T: crate::resources::Resource>(&mut self, resource: T) {
        self.resources_mut().register(resource);
    }
}

// Blanket implementation
impl<T: ?Sized + PluginBuilder> PluginBuilderExt for T {}
