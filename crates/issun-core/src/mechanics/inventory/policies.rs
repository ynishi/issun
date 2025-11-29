//! Policy trait definitions for the inventory mechanic.
//!
//! This module defines the three core policy dimensions that can be composed
//! to create different inventory behaviors:
//!
//! 1. **CapacityPolicy**: How to evaluate whether items can be added
//! 2. **StackingPolicy**: How to handle adding items to existing stacks
//! 3. **CostPolicy**: How to calculate holding costs

use super::types::*;

/// Policy for capacity evaluation.
///
/// Determines whether an inventory operation can proceed based on
/// capacity constraints (slots, weight, etc.).
///
/// # Examples
///
/// ```ignore
/// use issun_core::mechanics::inventory::policies::CapacityPolicy;
///
/// struct FixedSlotCapacity;
/// impl CapacityPolicy for FixedSlotCapacity {
///     fn can_add(state: &InventoryState, stack: &ItemStack, config: &InventoryConfig) -> bool {
///         // Check if there's room
///     }
/// }
/// ```
pub trait CapacityPolicy {
    /// Check if items can be added to the inventory.
    ///
    /// # Arguments
    ///
    /// * `state` - Current inventory state
    /// * `stack` - Item stack to add
    /// * `weight_per_item` - Weight of each individual item
    /// * `config` - Inventory configuration
    ///
    /// # Returns
    ///
    /// `Ok(())` if the operation can proceed, or `Err(RejectionReason)` if not.
    fn can_add(
        state: &InventoryState,
        stack: &ItemStack,
        weight_per_item: Weight,
        config: &InventoryConfig,
    ) -> Result<(), RejectionReason>;

    /// Check if items can be removed from the inventory.
    ///
    /// # Arguments
    ///
    /// * `state` - Current inventory state
    /// * `stack` - Item stack to remove
    ///
    /// # Returns
    ///
    /// `Ok(())` if the operation can proceed, or `Err(RejectionReason)` if not.
    fn can_remove(
        state: &InventoryState,
        stack: &ItemStack,
    ) -> Result<(), RejectionReason> {
        // Check if item exists
        let existing = state.find_stack(stack.item_id);

        match existing {
            None => Err(RejectionReason::ItemNotFound),
            Some(existing_stack) if existing_stack.quantity < stack.quantity => {
                Err(RejectionReason::InsufficientQuantity)
            }
            Some(_) => Ok(()),
        }
    }
}

/// Policy for item stacking behavior.
///
/// Determines how items are organized when added to the inventory.
///
/// # Examples
///
/// ```ignore
/// use issun_core::mechanics::inventory::policies::StackingPolicy;
///
/// struct AlwaysStack;
/// impl StackingPolicy for AlwaysStack {
///     fn add_to_inventory(state: &mut InventoryState, stack: ItemStack) {
///         // Try to stack with existing items
///     }
/// }
/// ```
pub trait StackingPolicy {
    /// Add an item stack to the inventory.
    ///
    /// This method handles the logic of whether to:
    /// - Stack with existing items of the same type
    /// - Create a new stack
    /// - Split across multiple stacks
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable inventory state
    /// * `stack` - Item stack to add
    /// * `weight_per_item` - Weight of each individual item
    fn add_to_inventory(
        state: &mut InventoryState,
        stack: ItemStack,
        weight_per_item: Weight,
    );

    /// Remove an item stack from the inventory.
    ///
    /// # Arguments
    ///
    /// * `state` - Mutable inventory state
    /// * `stack` - Item stack to remove
    fn remove_from_inventory(state: &mut InventoryState, stack: ItemStack);
}

/// Policy for holding cost calculation.
///
/// Determines how much it costs to hold items in inventory over time.
///
/// # Examples
///
/// ```ignore
/// use issun_core::mechanics::inventory::policies::CostPolicy;
///
/// struct SlotBasedCost;
/// impl CostPolicy for SlotBasedCost {
///     fn calculate_cost(state: &InventoryState, config: &InventoryConfig, elapsed_time: u32) -> f32 {
///         // Calculate based on slots used
///     }
/// }
/// ```
pub trait CostPolicy {
    /// Calculate the holding cost for the current inventory state.
    ///
    /// # Arguments
    ///
    /// * `state` - Current inventory state
    /// * `config` - Inventory configuration
    /// * `elapsed_time` - Time units elapsed since last calculation
    ///
    /// # Returns
    ///
    /// Total holding cost for the elapsed time period.
    fn calculate_cost(
        state: &InventoryState,
        config: &InventoryConfig,
        elapsed_time: u32,
    ) -> f32;
}
