//! Hook trait for custom inventory behavior

use crate::context::ResourceContext;
use async_trait::async_trait;

use super::types::{EntityId, ItemId};

/// Trait for custom inventory behavior
///
/// **Hook vs Event**:
/// - **Hook**: Synchronous, direct call, can modify resources, NO network replication
/// - **Event**: Asynchronous, Pub-Sub, network-friendly, for loose coupling
///
/// **Use Hook for**:
/// - Immediate calculations (e.g., capacity checks, item validation)
/// - Direct resource modification (e.g., applying item effects, HP recovery)
/// - Performance critical paths
/// - Local machine only
///
/// **Use Event for**:
/// - Notifying other systems (e.g., UI updates, achievement tracking)
/// - Network replication (multiplayer)
/// - Audit log / replay
#[async_trait]
pub trait InventoryHook: Send + Sync {
    /// Validate whether an item can be added to an inventory
    ///
    /// Return `Ok(())` to allow, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity whose inventory to check
    /// * `item_id` - Item to add
    /// * `quantity` - Number of items to add
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if addition is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Always allows addition
    ///
    /// # Example Use Cases
    ///
    /// - Capacity checks (inventory full)
    /// - Item restrictions (class-specific items, quest items)
    /// - Duplicate prevention (unique items)
    async fn validate_add_item(
        &self,
        _entity_id: &EntityId,
        _item_id: &ItemId,
        _quantity: u32,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called when an item is added to an inventory
    ///
    /// Use this for logging, statistics tracking, or side effects.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity whose inventory was modified
    /// * `item_id` - Item that was added
    /// * `quantity` - Number of items added
    /// * `resources` - Access to game resources for modification
    ///
    /// # Example Use Cases
    ///
    /// - Log to game log
    /// - Update achievements (collect X items)
    /// - Trigger quest progress
    async fn on_item_added(
        &self,
        _entity_id: &EntityId,
        _item_id: &ItemId,
        _quantity: u32,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when an item is removed from an inventory
    ///
    /// Use this for logging, statistics tracking, or side effects.
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity whose inventory was modified
    /// * `item_id` - Item that was removed
    /// * `quantity` - Number of items removed
    /// * `resources` - Access to game resources for modification
    async fn on_item_removed(
        &self,
        _entity_id: &EntityId,
        _item_id: &ItemId,
        _quantity: u32,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }

    /// Called when an item is used
    ///
    /// **This is the key feedback loop method for item effects.**
    ///
    /// Hook interprets the item usage and updates other resources.
    /// For example:
    /// - Consumables: HP/MP recovery, buff application
    /// - Equipment: Stat bonuses, special abilities
    /// - Key items: Unlock doors, progress quests
    ///
    /// # Arguments
    ///
    /// * `entity_id` - Entity using the item
    /// * `item_id` - Item being used
    /// * `resources` - Access to game resources for modification
    ///
    /// # Returns
    ///
    /// `Ok(())` if usage successful, `Err(reason)` if failed
    ///
    /// # Default
    ///
    /// Returns Ok (no effect)
    async fn on_item_used(
        &self,
        _entity_id: &EntityId,
        _item_id: &ItemId,
        _resources: &mut ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Validate whether an item can be transferred
    ///
    /// Return `Ok(())` to allow, `Err(reason)` to prevent.
    ///
    /// # Arguments
    ///
    /// * `from_entity` - Entity transferring the item
    /// * `to_entity` - Entity receiving the item
    /// * `item_id` - Item to transfer
    /// * `quantity` - Number of items to transfer
    /// * `resources` - Access to game resources (read-only for validation)
    ///
    /// # Returns
    ///
    /// `Ok(())` if transfer is allowed, `Err(reason)` if prevented
    ///
    /// # Default
    ///
    /// Always allows transfer
    ///
    /// # Example Use Cases
    ///
    /// - Prevent trading quest items
    /// - Prevent trading equipped items
    /// - Distance checks (must be nearby)
    async fn validate_transfer(
        &self,
        _from_entity: &EntityId,
        _to_entity: &EntityId,
        _item_id: &ItemId,
        _quantity: u32,
        _resources: &ResourceContext,
    ) -> Result<(), String> {
        Ok(())
    }

    /// Called when an item is transferred between entities
    ///
    /// Use this for logging, statistics tracking, or side effects.
    ///
    /// # Arguments
    ///
    /// * `from_entity` - Entity that transferred the item
    /// * `to_entity` - Entity that received the item
    /// * `item_id` - Item that was transferred
    /// * `quantity` - Number of items transferred
    /// * `resources` - Access to game resources for modification
    async fn on_item_transferred(
        &self,
        _from_entity: &EntityId,
        _to_entity: &EntityId,
        _item_id: &ItemId,
        _quantity: u32,
        _resources: &mut ResourceContext,
    ) {
        // Default: do nothing
    }
}

/// Default hook that does nothing
#[derive(Debug, Clone, Copy, Default)]
pub struct DefaultInventoryHook;

#[async_trait]
impl InventoryHook for DefaultInventoryHook {
    // All methods use default implementations
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_hook_does_nothing() {
        let hook = DefaultInventoryHook;
        let entity_id = "player_1".to_string();
        let item_id = "sword".to_string();
        let resources = ResourceContext::new();

        // Should not panic
        let result = hook
            .validate_add_item(&entity_id, &item_id, 1, &resources)
            .await;
        assert!(result.is_ok());

        let mut resources = ResourceContext::new();
        hook.on_item_added(&entity_id, &item_id, 1, &mut resources)
            .await;
        hook.on_item_removed(&entity_id, &item_id, 1, &mut resources)
            .await;

        let result = hook
            .on_item_used(&entity_id, &item_id, &mut resources)
            .await;
        assert!(result.is_ok());
    }
}
