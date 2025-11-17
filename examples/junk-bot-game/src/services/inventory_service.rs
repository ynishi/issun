//! Inventory service - demonstrates Service derive macro
//!
//! This service handles inventory operations (transfer, consume, equip).

use issun::prelude::*;
use issun::Service;  // Import Service derive macro
use crate::models::entities::Weapon;

/// Inventory service for item management
///
/// Demonstrates the Service derive macro in action.
/// Auto-generates: name(), as_any(), as_any_mut()
#[derive(Debug, Clone, Service)]
#[service(name = "inventory_service")]
pub struct InventoryService;

impl InventoryService {
    pub fn new() -> Self {
        Self
    }

    /// Transfer item between inventories
    pub fn transfer_weapon(
        from: &mut Vec<Weapon>,
        to: &mut Vec<Weapon>,
        index: usize,
    ) -> Option<Weapon> {
        if index >= from.len() {
            return None;
        }

        let weapon = from.remove(index);
        to.push(weapon.clone());
        Some(weapon)
    }

    /// Equip weapon (returns old weapon)
    pub fn equip_weapon(
        current: &mut Weapon,
        new_weapon: Weapon,
    ) -> Weapon {
        std::mem::replace(current, new_weapon)
    }

    /// Consume item from inventory
    pub fn consume_weapon(
        inventory: &mut Vec<Weapon>,
        index: usize,
    ) -> Option<Weapon> {
        if index >= inventory.len() {
            return None;
        }
        Some(inventory.remove(index))
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

    #[test]
    fn test_service_derive() {
        let service = InventoryService::new();

        // Service trait methods are auto-generated
        assert_eq!(service.name(), "inventory_service");

        // as_any works
        let any_ref = service.as_any();
        assert!(any_ref.is::<InventoryService>());
    }
}
