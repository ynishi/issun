//! Territory plugin implementation

use super::hook::{DefaultTerritoryHook, TerritoryHook};
use super::state::TerritoryState;
use super::system::TerritorySystem;
use super::territories::Territories;
use crate::Plugin;
use std::sync::Arc;

/// Built-in territory management plugin
///
/// This plugin provides territory management for strategy games.
/// It registers Territories (definitions) and TerritoryState (runtime state) resources,
/// and TerritorySystem that handles:
/// - Processing territory control changes
/// - Processing territory development
/// - Custom hooks for game-specific behavior
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Log territory changes to game log
/// - Calculate development costs with policy bonuses
/// - Update other resources when territory changes
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::territory::{TerritoryPlugin, TerritoryHook};
/// use async_trait::async_trait;
///
/// // Custom hook for logging
/// struct GameLogHook;
///
/// #[async_trait]
/// impl TerritoryHook for GameLogHook {
///     async fn on_control_changed(
///         &self,
///         territory: &Territory,
///         change: &ControlChanged,
///         resources: &mut ResourceContext,
///     ) {
///         // Log to game log
///         println!("Territory {} control: {:.0}% → {:.0}%",
///             territory.name,
///             change.old_control * 100.0,
///             change.new_control * 100.0
///         );
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         TerritoryPlugin::new()
///             .with_hook(GameLogHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:territory")]
pub struct TerritoryPlugin {
    #[plugin(skip)]
    hook: Arc<dyn TerritoryHook>,
    #[plugin(resource)]
    #[allow(dead_code)]
    territories: Territories,
    #[plugin(runtime_state)]
    #[allow(dead_code)]
    state: TerritoryState,
    #[plugin(system)]
    system: TerritorySystem,
}

impl TerritoryPlugin {
    /// Create a new territory plugin
    ///
    /// Uses the default hook (no-op) by default.
    /// Use `with_hook()` to add custom behavior.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultTerritoryHook);
        Self {
            hook: hook.clone(),
            territories: Territories::new(),
            state: TerritoryState::new(),
            system: TerritorySystem::new(hook),
        }
    }

    /// Add a custom hook for territory behavior
    ///
    /// The hook will be called when:
    /// - Territory control changes (`on_control_changed`)
    /// - Development is requested (`calculate_development_cost`)
    /// - Territory is developed (`on_developed`)
    /// - Effects are calculated (`calculate_effects`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of TerritoryHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::territory::{TerritoryPlugin, TerritoryHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl TerritoryHook for MyHook {
    ///     async fn on_control_changed(&self, territory: &Territory, change: &ControlChanged, resources: &mut ResourceContext) {
    ///         // Custom logic
    ///     }
    /// }
    ///
    /// let plugin = TerritoryPlugin::new()
    ///     .with_hook(MyHook);
    /// ```
    pub fn with_hook(mut self, hook: impl TerritoryHook + 'static) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = TerritorySystem::new(hook);
        self
    }
}

impl Default for TerritoryPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_name() {
        let _plugin = TerritoryPlugin::default();
        // Plugin derive macro automatically implements name()
    }

    #[test]
    fn test_plugin_default() {
        let _plugin = TerritoryPlugin::default();
        // Should not panic
    }

    #[test]
    fn test_plugin_with_hook() {
        let _plugin = TerritoryPlugin::new().with_hook(DefaultTerritoryHook);
        // Plugin derive macro automatically implements name()
    }
}
