//! Capacity policy strategies.
//!
//! Provides concrete implementations of the CapacityPolicy trait.

use crate::mechanics::inventory::policies::CapacityPolicy;
use crate::mechanics::inventory::types::*;

/// Fixed slot capacity strategy.
///
/// Limits inventory by number of slots. Each unique item type occupies one slot,
/// regardless of quantity (assumes stacking).
///
/// # Use Cases
///
/// - RPG character inventories (20-slot backpack)
/// - Grid-based inventories (Diablo, Resident Evil)
/// - Simple item limits
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::FixedSlotCapacity;
/// use issun_core::mechanics::inventory::policies::CapacityPolicy;
/// use issun_core::mechanics::inventory::{InventoryConfig, InventoryState, ItemStack};
///
/// let config = InventoryConfig {
///     max_slots: Some(5),
///     ..Default::default()
/// };
///
/// let mut state = InventoryState::new();
/// state.stacks.push(ItemStack::new(1, 10));
/// state.stacks.push(ItemStack::new(2, 5));
/// state.occupied_slots = 2;
///
/// // Can add new item (2/5 slots used)
/// let result = FixedSlotCapacity::can_add(&state, &ItemStack::new(3, 1), 0.0, &config);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedSlotCapacity;

impl CapacityPolicy for FixedSlotCapacity {
    fn can_add(
        state: &InventoryState,
        stack: &ItemStack,
        _weight_per_item: Weight,
        config: &InventoryConfig,
    ) -> Result<(), RejectionReason> {
        // Check if we have slot limit
        let Some(max_slots) = config.max_slots else {
            return Ok(()); // No limit
        };

        // If item already exists, we can stack (no new slot needed)
        if state.find_stack(stack.item_id).is_some() {
            return Ok(());
        }

        // Check if we have room for a new slot
        if state.occupied_slots >= max_slots {
            Err(RejectionReason::InsufficientSlots)
        } else {
            Ok(())
        }
    }
}

/// Weight-based capacity strategy.
///
/// Limits inventory by total weight carried.
/// Useful for realistic simulations where carrying capacity matters.
///
/// # Use Cases
///
/// - Survival games (weight affects movement speed)
/// - Realistic RPGs (Skyrim, Fallout)
/// - Logistics simulations
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::WeightBasedCapacity;
/// use issun_core::mechanics::inventory::policies::CapacityPolicy;
/// use issun_core::mechanics::inventory::{InventoryConfig, InventoryState, ItemStack};
///
/// let config = InventoryConfig {
///     max_weight: Some(100.0),
///     ..Default::default()
/// };
///
/// let mut state = InventoryState::new();
/// state.total_weight = 80.0;
///
/// // Can add 10 items at 1.5kg each? (80 + 15 = 95, under 100)
/// let result = WeightBasedCapacity::can_add(&state, &ItemStack::new(1, 10), 1.5, &config);
/// assert!(result.is_ok());
///
/// // Cannot add 20 items (80 + 30 = 110, over 100)
/// let result = WeightBasedCapacity::can_add(&state, &ItemStack::new(2, 20), 1.5, &config);
/// assert!(result.is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightBasedCapacity;

impl CapacityPolicy for WeightBasedCapacity {
    fn can_add(
        state: &InventoryState,
        stack: &ItemStack,
        weight_per_item: Weight,
        config: &InventoryConfig,
    ) -> Result<(), RejectionReason> {
        // Check if we have weight limit
        let Some(max_weight) = config.max_weight else {
            return Ok(()); // No limit
        };

        // Calculate total weight if we add this stack
        let added_weight = weight_per_item * stack.quantity as Weight;
        let new_total_weight = state.total_weight + added_weight;

        if new_total_weight > max_weight {
            Err(RejectionReason::WeightLimitExceeded)
        } else {
            Ok(())
        }
    }
}

