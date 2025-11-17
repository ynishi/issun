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
    // Plugin
    InventoryPlugin,
    // Service
    InventoryService,
    // Types
    Item,
};

pub use loot::{
    // Types
    DropConfig,
    // Plugin
    LootPlugin,
    // Service
    LootService,
    Rarity,
};

pub use dungeon::{
    // Types
    Connection,
    ConnectionPattern,
    DungeonConfig,
    // Plugin
    DungeonPlugin,
    // Service
    DungeonService,
    DungeonState,
    // System
    DungeonSystem,
    RoomId,
};

pub use room_buff::{
    // Types
    ActiveBuff,
    ActiveBuffs,
    BuffConfig,
    BuffDuration,
    BuffEffect,
    // Service
    BuffService,
    // System
    BuffSystem,
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

    // Note: Scene registration removed in favor of SceneDirector-based architecture
    // Scenes are now managed directly by SceneDirector, not through plugins

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
