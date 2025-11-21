//! Hook trait for custom loot behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::events::LootSourceId;
use super::types::Rarity;

/// Trait for custom loot behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., drop chance modifiers based on player luck)
/// - Direct resource modification (e.g., adding items to inventory)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates, achievement tracking)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait LootHook: Send + Sync {
    /// Modify drop chance based on game state
    ///
    /// Allows game-specific bonuses/penalties.
    /// Examples: luck stat, difficulty modifiers, buff effects
    ///
    /// # Arguments
    ///
    /// * `source_id` - Source of the loot (enemy, chest, etc.)
    /// * `base_drop_rate` - Base drop chance (0.0 - 1.0)
    /// * `resources` - Access to game resources (read-only for calculations)
    ///
    /// # Returns
    ///
    /// Final drop rate (0.0 - 1.0)
    ///
    /// # Default
    ///
    /// Returns base_drop_rate unchanged
    async fn modify_drop_chance(
        &self,
        _source_id: &LootSourceId,
        base_drop_rate: f32,
        _resources: &ResourceContext,
    ) -> f32 {
        base_drop_rate
    }

    /// Generate loot items for a given rarity
    ///
    /// **This is the main hook for game-specific loot generation.**
    ///
    /// The hook should:
    /// 1. Determine what items to drop based on source and rarity
    /// 2. Return a list of item IDs
    ///
    /// # Arguments
    ///
    /// * `source_id` - Source of the loot (enemy, chest, etc.)
    /// * `rarity` - Selected rarity tier
    /// * `resources` - Access to game resources (read-only for loot table lookup)
    ///
    /// # Returns
    ///
    /// Vector of item IDs to drop
    ///
    /// # Default
    ///
    /// Returns empty list (no items)
    ///
    /// # Example
    ///
    /// ```ignore
    /// async fn generate_loot(
    ///     &self,
    ///     source_id: &str,
    ///     rarity: Rarity,
    ///     resources: &ResourceContext,
    /// ) -> Vec<String> {
    ///     // Look up loot table based on source
    ///     match source_id {
    ///         "goblin" => match rarity {
    ///             Rarity::Common => vec!["rusty_sword".to_string()],
    ///             Rarity::Rare => vec!["magic_dagger".to_string()],
    ///             _ => vec![],
    ///         },
    ///         _ => vec![],
    ///     }
    /// }
    /// ```
    async fn generate_loot(
        &self,
        _source_id: &LootSourceId,
        _rarity: Rarity,
        _resources: &ResourceContext,
    ) -> Vec<String> {
        Vec::new()
    }

    /// Called when loot is generated
    ///
    /// Use this for:
    /// - Adding items to player inventory
    /// - Logging loot drops
    /// - Tracking statistics (total drops, rare drops, etc.)
    ///
    /// # Arguments
    ///
    /// * `source_id` - Source that generated the loot
    /// * `items` - List of item IDs that were generated
    /// * `rarity` - Rarity tier that was selected
    /// * `resources` - Access to game resources for modification
    async fn on_loot_generated(
        &self,
        _source_id: &LootSourceId,
        _items: &[String],
        _rarity: Rarity,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}

/// Default hook that does nothing (no loot generation)
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultLootHook;

#[async_trait]
impl LootHook for DefaultLootHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultLootHook;
        let source_id = "enemy_1".to_string();
        let resources = ResourceContext::new();

        // Should not panic
        let drop_rate = hook.modify_drop_chance(&source_id, 0.5, &resources).await;
        assert_eq!(drop_rate, 0.5);

        let items = hook
            .generate_loot(&source_id, Rarity::Common, &resources)
            .await;
        assert!(items.is_empty());

        let mut resources = ResourceContext::new();
        hook.on_loot_generated(&source_id, &[], Rarity::Common, &mut resources)
            .await;
    }
}
