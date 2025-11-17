//! Inventory service for item management
//!
//! Provides centralized inventory operations: transfer, equip, consume.
//! Follows Domain-Driven Design principles - inventory logic as a service.

use super::types::Item;

/// Inventory service providing centralized item management
///
/// This service handles all inventory-related operations, similar to
/// how CombatService handles damage calculations. Ensures consistent
/// item transfer, equipment swapping, and consumption.
///
/// # Example
///
/// ```ignore
/// let service = InventoryService::new();
/// let item = service.transfer_item(&mut from_inv, &mut to_inv, 0);
/// ```
#[derive(Debug, Clone, issun_macros::Service)]
#[service(name = "inventory_service")]
pub struct InventoryService;

impl InventoryService {
    /// Create a new inventory service
    pub fn new() -> Self {
        Self
    }

    /// Transfer item between inventories
    ///
    /// # Arguments
    ///
    /// * `from` - Source inventory (mutable reference)
    /// * `to` - Target inventory (mutable reference)
    /// * `index` - Index of item to transfer
    ///
    /// # Returns
    ///
    /// Some(item) if transfer successful, None if index out of bounds
    pub fn transfer_item<T: Item>(from: &mut Vec<T>, to: &mut Vec<T>, index: usize) -> Option<T> {
        if index >= from.len() {
            return None;
        }

        let item = from.remove(index);
        to.push(item.clone());
        Some(item)
    }

    /// Equip item (swap with current)
    ///
    /// # Arguments
    ///
    /// * `current` - Currently equipped item (mutable reference)
    /// * `new_item` - New item to equip
    ///
    /// # Returns
    ///
    /// Previously equipped item
    pub fn equip_item<T: Item>(current: &mut T, new_item: T) -> T {
        std::mem::replace(current, new_item)
    }

    /// Consume item from inventory (remove and return)
    ///
    /// # Arguments
    ///
    /// * `inventory` - Inventory to consume from (mutable reference)
    /// * `index` - Index of item to consume
    ///
    /// # Returns
    ///
    /// Some(item) if consumption successful, None if index out of bounds
    pub fn consume_item<T: Item>(inventory: &mut Vec<T>, index: usize) -> Option<T> {
        if index >= inventory.len() {
            return None;
        }
        Some(inventory.remove(index))
    }

    /// Add item to inventory
    ///
    /// # Arguments
    ///
    /// * `inventory` - Inventory to add to (mutable reference)
    /// * `item` - Item to add
    pub fn add_item<T: Item>(inventory: &mut Vec<T>, item: T) {
        inventory.push(item);
    }

    /// Remove item at index (without returning)
    ///
    /// # Arguments
    ///
    /// * `inventory` - Inventory to remove from (mutable reference)
    /// * `index` - Index of item to remove
    ///
    /// # Returns
    ///
    /// true if removal successful, false if index out of bounds
    pub fn remove_item<T: Item>(inventory: &mut Vec<T>, index: usize) -> bool {
        if index >= inventory.len() {
            return false;
        }
        inventory.remove(index);
        true
    }

    /// Get inventory size
    pub fn inventory_size<T: Item>(inventory: &[T]) -> usize {
        inventory.len()
    }

    /// Check if inventory is empty
    pub fn is_empty<T: Item>(inventory: &[T]) -> bool {
        inventory.is_empty()
    }
}

impl Default for InventoryService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock item for testing
    #[derive(Clone, Debug, PartialEq, Eq)]
    struct MockItem {
        name: String,
        value: i32,
    }

    #[test]
    fn test_transfer_item() {
        let mut from = vec![
            MockItem {
                name: "Sword".to_string(),
                value: 100,
            },
            MockItem {
                name: "Shield".to_string(),
                value: 50,
            },
        ];
        let mut to = Vec::new();

        let transferred = InventoryService::transfer_item(&mut from, &mut to, 0);

        assert!(transferred.is_some());
        assert_eq!(transferred.unwrap().name, "Sword");
        assert_eq!(from.len(), 1);
        assert_eq!(to.len(), 1);
        assert_eq!(to[0].name, "Sword");
    }

    #[test]
    fn test_transfer_item_invalid_index() {
        let mut from = vec![MockItem {
            name: "Sword".to_string(),
            value: 100,
        }];
        let mut to = Vec::new();

        let result = InventoryService::transfer_item(&mut from, &mut to, 10);

        assert!(result.is_none());
        assert_eq!(from.len(), 1);
        assert_eq!(to.len(), 0);
    }

    #[test]
    fn test_equip_item() {
        let mut current = MockItem {
            name: "Old Sword".to_string(),
            value: 50,
        };
        let new = MockItem {
            name: "New Sword".to_string(),
            value: 100,
        };

        let old = InventoryService::equip_item(&mut current, new);

        assert_eq!(old.name, "Old Sword");
        assert_eq!(current.name, "New Sword");
    }

    #[test]
    fn test_consume_item() {
        let mut inventory = vec![
            MockItem {
                name: "Potion".to_string(),
                value: 20,
            },
            MockItem {
                name: "Elixir".to_string(),
                value: 50,
            },
        ];

        let consumed = InventoryService::consume_item(&mut inventory, 0);

        assert!(consumed.is_some());
        assert_eq!(consumed.unwrap().name, "Potion");
        assert_eq!(inventory.len(), 1);
        assert_eq!(inventory[0].name, "Elixir");
    }

    #[test]
    fn test_add_item() {
        let mut inventory = Vec::new();
        let item = MockItem {
            name: "Sword".to_string(),
            value: 100,
        };

        InventoryService::add_item(&mut inventory, item);

        assert_eq!(inventory.len(), 1);
        assert_eq!(inventory[0].name, "Sword");
    }

    #[test]
    fn test_remove_item() {
        let mut inventory = vec![MockItem {
            name: "Sword".to_string(),
            value: 100,
        }];

        let result = InventoryService::remove_item(&mut inventory, 0);

        assert!(result);
        assert_eq!(inventory.len(), 0);
    }

    #[test]
    fn test_inventory_size() {
        let inventory = vec![
            MockItem {
                name: "Sword".to_string(),
                value: 100,
            },
            MockItem {
                name: "Shield".to_string(),
                value: 50,
            },
        ];

        assert_eq!(InventoryService::inventory_size(&inventory), 2);
    }

    #[test]
    fn test_is_empty() {
        let empty: Vec<MockItem> = Vec::new();
        let non_empty = vec![MockItem {
            name: "Sword".to_string(),
            value: 100,
        }];

        assert!(InventoryService::is_empty(&empty));
        assert!(!InventoryService::is_empty(&non_empty));
    }
}
