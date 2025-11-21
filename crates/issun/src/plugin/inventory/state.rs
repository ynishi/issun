//! Inventory runtime state (Mutable)

use super::types::{EntityId, InventoryError, ItemId};
use crate::state::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Inventory runtime state (Mutable)
///
/// Contains inventory data that changes during gameplay.
/// This is a save/load target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryState {
    /// Inventories mapped by entity ID
    /// Inner HashMap maps ItemId -> quantity
    inventories: HashMap<EntityId, HashMap<ItemId, u32>>,
}

impl State for InventoryState {}

impl InventoryState {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            inventories: HashMap::new(),
        }
    }

    // ========================================
    // Inventory Management
    // ========================================

    /// Get or create inventory for an entity
    fn get_or_create_inventory(&mut self, entity_id: &EntityId) -> &mut HashMap<ItemId, u32> {
        self.inventories.entry(entity_id.clone()).or_default()
    }

    /// Get inventory for an entity (read-only)
    pub fn get_inventory(&self, entity_id: &EntityId) -> Option<&HashMap<ItemId, u32>> {
        self.inventories.get(entity_id)
    }

    /// Check if entity has an inventory
    pub fn has_inventory(&self, entity_id: &EntityId) -> bool {
        self.inventories.contains_key(entity_id)
    }

    /// Get total number of item slots used by an entity
    pub fn get_slot_count(&self, entity_id: &EntityId) -> usize {
        self.get_inventory(entity_id)
            .map(|inv| inv.len())
            .unwrap_or(0)
    }

    /// Get total number of items (sum of all quantities) for an entity
    pub fn get_total_items(&self, entity_id: &EntityId) -> u32 {
        self.get_inventory(entity_id)
            .map(|inv| inv.values().sum())
            .unwrap_or(0)
    }

    // ========================================
    // Item Operations
    // ========================================

    /// Add item to an entity's inventory
    pub fn add_item(
        &mut self,
        entity_id: &EntityId,
        item_id: &ItemId,
        quantity: u32,
    ) -> Result<(), InventoryError> {
        if quantity == 0 {
            return Ok(());
        }

        let inventory = self.get_or_create_inventory(entity_id);
        *inventory.entry(item_id.clone()).or_insert(0) += quantity;
        Ok(())
    }

    /// Remove item from an entity's inventory
    pub fn remove_item(
        &mut self,
        entity_id: &EntityId,
        item_id: &ItemId,
        quantity: u32,
    ) -> Result<(), InventoryError> {
        if quantity == 0 {
            return Ok(());
        }

        let inventory = self
            .inventories
            .get_mut(entity_id)
            .ok_or(InventoryError::EntityNotFound)?;

        let current = inventory.get(item_id).ok_or(InventoryError::ItemNotFound)?;

        if *current < quantity {
            return Err(InventoryError::ItemNotFound);
        }

        if *current == quantity {
            inventory.remove(item_id);
        } else {
            *inventory.get_mut(item_id).unwrap() -= quantity;
        }

        Ok(())
    }

    /// Get quantity of a specific item in an entity's inventory
    pub fn get_item_quantity(&self, entity_id: &EntityId, item_id: &ItemId) -> u32 {
        self.get_inventory(entity_id)
            .and_then(|inv| inv.get(item_id))
            .copied()
            .unwrap_or(0)
    }

    /// Check if entity has at least the specified quantity of an item
    pub fn has_item(&self, entity_id: &EntityId, item_id: &ItemId, quantity: u32) -> bool {
        self.get_item_quantity(entity_id, item_id) >= quantity
    }

    /// Transfer item between entities
    pub fn transfer_item(
        &mut self,
        from_entity: &EntityId,
        to_entity: &EntityId,
        item_id: &ItemId,
        quantity: u32,
    ) -> Result<(), InventoryError> {
        // Validate source has enough items
        if !self.has_item(from_entity, item_id, quantity) {
            return Err(InventoryError::ItemNotFound);
        }

        // Remove from source
        self.remove_item(from_entity, item_id, quantity)?;

        // Add to target
        self.add_item(to_entity, item_id, quantity)?;

        Ok(())
    }

    /// Clear an entity's inventory
    pub fn clear_inventory(&mut self, entity_id: &EntityId) {
        self.inventories.remove(entity_id);
    }

    /// Clear all inventories
    pub fn clear_all(&mut self) {
        self.inventories.clear();
    }
}

