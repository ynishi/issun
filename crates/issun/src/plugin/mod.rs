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
pub mod inventory;
// pub mod loot;       // TODO: Implement
// pub mod dungeon;    // TODO: Implement

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
}
