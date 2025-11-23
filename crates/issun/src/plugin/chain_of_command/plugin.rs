//! ChainOfCommandPlugin implementation

use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;

use super::config::ChainOfCommandConfig;
use super::hook::{ChainOfCommandHook, DefaultChainOfCommandHook};
use super::rank_definitions::RankDefinitions;
use super::state::HierarchyState;
use super::types::FactionId;

/// Plugin for organizational hierarchy and command structure management
///
/// # Example
///
/// ```ignore
/// use issun::prelude::*;
/// use issun::plugin::chain_of_command::{
///     ChainOfCommandPlugin, ChainOfCommandConfig, RankDefinitions, RankDefinition, AuthorityLevel
/// };
///
/// let mut ranks = RankDefinitions::new();
/// ranks.add(RankDefinition::new("private", "Private", 0, AuthorityLevel::Private));
/// ranks.add(RankDefinition::new("sergeant", "Sergeant", 1, AuthorityLevel::SquadLeader));
/// ranks.add(RankDefinition::new("captain", "Captain", 2, AuthorityLevel::Captain));
///
/// let game = GameBuilder::new()
///     .add_plugin(
///         ChainOfCommandPlugin::new()
///             .with_ranks(ranks)
///             .with_config(ChainOfCommandConfig {
///                 min_tenure_for_promotion: 10,
///                 loyalty_decay_rate: 0.01,
///                 base_order_compliance_rate: 0.85,
///                 min_loyalty_for_promotion: 0.6,
///             })
///             .register_faction("faction_a")
///             .register_faction("faction_b")
///     )
///     .build()
///     .await?;
/// ```
pub struct ChainOfCommandPlugin<H: ChainOfCommandHook = DefaultChainOfCommandHook> {
    config: ChainOfCommandConfig,
    ranks: RankDefinitions,
    registered_factions: Vec<FactionId>,
    #[allow(dead_code)]
    hook: H,
}

impl ChainOfCommandPlugin<DefaultChainOfCommandHook> {
    /// Create a new chain of command plugin with default hook
    pub fn new() -> Self {
        Self {
            config: ChainOfCommandConfig::default(),
            ranks: RankDefinitions::new(),
            registered_factions: Vec::new(),
            hook: DefaultChainOfCommandHook,
        }
    }
}

impl Default for ChainOfCommandPlugin<DefaultChainOfCommandHook> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: ChainOfCommandHook> ChainOfCommandPlugin<H> {
    /// Create with a custom hook
    pub fn with_hook<NewH: ChainOfCommandHook>(self, hook: NewH) -> ChainOfCommandPlugin<NewH> {
        ChainOfCommandPlugin {
            config: self.config,
            ranks: self.ranks,
            registered_factions: self.registered_factions,
            hook,
        }
    }

    /// Set configuration
    pub fn with_config(mut self, config: ChainOfCommandConfig) -> Self {
        self.config = config;
        self
    }

    /// Set rank definitions
    pub fn with_ranks(mut self, ranks: RankDefinitions) -> Self {
        self.ranks = ranks;
        self
    }

    /// Register a faction (creates empty hierarchy)
    pub fn register_faction(mut self, faction_id: impl Into<String>) -> Self {
        self.registered_factions.push(faction_id.into());
        self
    }

    /// Register multiple factions at once
    pub fn register_factions(mut self, faction_ids: Vec<impl Into<String>>) -> Self {
        for faction_id in faction_ids {
            self.registered_factions.push(faction_id.into());
        }
        self
    }
}

#[async_trait]
impl<H: ChainOfCommandHook + Send + Sync + 'static> Plugin for ChainOfCommandPlugin<H> {
    fn name(&self) -> &'static str {
        "chain_of_command_plugin"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register config (ReadOnly)
        builder.register_resource(self.config.clone());

        // Register rank definitions (ReadOnly)
        builder.register_resource(self.ranks.clone());

        // Register state (Mutable)
        let mut state = HierarchyState::new();
        for faction_id in &self.registered_factions {
            state.register_faction(faction_id);
        }
        builder.register_runtime_state(state);

        // Note: System registration would happen here, but issun's plugin system
        // currently doesn't have a direct system registration API.
        // Systems need to be created and called manually in the game loop.
        // HierarchySystem<H>::new(self.hook.clone()) would be created in game code.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::chain_of_command::{AuthorityLevel, RankDefinition};

    #[test]
    fn test_plugin_creation() {
        let plugin = ChainOfCommandPlugin::new();
        assert_eq!(plugin.name(), "chain_of_command_plugin");
    }

    #[test]
    fn test_plugin_with_config() {
        let config = ChainOfCommandConfig {
            min_tenure_for_promotion: 10,
            loyalty_decay_rate: 0.01,
            base_order_compliance_rate: 0.85,
            min_loyalty_for_promotion: 0.6,
        };

        let plugin = ChainOfCommandPlugin::new().with_config(config.clone());
        assert_eq!(plugin.config.min_tenure_for_promotion, 10);
        assert_eq!(plugin.config.loyalty_decay_rate, 0.01);
    }

    #[test]
    fn test_plugin_with_ranks() {
        let mut ranks = RankDefinitions::new();
        ranks.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));
        ranks.add(RankDefinition::new(
            "sergeant",
            "Sergeant",
            1,
            AuthorityLevel::SquadLeader,
        ));

        let plugin = ChainOfCommandPlugin::new().with_ranks(ranks.clone());
        assert!(!plugin.ranks.is_empty());
    }

    #[test]
    fn test_plugin_register_faction() {
        let plugin = ChainOfCommandPlugin::new()
            .register_faction("faction_a")
            .register_faction("faction_b");

        assert_eq!(plugin.registered_factions.len(), 2);
        assert_eq!(plugin.registered_factions[0], "faction_a");
        assert_eq!(plugin.registered_factions[1], "faction_b");
    }

    #[test]
    fn test_plugin_register_factions() {
        let plugin = ChainOfCommandPlugin::new().register_factions(vec![
            "faction_a",
            "faction_b",
            "faction_c",
        ]);

        assert_eq!(plugin.registered_factions.len(), 3);
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        #[derive(Clone, Copy)]
        struct CustomHook;

        #[async_trait]
        impl ChainOfCommandHook for CustomHook {}

        let plugin = ChainOfCommandPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "chain_of_command_plugin");
    }

    #[test]
    fn test_plugin_builder_chain() {
        let mut ranks = RankDefinitions::new();
        ranks.add(RankDefinition::new(
            "private",
            "Private",
            0,
            AuthorityLevel::Private,
        ));

        let plugin = ChainOfCommandPlugin::new()
            .with_config(ChainOfCommandConfig::default().with_min_tenure(15))
            .with_ranks(ranks)
            .register_faction("faction_a")
            .register_faction("faction_b");

        assert_eq!(plugin.config.min_tenure_for_promotion, 15);
        assert!(!plugin.ranks.is_empty());
        assert_eq!(plugin.registered_factions.len(), 2);
    }

    #[test]
    fn test_default_plugin() {
        let plugin = ChainOfCommandPlugin::default();
        assert_eq!(plugin.name(), "chain_of_command_plugin");
        assert_eq!(plugin.registered_factions.len(), 0);
    }
}
