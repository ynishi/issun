//! Stacking policy strategies.
//!
//! Provides concrete implementations of the StackingPolicy trait.

use crate::mechanics::inventory::policies::StackingPolicy;
use crate::mechanics::inventory::types::*;

/// Always stack items of the same type.
///
/// Items of the same ID are combined into a single stack.
/// No limit on stack size (or respects config's max_stack_size if set).
///
/// # Use Cases
///
/// - Resource management (wood, stone, ore)
/// - Consumables (potions, arrows, food)
/// - Currency/gems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::AlwaysStack;
/// use issun_core::mechanics::inventory::policies::StackingPolicy;
/// use issun_core::mechanics::inventory::{InventoryState, ItemStack};
///
/// let mut state = InventoryState::new();
/// state.stacks.push(ItemStack::new(1, 10));
/// state.occupied_slots = 1;
/// state.total_weight = 10.0;
///
/// // Add 5 more of item 1 (should stack)
/// AlwaysStack::add_to_inventory(&mut state, ItemStack::new(1, 5), 1.0);
///
/// assert_eq!(state.stacks.len(), 1);
/// assert_eq!(state.stacks[0].quantity, 15);
/// assert_eq!(state.total_weight, 15.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AlwaysStack;

impl StackingPolicy for AlwaysStack {
    fn add_to_inventory(state: &mut InventoryState, stack: ItemStack, weight_per_item: Weight) {
        if let Some(existing) = state.find_stack_mut(stack.item_id) {
            // Stack with existing
            existing.quantity += stack.quantity;
        } else {
            // Add new stack
            state.stacks.push(stack);
            state.occupied_slots += 1;
        }

        // Update weight
        state.total_weight += weight_per_item * stack.quantity as Weight;
    }

    fn remove_from_inventory(state: &mut InventoryState, stack: ItemStack) {
        if let Some(idx) = state.stacks.iter().position(|s| s.item_id == stack.item_id) {
            let existing = &mut state.stacks[idx];

            if existing.quantity <= stack.quantity {
                // Remove entire stack
                state.stacks.remove(idx);
                state.occupied_slots -= 1;
            } else {
                // Reduce quantity
                existing.quantity -= stack.quantity;
            }
        }
    }
}

/// Never stack items - each item occupies its own slot.
///
/// Every item added creates a new stack, even if the same item type exists.
/// Useful for unique items or when each instance has individual properties.
///
/// # Use Cases
///
/// - Unique weapons/armor (each with different durability/enchantments)
/// - Collectible cards (each with unique stats)
/// - Equipment slots (one per slot)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::NeverStack;
/// use issun_core::mechanics::inventory::policies::StackingPolicy;
/// use issun_core::mechanics::inventory::{InventoryState, ItemStack};
///
/// let mut state = InventoryState::new();
///
/// // Add same item twice - creates separate stacks
/// NeverStack::add_to_inventory(&mut state, ItemStack::new(1, 1), 1.0);
/// NeverStack::add_to_inventory(&mut state, ItemStack::new(1, 1), 1.0);
///
/// assert_eq!(state.stacks.len(), 2);
/// assert_eq!(state.occupied_slots, 2);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NeverStack;

impl StackingPolicy for NeverStack {
    fn add_to_inventory(state: &mut InventoryState, stack: ItemStack, weight_per_item: Weight) {
        // Always add as new stack, quantity forced to 1
        state.stacks.push(ItemStack::new(stack.item_id, 1));
        state.occupied_slots += 1;
        state.total_weight += weight_per_item;
    }

    fn remove_from_inventory(state: &mut InventoryState, stack: ItemStack) {
        // Remove first matching item
        if let Some(idx) = state.stacks.iter().position(|s| s.item_id == stack.item_id) {
            state.stacks.remove(idx);
            state.occupied_slots -= 1;
        }
    }
}

