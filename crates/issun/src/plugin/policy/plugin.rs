//! Policy plugin implementation

use super::config::PolicyConfig;
use super::hook::{DefaultPolicyHook, PolicyHook};
use super::policies::Policies;
use super::state::PolicyState;
use super::system::PolicySystem;
use crate::Plugin;
use std::sync::Arc;

/// Built-in policy management plugin
///
/// This plugin provides policy/card/buff management for games.
/// It registers Policies, PolicyConfig, PolicyState resources and PolicySystem that handles:
/// - Processing policy activation requests
/// - Processing policy deactivation requests
/// - Processing policy cycling requests
/// - Custom hooks for game-specific behavior
///
/// # Hook Customization
///
/// You can provide a custom hook to add game-specific behavior:
/// - Log policy changes to game log
/// - Validate policy activation based on game state
/// - Modify effect values based on context (e.g., events, seasons)
/// - Update UI when policies change
///
/// # Example
///
/// ```ignore
/// use issun::builder::GameBuilder;
/// use issun::plugin::policy::{PolicyPlugin, PolicyHook};
/// use async_trait::async_trait;
///
/// // Custom hook for logging
/// struct GameLogHook;
///
/// #[async_trait]
/// impl PolicyHook for GameLogHook {
///     async fn on_policy_activated(
///         &self,
///         policy: &Policy,
///         previous: Option<&Policy>,
///         resources: &mut ResourceContext,
///     ) {
///         // Log to game log
///         println!("Policy activated: {}", policy.name);
///     }
/// }
///
/// let game = GameBuilder::new()
///     .with_plugin(
///         PolicyPlugin::new()
///             .with_hook(GameLogHook)
///     )
///     .build()
///     .await?;
/// ```
#[derive(Plugin)]
#[plugin(name = "issun:policy")]
pub struct PolicyPlugin {
    #[plugin(skip)]
    hook: Arc<dyn PolicyHook>,
    #[plugin(resource)]
    config: PolicyConfig,
    #[plugin(resource)]
    policies: Policies,
    #[plugin(runtime_state)]
    #[allow(dead_code)]
    state: PolicyState,
    #[plugin(system)]
    system: PolicySystem,
}

impl PolicyPlugin {
    /// Create a new policy plugin
    ///
    /// Uses the default hook (no-op) and default config by default.
    /// Use `with_hook()` to add custom behavior and `with_config()` to customize configuration.
    pub fn new() -> Self {
        let hook = Arc::new(DefaultPolicyHook);
        Self {
            hook: hook.clone(),
            config: PolicyConfig::default(),
            policies: Policies::new(),
            state: PolicyState::new(),
            system: PolicySystem::new(hook),
        }
    }

    /// Add a custom hook for policy behavior
    ///
    /// The hook will be called when:
    /// - Policies are activated (`on_policy_activated`)
    /// - Policies are deactivated (`on_policy_deactivated`)
    /// - Effect values are calculated (`calculate_effect`)
    /// - Policy activation is validated (`validate_activation`)
    ///
    /// # Arguments
    ///
    /// * `hook` - Implementation of PolicyHook trait
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::policy::{PolicyPlugin, PolicyHook};
    ///
    /// struct MyHook;
    ///
    /// #[async_trait]
    /// impl PolicyHook for MyHook {
    ///     async fn on_policy_activated(
    ///         &self,
    ///         policy: &Policy,
    ///         previous: Option<&Policy>,
    ///         resources: &mut ResourceContext,
    ///     ) {
    ///         // Custom behavior...
    ///     }
    /// }
    ///
    /// let plugin = PolicyPlugin::new().with_hook(MyHook);
    /// ```
    pub fn with_hook<H: PolicyHook + 'static>(mut self, hook: H) -> Self {
        let hook = Arc::new(hook);
        self.hook = hook.clone();
        self.system = PolicySystem::new(hook);
        self
    }

    /// Set custom policy configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Policy configuration (activation mode, aggregation strategies, etc.)
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::policy::{PolicyPlugin, PolicyConfig, AggregationStrategy};
    /// use std::collections::HashMap;
    ///
    /// let mut config = PolicyConfig::default();
    /// config.allow_multiple_active = true;
    /// config.max_active_policies = Some(3);
    /// config.aggregation_strategies.insert(
    ///     "income_multiplier".into(),
    ///     AggregationStrategy::Multiply
    /// );
    ///
    /// let plugin = PolicyPlugin::new().with_config(config);
    /// ```
    pub fn with_config(mut self, config: PolicyConfig) -> Self {
        self.config = config;
        self
    }

    /// Add policy definitions
    ///
    /// # Arguments
    ///
    /// * `policies` - Collection of policy definitions
    ///
    /// # Example
    ///
    /// ```ignore
    /// use issun::plugin::policy::{PolicyPlugin, Policies, Policy};
    ///
    /// let mut policies = Policies::new();
    /// policies.add(Policy::new("policy1", "Policy 1", "Description"));
    ///
    /// let plugin = PolicyPlugin::new().with_policies(policies);
    /// ```
    pub fn with_policies(mut self, policies: Policies) -> Self {
        self.policies = policies;
        self
    }
}

impl Default for PolicyPlugin {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        let _plugin = PolicyPlugin::new();
        // Plugin derive macro automatically implements name()
    }

    #[test]
    fn test_plugin_with_custom_hook() {
        struct CustomHook;

        #[async_trait::async_trait]
        impl PolicyHook for CustomHook {}

        let _plugin = PolicyPlugin::new().with_hook(CustomHook);
        // Plugin derive macro automatically implements name()
    }

    #[test]
    fn test_plugin_with_custom_config() {
        let config = PolicyConfig {
            allow_multiple_active: true,
            ..Default::default()
        };

        let _plugin = PolicyPlugin::new().with_config(config);
        // Plugin derive macro automatically implements name()
    }

    #[test]
    fn test_plugin_with_policies() {
        use super::super::types::Policy;

        let mut policies = Policies::new();
        policies.add(Policy::new("test", "Test", "Test policy"));

        let _plugin = PolicyPlugin::new().with_policies(policies);
        // Plugin derive macro automatically implements name()
    }
}
