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
pub mod accounting;
pub mod action;
pub mod combat;
pub mod dungeon;
pub mod economy;
pub mod faction;
pub mod inventory;
pub mod loot;
pub mod metrics;
pub mod policy;
pub mod reputation;
pub mod research;
pub mod room_buff;
pub mod save_load;
pub mod subjective_reality;
pub mod contagion;
pub mod chain_of_command;
pub mod territory;
pub mod time;

// Re-exports for convenience
pub use action::{
    // Config
    ActionConfig,
    // Plugin
    ActionPlugin,
    // Types
    ActionPoints,
};

pub use combat::{
    // Events
    BattleId,
    BattleState,
    // Resources
    CombatConfig,
    CombatEndRequested,
    CombatEndedEvent,
    // Hook
    CombatHook,
    CombatLogEntry,
    // Plugin
    CombatPlugin,
    CombatResult,
    // Service
    CombatService,
    CombatStartRequested,
    CombatStartedEvent,
    CombatState,
    // System
    CombatSystem,
    CombatTurnAdvanceRequested,
    CombatTurnCompletedEvent,
    // Types
    Combatant,
    DamageResult,
    DefaultCombatHook,
};

pub use inventory::{
    DefaultInventoryHook,
    EntityId,
    // Resources
    InventoryConfig,
    InventoryError,
    // Hook
    InventoryHook,
    // Plugin
    InventoryPlugin,
    // Service
    InventoryService,
    InventoryState,
    // System
    InventorySystem,
    // Types
    Item,
    // Events
    ItemAddRequested,
    ItemAddedEvent,
    ItemId,
    ItemRemoveRequested,
    ItemRemovedEvent,
    ItemTransferRequested,
    ItemTransferredEvent,
    ItemUseRequested,
    ItemUsedEvent,
};

pub use loot::{
    DefaultLootHook,
    DropConfig,
    // Resources
    LootConfig,
    LootGenerateRequested,
    LootGeneratedEvent,
    // Hook
    LootHook,
    LootNotGeneratedEvent,
    // Plugin
    LootPlugin,
    // Service
    LootService,
    // Events
    LootSourceId,
    // System
    LootSystem,
    // Types
    Rarity,
    RarityRollRequested,
};

pub use dungeon::{
    // Types
    Connection,
    ConnectionPattern,
    ConnectionUnlockRequested,
    ConnectionUnlockedEvent,
    DefaultDungeonHook,
    // Resources
    DungeonConfig,
    // Hook
    DungeonHook,
    // Plugin
    DungeonPlugin,
    // Service
    DungeonService,
    DungeonState,
    // System
    DungeonSystem,
    FloorAdvanceRequested,
    FloorAdvancedEvent,
    RoomEnteredEvent,
    RoomId,
    // Events
    RoomMoveRequested,
};

pub use room_buff::{
    // Types
    ActiveBuff,
    ActiveBuffs,
    BuffAppliedEvent,
    BuffApplyRequested,
    BuffConfig,
    BuffDuration,
    BuffEffect,
    BuffExpiredEvent,
    // Events
    BuffId,
    BuffRemoveRequested,
    BuffRemovedEvent,
    // Service
    BuffService,
    // System
    BuffSystem,
    BuffTickRequested,
    DefaultRoomBuffHook,
    // Resources
    RoomBuffDatabase,
    // Hook
    RoomBuffHook,
    // Plugin
    RoomBuffPlugin,
};

pub use territory::{
    ControlChanged,
    DefaultTerritoryHook,
    Developed,
    // Resources
    Territories,
    // Types
    Territory,
    // Events
    TerritoryControlChangeRequested,
    TerritoryControlChangedEvent,
    TerritoryDevelopedEvent,
    TerritoryDevelopmentRequested,
    TerritoryEffects,
    TerritoryEffectsUpdatedEvent,
    TerritoryError,
    // Hook
    TerritoryHook,
    TerritoryId,
    // Plugin
    TerritoryPlugin,
    // Service
    TerritoryService,
    TerritoryState,
    // System
    TerritorySystem,
};

pub use faction::{
    DefaultFactionHook,
    // Types
    Faction,
    FactionError,
    // Hook
    FactionHook,
    FactionId,
    // Plugin
    FactionPlugin,
    FactionState,
    // System
    FactionSystem,
    // Resources
    Factions,
    Operation,
    OperationCompletedEvent,
    OperationFailedEvent,
    OperationId,
    // Events
    OperationLaunchRequested,
    OperationLaunchedEvent,
    OperationResolveRequested,
    OperationStatus,
    Outcome,
};

pub use policy::{
    AggregationStrategy,
    DefaultPolicyHook,
    // Resources
    Policies,
    // Types
    Policy,
    // Events
    PolicyActivateRequested,
    PolicyActivatedEvent,
    PolicyConfig,
    PolicyCycleRequested,
    PolicyDeactivateRequested,
    PolicyDeactivatedEvent,
    // Hook
    PolicyHook,
    PolicyId,
    // Plugin
    PolicyPlugin,
    PolicyState,
};

pub use reputation::{
    DefaultReputationHook,
    // Events
    ReputationChangeRequested,
    ReputationChangedEvent,
    // Resources
    ReputationConfig,
    ReputationEntry,
    ReputationError,
    // Hook
    ReputationHook,
    // Plugin
    ReputationPlugin,
    // Service
    ReputationService,
    ReputationSetRequested,
    ReputationState,
    // System
    ReputationSystem,
    ReputationThreshold,
    ReputationThresholdCrossedEvent,
    // Types
    SubjectId,
};

