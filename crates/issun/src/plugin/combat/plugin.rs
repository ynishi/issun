//! Turn-based combat plugin implementation

use super::config::CombatConfig;
use super::hook::{CombatHook, DefaultCombatHook};
use super::service::CombatService;
use super::state::CombatState;
use super::system::CombatSystem;
use crate::Plugin;
use std::sync::Arc;

/// Turn-based combat plugin
///
/// This plugin provides turn-based combat functionality with:
/// - Turn management and combat state tracking
/// - Customizable combat logic via hooks
/// - Event-driven architecture for loose coupling
/// - Damage calculation service
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Implement turn-based combat logic (who attacks whom)
/// - Calculate damage modifiers (critical hits, elemental weakness, buffs)
/// - Award XP, loot, and achievements on combat end
/// - Log combat events to game log
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::combat::{CombatPlugin, CombatHook};
/// use async_trait::async_trait;
///
/// // Custom hook for turn processing
/// struct MyCompat Hook;
///
/// #[async_trait]
/// impl CombatHook for MyCombatHook {
///     async fn process_turn(
///         &self,
///         battle_id: &str,
///         turn: u32,
///         resources: &mut ResourceContext,
///     ) -> Vec<String> {
///         // Implement combat logic here
///         vec!["Player attacks enemy!".to_string()]
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         CombatPlugin::new()
///             .with_hook(MyCombatHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:combat")]
pub struct CombatPlugin {
    hook: Arc<dyn CombatHook>,

    #[resource]
    config: CombatConfig,

    #[state]
    state: CombatState,

    #[service]
    service: CombatService,

    #[system]
    system: CombatSystem,
}

impl CombatPlugin {
    /// Create a new combat plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultCombatHook);
        Self {
            hook: hook.clone(),
            config: CombatConfig::default(),
            state: CombatState::new(),
            service: CombatService::new(),
            system: CombatSystem::new(hook),
        }
    }

    /// Add a custom hook for combat behavior
    ///
    /// The hook will be called when:
    /// - Before each turn (`before_turn`)
    /// - During turn processing (`process_turn`) - **main combat logic**
    /// - After each turn (`after_turn`)
    /// - When combat ends (`on_combat_ended`)
    /// - For damage calculation (`calculate_damage_modifier`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of CombatHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::combat::{CombatPlugin, CombatHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl CombatHook for MyHook {
    ///     async fn process_turn(
    ///         &self,
    ///         battle_id: &str,
    ///         turn: u32,
    ///         resources: &mut ResourceContext,
    ///     ) -> Vec<String> {
    ///         // Custom combat logic...
    ///         Vec::new()
    ///     }
    /// }
    ///
    /// let plugin = CombatPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: CombatHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        // Re-create system with new hook
        self.system = CombatSystem::new(hook);
        self
    }

    /// Set custom combat configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Combat configuration (log settings, score, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::combat::{CombatPlugin, CombatConfig};
    ///
    /// let config = CombatConfig {
    ///     enable_log: true,
    ///     max_log_entries: 200,
    ///     score_per_enemy: 50,
    /// };
    ///
    /// let plugin = CombatPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: CombatConfig) -> Self {
        self.config = config;
        self
    }
}

impl Default for CombatPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Plugin;

    #[test]
    fn test_plugin_creation() {
        let plugin = CombatPlugin::new();
        assert_eq!(plugin.name(), "issun:combat");
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl CombatHook for CustomHook {}

        let plugin = CombatPlugin::new().with_hook(CustomHook);
        assert_eq!(plugin.name(), "issun:combat");
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = CombatConfig {
            enabled: true,
            default_max_hp: 100,
            difficulty_multiplier: 1.0,
            enable_log: false,
            max_log_entries: 50,
            score_per_enemy: 20,
        };

        let plugin = CombatPlugin::new().with_config(config);
        assert_eq!(plugin.name(), "issun:combat");
    }
}