impl Default for InventoryState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_new() {
        let state = InventoryState::new();
        assert_eq!(state.get_slot_count(&"player_1".to_string()), 0);
        assert_eq!(state.get_total_items(&"player_1".to_string()), 0);
    }

    #[test]
    fn test_add_item() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();
        let item_id = "sword".to_string();

        let result = state.add_item(&entity_id, &item_id, 1);
        assert!(result.is_ok());
        assert_eq!(state.get_item_quantity(&entity_id, &item_id), 1);
    }

    #[test]
    fn test_add_item_stacking() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();
        let item_id = "potion".to_string();

        state.add_item(&entity_id, &item_id, 5).unwrap();
        state.add_item(&entity_id, &item_id, 3).unwrap();

        assert_eq!(state.get_item_quantity(&entity_id, &item_id), 8);
        assert_eq!(state.get_slot_count(&entity_id), 1); // Only one slot used
    }

    #[test]
    fn test_remove_item() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();
        let item_id = "sword".to_string();

        state.add_item(&entity_id, &item_id, 5).unwrap();
        let result = state.remove_item(&entity_id, &item_id, 3);

        assert!(result.is_ok());
        assert_eq!(state.get_item_quantity(&entity_id, &item_id), 2);
    }

    #[test]
    fn test_remove_all_items() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();
        let item_id = "sword".to_string();

        state.add_item(&entity_id, &item_id, 5).unwrap();
        state.remove_item(&entity_id, &item_id, 5).unwrap();

        assert_eq!(state.get_item_quantity(&entity_id, &item_id), 0);
        assert_eq!(state.get_slot_count(&entity_id), 0); // Slot freed
    }

    #[test]
    fn test_remove_item_not_found() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();
        let item_id = "sword".to_string();

        let result = state.remove_item(&entity_id, &item_id, 1);
        assert!(matches!(result, Err(InventoryError::EntityNotFound)));
    }

    #[test]
    fn test_remove_item_insufficient_quantity() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();
        let item_id = "sword".to_string();

        state.add_item(&entity_id, &item_id, 3).unwrap();
        let result = state.remove_item(&entity_id, &item_id, 5);

        assert!(matches!(result, Err(InventoryError::ItemNotFound)));
    }

    #[test]
    fn test_has_item() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();
        let item_id = "potion".to_string();

        state.add_item(&entity_id, &item_id, 5).unwrap();

        assert!(state.has_item(&entity_id, &item_id, 3));
        assert!(state.has_item(&entity_id, &item_id, 5));
        assert!(!state.has_item(&entity_id, &item_id, 6));
    }

    #[test]
    fn test_transfer_item() {
        let mut state = InventoryState::new();
        let from = "player_1".to_string();
        let to = "player_2".to_string();
        let item_id = "sword".to_string();

        state.add_item(&from, &item_id, 5).unwrap();

        let result = state.transfer_item(&from, &to, &item_id, 3);
        assert!(result.is_ok());

        assert_eq!(state.get_item_quantity(&from, &item_id), 2);
        assert_eq!(state.get_item_quantity(&to, &item_id), 3);
    }

    #[test]
    fn test_transfer_item_insufficient() {
        let mut state = InventoryState::new();
        let from = "player_1".to_string();
        let to = "player_2".to_string();
        let item_id = "sword".to_string();

        state.add_item(&from, &item_id, 2).unwrap();

        let result = state.transfer_item(&from, &to, &item_id, 5);
        assert!(matches!(result, Err(InventoryError::ItemNotFound)));
    }

    #[test]
    fn test_get_total_items() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();

        state.add_item(&entity_id, &"sword".to_string(), 1).unwrap();
        state
            .add_item(&entity_id, &"potion".to_string(), 5)
            .unwrap();
        state
            .add_item(&entity_id, &"shield".to_string(), 1)
            .unwrap();

        assert_eq!(state.get_total_items(&entity_id), 7);
        assert_eq!(state.get_slot_count(&entity_id), 3);
    }

    #[test]
    fn test_clear_inventory() {
        let mut state = InventoryState::new();
        let entity_id = "player_1".to_string();

        state.add_item(&entity_id, &"sword".to_string(), 5).unwrap();
        state
            .add_item(&entity_id, &"potion".to_string(), 3)
            .unwrap();

        state.clear_inventory(&entity_id);

        assert_eq!(state.get_total_items(&entity_id), 0);
        assert_eq!(state.get_slot_count(&entity_id), 0);
    }
}