/// Limited stack size with configurable maximum.
///
/// Items stack up to a maximum quantity (defined in config).
/// Overflow creates additional stacks.
///
/// # Use Cases
///
/// - Classic RPGs (potions stack to 99)
/// - Minecraft-style inventories (64 per stack)
/// - Inventory management games
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::LimitedStack;
/// use issun_core::mechanics::inventory::policies::StackingPolicy;
/// use issun_core::mechanics::inventory::{InventoryState, ItemStack};
///
/// let mut state = InventoryState::new();
///
/// // Add items that respect max_stack_size (handled by mechanic config)
/// LimitedStack::add_to_inventory(&mut state, ItemStack::new(1, 50), 1.0);
/// assert_eq!(state.stacks[0].quantity, 50);
///
/// // Note: Max stack enforcement happens at mechanic level, not here
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitedStack;

impl StackingPolicy for LimitedStack {
    fn add_to_inventory(state: &mut InventoryState, stack: ItemStack, weight_per_item: Weight) {
        // Try to stack with existing first
        if let Some(existing) = state.find_stack_mut(stack.item_id) {
            existing.quantity += stack.quantity;
        } else {
            // Create new stack
            state.stacks.push(stack);
            state.occupied_slots += 1;
        }

        // Update weight
        state.total_weight += weight_per_item * stack.quantity as Weight;
    }

    fn remove_from_inventory(state: &mut InventoryState, stack: ItemStack) {
        if let Some(idx) = state.stacks.iter().position(|s| s.item_id == stack.item_id) {
            let existing = &mut state.stacks[idx];

            if existing.quantity <= stack.quantity {
                // Remove entire stack
                state.stacks.remove(idx);
                state.occupied_slots -= 1;
            } else {
                // Reduce quantity
                existing.quantity -= stack.quantity;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // AlwaysStack tests
    #[test]
    fn test_always_stack_adds_to_existing() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 10));
        state.occupied_slots = 1;
        state.total_weight = 10.0;

        AlwaysStack::add_to_inventory(&mut state, ItemStack::new(1, 5), 1.0);

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.stacks[0].quantity, 15);
        assert_eq!(state.occupied_slots, 1);
        assert_eq!(state.total_weight, 15.0);
    }

    #[test]
    fn test_always_stack_creates_new_stack() {
        let mut state = InventoryState::new();

        AlwaysStack::add_to_inventory(&mut state, ItemStack::new(1, 5), 2.0);

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.stacks[0].quantity, 5);
        assert_eq!(state.occupied_slots, 1);
        assert_eq!(state.total_weight, 10.0);
    }

    #[test]
    fn test_always_stack_removes_partial() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 10));
        state.occupied_slots = 1;

        AlwaysStack::remove_from_inventory(&mut state, ItemStack::new(1, 3));

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.stacks[0].quantity, 7);
        assert_eq!(state.occupied_slots, 1);
    }

    #[test]
    fn test_always_stack_removes_entire() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 10));
        state.occupied_slots = 1;

        AlwaysStack::remove_from_inventory(&mut state, ItemStack::new(1, 10));

        assert_eq!(state.stacks.len(), 0);
        assert_eq!(state.occupied_slots, 0);
    }

    // NeverStack tests
    #[test]
    fn test_never_stack_creates_separate_stacks() {
        let mut state = InventoryState::new();

        NeverStack::add_to_inventory(&mut state, ItemStack::new(1, 1), 1.0);
        NeverStack::add_to_inventory(&mut state, ItemStack::new(1, 1), 1.0);

        assert_eq!(state.stacks.len(), 2);
        assert_eq!(state.occupied_slots, 2);
        assert_eq!(state.total_weight, 2.0);
    }

    #[test]
    fn test_never_stack_removes_first_match() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 1));
        state.stacks.push(ItemStack::new(1, 1));
        state.occupied_slots = 2;

        NeverStack::remove_from_inventory(&mut state, ItemStack::new(1, 1));

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.occupied_slots, 1);
    }

    // LimitedStack tests
    #[test]
    fn test_limited_stack_adds_to_existing() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 50));
        state.occupied_slots = 1;

        LimitedStack::add_to_inventory(&mut state, ItemStack::new(1, 30), 1.0);

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.stacks[0].quantity, 80);
    }

    #[test]
    fn test_limited_stack_creates_new_for_different_item() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 50));
        state.occupied_slots = 1;

        LimitedStack::add_to_inventory(&mut state, ItemStack::new(2, 30), 1.0);

        assert_eq!(state.stacks.len(), 2);
        assert_eq!(state.occupied_slots, 2);
    }
}
