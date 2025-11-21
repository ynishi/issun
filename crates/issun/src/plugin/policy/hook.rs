//! Hook trait for custom policy behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::*;

/// Trait for custom policy behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., effect modifiers based on game state)
/// - Direct resource modification (e.g., logging to GameContext)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait PolicyHook: Send + Sync {
    /// Called when a policy is activated
    ///
    /// This is called immediately after the policy is marked as active in the registry,
    /// allowing you to modify other resources (e.g., log events, update UI state).
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy being activated
    /// * `previous_policy` - The previously active policy (if any)
    /// * `resources` - Access to game resources for modification
    async fn on_policy_activated(
        &self,
        _policy: &Policy,
        _previous_policy: Option<&Policy>,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when a policy is deactivated
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy being deactivated
    /// * `resources` - Access to game resources for modification
    async fn on_policy_deactivated(&self, _policy: &Policy, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Calculate the effective value of an effect
    ///
    /// This allows game-specific logic to modify effect values based on context.
    /// For example, a "harsh winter" event might reduce the effectiveness of
    /// economic policies.
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy providing the effect
    /// * `effect_name` - The name of the effect (e.g., "income_multiplier")
    /// * `base_value` - The base value from the policy's effects map
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// The effective value (potentially modified by game state)
    ///
    /// # Default
    ///
    /// Returns the base value unchanged
    async fn calculate_effect(
        &self,
        _policy: &Policy,
        _effect_name: &str,
        base_value: f32,
        _resources: &ResourceContext,
    ) -> f32 {
        // Default: no modification
        base_value
    }

    /// Validate whether a policy can be activated
    ///
    /// Return `Ok(())` to allow activation, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `policy` - The policy to validate
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if activation is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Always allows activation
    async fn validate_activation(
        &self,
        _policy: &Policy,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        // Default: always allow
        Ok(())
    }
}

/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultPolicyHook;

#[async_trait]
impl PolicyHook for DefaultPolicyHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ResourceContext;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultPolicyHook;
        let policy = Policy::new("test", "Test", "Test");
        let mut resources = ResourceContext::new();

        // Should not panic
        hook.on_policy_activated(&policy, None, &mut resources)
            .await;
        hook.on_policy_deactivated(&policy, &mut resources).await;
        let value = hook
            .calculate_effect(&policy, "test", 1.0, &resources)
            .await;
        assert_eq!(value, 1.0);
        let result = hook.validate_activation(&policy, &resources).await;
        assert!(result.is_ok());
    }
}
