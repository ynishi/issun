//! Faction plugin implementation

use super::factions::Factions;
use super::hook::{DefaultFactionHook, FactionHook};
use super::state::FactionState;
use super::system::FactionSystem;
use crate::plugin::{Plugin, PluginBuilder, PluginBuilderExt};
use async_trait::async_trait;
use std::sync::Arc;

/// Built-in faction management plugin
///
/// This plugin provides faction/organization/group management for games.
/// It registers Factions, FactionState resources and FactionSystem that handles:
/// - Processing operation launch requests
/// - Processing operation resolution requests
/// - Custom hooks for game-specific behavior
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Log operation events to game log
/// - Calculate operation costs with faction bonuses
/// - Update other resources when operations complete (territories, budgets, etc.)
/// - Handle faction relationships
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::faction::{FactionPlugin, FactionHook};
/// use async_trait::async_trait;
///
/// // Custom hook for logging
/// struct GameLogHook;
///
/// #[async_trait]
/// impl FactionHook for GameLogHook {
///     async fn on_operation_completed(
///         &self,
///         faction: &Faction,
///         operation: &Operation,
///         outcome: &Outcome,
///         resources: &mut ResourceContext,
///     ) {
///         // Log to game log
///         println!("Faction {} completed {}: {}",
///             faction.name,
///             operation.name,
///             if outcome.success { "SUCCESS" } else { "FAILED" }
///         );
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         FactionPlugin::new()
///             .with_hook(GameLogHook)
///     )
///     .build()
///     .await?;
/// ```
pub struct FactionPlugin {
    hook: Arc<dyn FactionHook>,
    factions: Factions,
}

impl FactionPlugin {
    /// Create a new faction plugin
    ///
    /// Uses the default hook (no-op) by default.
    /// Use `with_hook()` to add custom behavior.
    pub fn new() -> Self {
        Self {
            hook: Arc::new(DefaultFactionHook),
            factions: Factions::new(),
        }
    }

    /// Add a custom hook for faction behavior
    ///
    /// The hook will be called when:
    /// - Operations are launched (`on_operation_launched`)
    /// - Operation cost is calculated (`calculate_operation_cost`)
    /// - Operations are completed (`on_operation_completed`)
    /// - Operations fail (`on_operation_failed`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of FactionHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::faction::{FactionPlugin, FactionHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl FactionHook for MyHook {
    ///     async fn on_operation_completed(
    ///         &self,
    ///         faction: &Faction,
    ///         operation: &Operation,
    ///         outcome: &Outcome,
    ///         resources: &mut ResourceContext,
    ///     ) {
    ///         // Custom logic
    ///     }
    /// }
    ///
    /// let plugin = FactionPlugin::new()
    ///     .with_hook(MyHook);
    /// ```
    pub fn with_hook(mut self, hook: impl FactionHook + 'static) -> Self {
        self.hook = Arc::new(hook);
        self
    }

    /// Add faction definitions
    ///
    /// # Arguments
    ///
    /// * `factions` - Collection of faction definitions
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::faction::{FactionPlugin, Factions, Faction};
    ///
    /// let mut factions = Factions::new();
    /// factions.add(Faction::new("crimson", "Crimson Syndicate"));
    ///
    /// let plugin = FactionPlugin::new().with_factions(factions);
    /// ```
    pub fn with_factions(mut self, factions: Factions) -> Self {
        self.factions = factions;
        self
    }
}

impl Default for FactionPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Plugin for FactionPlugin {
    fn name(&self) -> &'static str {
        "issun:faction"
    }

    fn build(&self, builder: &mut dyn PluginBuilder) {
        // Register faction definitions (ReadOnly)
        builder.register_resource(self.factions.clone());

        // Register faction state (Mutable)
        builder.register_runtime_state(FactionState::new());

        // Register system with hook
        builder.register_system(Box::new(FactionSystem::new(Arc::clone(&self.hook))));
    }

    async fn initialize(&mut self) {
        // No initialization needed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let plugin = FactionPlugin::default();
        assert_eq!(plugin.name(), "issun:faction");
    }

    #[test]
    fn test_plugin_default() {
        let plugin = FactionPlugin::default();
        // Should not panic
        assert_eq!(plugin.name(), "issun:faction");
    }

    #[test]
    fn test_plugin_with_hook() {
        let plugin = FactionPlugin::new().with_hook(DefaultFactionHook);
        assert_eq!(plugin.name(), "issun:faction");
    }

    #[test]
    fn test_plugin_with_factions() {
        use super::super::types::Faction;

        let mut factions = Factions::new();
        factions.add(Faction::new("crimson", "Crimson Syndicate"));

        let plugin = FactionPlugin::new().with_factions(factions);
        assert_eq!(plugin.name(), "issun:faction");
    }
}
