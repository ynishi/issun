//! Hook trait for custom room buff behavior

use crate::{context::ResourceContext, plugin::ActiveBuff};
use async_trait::async_trait;

/// Trait for custom room buff behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., buff effect application)
/// - Direct resource modification (e.g., modifying combatant stats)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates, achievement tracking)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait RoomBuffHook: Send + Sync {
    /// Called when a buff is applied
    ///
    /// **This is the key feedback loop method.**
    ///
    /// Hook interprets buff application and updates other resources.
    /// For example:
    /// - Combat: Apply stat bonuses to combatants
    /// - Loot: Modify drop rate multipliers
    /// - Economy: Apply income/expense modifiers
    ///
    /// # Arguments
    ///
    /// * `buff` - The buff being applied (with config and remaining turns)
    /// * `resources` - Access to game resources for modification
    async fn on_buff_applied(&self, _buff: &ActiveBuff, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Called when a buff is removed
    ///
    /// Use this to:
    /// - Remove stat bonuses
    /// - Log buff removal
    /// - Update UI
    ///
    /// # Arguments
    ///
    /// * `buff` - The buff being removed
    /// * `resources` - Access to game resources for modification
    async fn on_buff_removed(&self, _buff: &ActiveBuff, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Called when a buff expires naturally (not manually removed)
    ///
    /// Use this for:
    /// - Logging expiration events
    /// - Triggering achievements
    /// - Special effects on expiration
    ///
    /// # Arguments
    ///
    /// * `buff` - The buff that expired
    /// * `resources` - Access to game resources for modification
    async fn on_buff_expired(&self, _buff: &ActiveBuff, _resources: &mut ResourceContext) {
        // Default: do nothing
    }

    /// Called each turn for active buffs
    ///
    /// Use this for:
    /// - Per-turn effects (HP regen, damage over time, etc.)
    /// - Stacking buff effects
    /// - Conditional buff behavior
    ///
    /// # Arguments
    ///
    /// * `buff` - The active buff being ticked
    /// * `resources` - Access to game resources for modification
    async fn on_buff_tick(&self, _buff: &ActiveBuff, _resources: &mut ResourceContext) {
        // Default: do nothing
    }
}

/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultRoomBuffHook;

#[async_trait]
impl RoomBuffHook for DefaultRoomBuffHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use crate::plugin::{ActiveBuff, BuffConfig};

    use super::*;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultRoomBuffHook;
        let buff = ActiveBuff::new(BuffConfig {
            id: "test".to_string(),
            name: "Test Buff".to_string(),
            duration: super::super::types::BuffDuration::Permanent,
            effect: super::super::types::BuffEffect::AttackBonus(10),
        });
        let mut resources = ResourceContext::new();

        // Should not panic
        hook.on_buff_applied(&buff, &mut resources).await;
        hook.on_buff_removed(&buff, &mut resources).await;
        hook.on_buff_expired(&buff, &mut resources).await;
        hook.on_buff_tick(&buff, &mut resources).await;
    }
}