pub use research::{
    DefaultResearchHook,
    ProgressModel,
    ResearchCancelRequested,
    ResearchCancelledEvent,
    ResearchCompleteRequested,
    ResearchCompletedEvent,
    ResearchConfig,
    ResearchError,
    // Hook
    ResearchHook,
    // Types
    ResearchId,
    // Plugin
    ResearchPlugin,
    ResearchProgressRequested,
    ResearchProgressUpdatedEvent,
    ResearchProject,
    // Resources
    ResearchProjects,
    // Events
    ResearchQueueRequested,
    ResearchQueuedEvent,
    ResearchResult,
    ResearchStartRequested,
    ResearchStartedEvent,
    ResearchState,
    ResearchStatus,
    // System
    ResearchSystem,
};

pub use metrics::{
    AggregatedMetric,
    AggregationType,
    ClearMetricsRequested,
    CreateSnapshotRequested,
    // Events
    DefineMetricRequested,
    GenerateReportRequested,
    MetricDefined,
    MetricDefinition,
    // Types
    MetricId,
    MetricRecorded,
    MetricRemoved,
    // Reporting
    MetricReport,
    MetricSnapshot,
    MetricType,
    MetricValue,
    MetricsCleared,
    MetricsConfig,
    // Hook
    MetricsHook,
    // Plugin
    MetricsPlugin,
    // Resource
    MetricsRegistry,
    // System
    MetricsSystem,
    NoOpMetricsHook,
    RecordMetricRequested,
    RemoveMetricRequested,
    ReportGenerated,
    SnapshotCreated,
};

pub use save_load::{
    AutoSaveCompleted,
    AutoSaveRequested,
    DefaultSaveLoadHook,
    DeleteSaveRequested,
    GameLoaded,
    // Events - State
    GameSaved,
    GetSaveMetadataRequested,
    ListSavesRequested,
    LoadGameRequested,
    SaveDeleted,
    SaveFormat,
    // Events - Command
    SaveGameRequested,
    // Config
    SaveLoadConfig,
    SaveLoadFailed,
    // Hook
    SaveLoadHook,
    // Plugin
    SaveLoadPlugin,
    // System
    SaveLoadSystem,
    SaveMetadataRetrieved,
    SavesListed,
};

pub use time::{
    // Events
    ActionConsumedEvent,
    AdvanceTimeRequested,
    // Plugins
    BuiltInTimePlugin,
    DayChanged,
    // Resources
    GameTimer,
    // Config
    TimeConfig,
    TurnBasedTimePlugin,
};

pub use accounting::{
    // Resources
    AccountingConfig,
    // Hook
    AccountingHook,
    // Plugin
    AccountingPlugin,
    // Service
    AccountingService,
    AccountingState,
    // System
    AccountingSystem,
    // Types
    BudgetChannel,
    BudgetLedger,
    BudgetTransferRequested,
    BudgetTransferredEvent,
    DefaultAccountingHook,
    SettlementCompletedEvent,
    // Events
    SettlementRequested,
};

pub use economy::{
    // Types
    ConversionRule,
    // Resources
    ConversionRules,
    Currency,
    CurrencyDefinition,
    CurrencyDefinitions,
    CurrencyDeposited,
    CurrencyExchangeFailed,
    // Events - Command
    CurrencyExchangeRequested,
    // Events - State
    CurrencyExchanged,
    CurrencyId,
    CurrencyWithdrawn,
    EconomyConfig,
    // Service
    EconomyError,
    // Plugin
    EconomyPlugin,
    EconomyResult,
    EconomyService,
    // System
    EconomySystem,
    ExchangeRate,
    ExchangeRates,
    FlowResourceGenerated,
    RateType,
    ResourceAddRequested,
    ResourceAdded,
    ResourceConsumeFailed,
    ResourceConsumeRequested,
    ResourceConsumed,
    ResourceConversionFailed,
    ResourceConversionRequested,
    ResourceConverted,
    ResourceDefinition,
    ResourceDefinitions,
    ResourceId,
    // State
    ResourceInventory,
    ResourceType,
    Wallet,
};

// ChainOfCommandPlugin exports (Phase 1 complete - types, ranks, config only)
pub use chain_of_command::{
    // Types
    Member,
    MemberId,
    Order,
    OrderType,
    OrderOutcome,
    OrderError,
    Priority,
    PromotionError,
    RankId,
    // Resources
    ChainOfCommandConfig,
    RankDefinitions,
    RankDefinition,
    AuthorityLevel,
    // State (Phase 2) ✅
    HierarchyState,
    OrganizationHierarchy,
    // TODO: Phase 3-5 exports
    // Hook (Phase 5)
    // ChainOfCommandHook,
    // DefaultChainOfCommandHook,
    // Plugin (Phase 5)
    // ChainOfCommandPlugin,
    // Service (Phase 3)
    // HierarchyService,
    // System (Phase 4)
    // HierarchySystem,
};

use crate::builder::RuntimeResourceEntry;
use std::any::TypeId;

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

    /// Register a runtime (mutable) resource stored inside `ResourceContext`
    fn register_runtime_resource_boxed(
        &mut self,
        type_id: TypeId,
        resource: Box<dyn RuntimeResourceEntry>,
    );

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

    /// Register a runtime resource (mutable shared state)
    ///
    /// These values live in `ResourceContext` and are mutated exclusively
    /// by systems. Use this for shared runtime state (party status, dungeon
    /// progression, active buffs, etc.).
    fn register_runtime_state<T: 'static + Send + Sync>(&mut self, resource: T) {
        self.register_runtime_resource_boxed(TypeId::of::<T>(), Box::new(resource));
    }
}

// Blanket implementation
impl<T: ?Sized + PluginBuilder> PluginBuilderExt for T {}