/// Unlimited capacity strategy.
///
/// No capacity restrictions. Items can be added freely.
///
/// # Use Cases
///
/// - Creative/sandbox modes
/// - Storage chests with unlimited space
/// - Abstract inventories (quest items, currencies)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::UnlimitedCapacity;
/// use issun_core::mechanics::inventory::policies::CapacityPolicy;
/// use issun_core::mechanics::inventory::{InventoryConfig, InventoryState, ItemStack};
///
/// let config = InventoryConfig::default();
/// let state = InventoryState::new();
///
/// // Always succeeds
/// let result = UnlimitedCapacity::can_add(&state, &ItemStack::new(1, 999999), 100.0, &config);
/// assert!(result.is_ok());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UnlimitedCapacity;

impl CapacityPolicy for UnlimitedCapacity {
    fn can_add(
        _state: &InventoryState,
        _stack: &ItemStack,
        _weight_per_item: Weight,
        _config: &InventoryConfig,
    ) -> Result<(), RejectionReason> {
        Ok(()) // Always allow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> InventoryConfig {
        InventoryConfig {
            max_slots: Some(10),
            max_weight: Some(100.0),
            max_stack_size: Some(99),
            holding_cost_per_slot: 0.0,
            holding_cost_per_weight: 0.0,
        }
    }

    // FixedSlotCapacity tests
    #[test]
    fn test_fixed_slot_can_add_to_existing_stack() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 5));
        state.occupied_slots = 1;

        // Adding to existing stack should always work (no new slot needed)
        let result = FixedSlotCapacity::can_add(&state, &ItemStack::new(1, 10), 0.0, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_slot_can_add_new_item() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.occupied_slots = 5;

        // 5/10 slots used, should be able to add new item
        let result = FixedSlotCapacity::can_add(&state, &ItemStack::new(1, 1), 0.0, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fixed_slot_full_inventory() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.occupied_slots = 10;

        // 10/10 slots used, cannot add new item
        let result = FixedSlotCapacity::can_add(&state, &ItemStack::new(99, 1), 0.0, &config);
        assert_eq!(result, Err(RejectionReason::InsufficientSlots));
    }

    #[test]
    fn test_fixed_slot_unlimited() {
        let config = InventoryConfig {
            max_slots: None,
            ..default_config()
        };
        let mut state = InventoryState::new();
        state.occupied_slots = 9999;

        // No slot limit, always succeeds
        let result = FixedSlotCapacity::can_add(&state, &ItemStack::new(1, 1), 0.0, &config);
        assert!(result.is_ok());
    }

    // WeightBasedCapacity tests
    #[test]
    fn test_weight_based_under_limit() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.total_weight = 50.0;

        // Adding 40kg total (50 + 40 = 90, under 100)
        let result = WeightBasedCapacity::can_add(&state, &ItemStack::new(1, 10), 4.0, &config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_weight_based_over_limit() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.total_weight = 90.0;

        // Adding 20kg total (90 + 20 = 110, over 100)
        let result = WeightBasedCapacity::can_add(&state, &ItemStack::new(1, 10), 2.0, &config);
        assert_eq!(result, Err(RejectionReason::WeightLimitExceeded));
    }

    #[test]
    fn test_weight_based_unlimited() {
        let config = InventoryConfig {
            max_weight: None,
            ..default_config()
        };
        let mut state = InventoryState::new();
        state.total_weight = 9999.0;

        // No weight limit, always succeeds
        let result = WeightBasedCapacity::can_add(&state, &ItemStack::new(1, 100), 100.0, &config);
        assert!(result.is_ok());
    }

    // UnlimitedCapacity tests
    #[test]
    fn test_unlimited_capacity_always_succeeds() {
        let config = default_config();
        let state = InventoryState::new();

        let result = UnlimitedCapacity::can_add(&state, &ItemStack::new(1, 999999), 999.0, &config);
        assert!(result.is_ok());
    }
}
