//! Hook trait for custom action behavior

use crate::context::ResourceContext;
use crate::plugin::action::ActionConsumed;
use async_trait::async_trait;

/// Trait for custom behavior when actions are consumed
///
/// This trait enables direct resource modification without going through events,
/// which is justified because action consumption always triggers game-specific
/// side effects (logging, statistics, etc.).
///
/// # Example
///
/// ```ignore
/// use issun::plugin::action::{ActionHook, ActionConsumed};
/// use issun::context::ResourceContext;
/// use async_trait::async_trait;
///
/// struct GameLogHook;
///
/// #[async_trait]
/// impl ActionHook for GameLogHook {
///     async fn on_action_consumed(
///         &self,
///         consumed: &ActionConsumed,
///         resources: &mut ResourceContext,
///     ) {
///         if let Some(mut log) = resources.get_mut::<GameLog>().await {
///             log.record(format!(
///                 "Action: {} ({} remaining)",
///                 consumed.context,
///                 consumed.remaining
///             ));
///         }
///     }
/// }
/// ```
#[async_trait]
pub trait ActionHook: Send + Sync {
    /// Called after action is successfully consumed
    ///
    /// This method can modify other resources via ResourceContext,
    /// enabling game-specific side effects like logging, statistics tracking, etc.
    ///
    /// # Arguments
    ///
    /// * `consumed` - Details about the consumed action
    /// * `resources` - Access to game resources for modification
    async fn on_action_consumed(&self, consumed: &ActionConsumed, resources: &mut ResourceContext);

    /// Called when actions are depleted
    ///
    /// Return `true` to auto-advance time, `false` to prevent auto-advance.
    /// This allows games to implement manual turn-end confirmation or
    /// other custom behavior when all actions are consumed.
    ///
    /// # Arguments
    ///
    /// * `resources` - Access to game resources for checking state
    ///
    /// # Returns
    ///
    /// `true` to allow auto-advance, `false` to prevent
    ///
    /// # Default
    ///
    /// Returns `true` (allow auto-advance)
    async fn on_actions_depleted(&self, _resources: &mut ResourceContext) -> bool {
        true // Default: allow auto-advance
    }

    /// Called when actions are reset
    ///
    /// This is typically called on day/turn boundaries when action points
    /// are refreshed to maximum.
    ///
    /// # Arguments
    ///
    /// * `new_count` - New action count after reset
    /// * `resources` - Access to game resources for modification
    ///
    /// # Default
    ///
    /// Does nothing
    async fn on_actions_reset(&self, _new_count: u32, _resources: &mut ResourceContext) {
        // Default: do nothing
    }
}

/// No-op default hook
///
/// This is the default hook used when no custom hook is provided.
/// It implements all hook methods with no-op behavior.
#[derive(Default, Debug, Clone, Copy)]
pub struct DefaultActionHook;

#[async_trait]
impl ActionHook for DefaultActionHook {
    async fn on_action_consumed(&self, _: &ActionConsumed, _: &mut ResourceContext) {
        // No-op
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook() {
        let hook = DefaultActionHook;
        let mut resources = ResourceContext::new();

        let consumed = ActionConsumed {
            context: "test".into(),
            remaining: 2,
            depleted: false,
        };

        // Should not panic
        hook.on_action_consumed(&consumed, &mut resources).await;
        assert!(hook.on_actions_depleted(&mut resources).await);
        hook.on_actions_reset(3, &mut resources).await;
    }
}
