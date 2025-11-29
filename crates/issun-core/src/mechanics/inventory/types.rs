//! Core types for the inventory mechanic.
//!
//! This module defines the fundamental data structures used by the inventory mechanic:
//! - Config: Static configuration (capacity limits, stacking rules, holding costs)
//! - Input: Per-operation input data (add/remove/transfer operations)
//! - Event: Events emitted when inventory state changes occur
//! - State: Per-entity mutable state (items held, weight, slots used)

/// Item identifier type.
///
/// In a real game, this would typically map to an asset ID or item definition.
/// The inventory mechanic treats this as an opaque identifier.
pub type ItemId = u64;

/// Quantity type for stackable items.
pub type Quantity = u32;

/// Weight type for weight-based capacity systems.
pub type Weight = f32;

/// An item stack in the inventory.
///
/// Represents a quantity of a specific item type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemStack {
    /// The item identifier
    pub item_id: ItemId,
    /// How many of this item
    pub quantity: Quantity,
}

impl ItemStack {
    /// Create a new item stack.
    pub fn new(item_id: ItemId, quantity: Quantity) -> Self {
        Self { item_id, quantity }
    }
}

/// Configuration for the inventory mechanic.
///
/// This type is typically stored as a resource in the game engine and
/// shared across all entities using the inventory mechanic.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::InventoryConfig;
///
/// // RPG character inventory (20 slots)
/// let config = InventoryConfig {
///     max_slots: Some(20),
///     max_weight: None,
///     max_stack_size: Some(99),
///     holding_cost_per_slot: 0.0,
///     holding_cost_per_weight: 0.0,
/// };
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryConfig {
    /// Maximum number of inventory slots (None = unlimited)
    pub max_slots: Option<usize>,
    /// Maximum total weight (None = no weight limit)
    pub max_weight: Option<Weight>,
    /// Maximum stack size per item type (None = unlimited stacking)
    pub max_stack_size: Option<Quantity>,
    /// Cost per occupied slot per time unit
    pub holding_cost_per_slot: f32,
    /// Cost per unit weight per time unit
    pub holding_cost_per_weight: f32,
}

impl Default for InventoryConfig {
    fn default() -> Self {
        Self {
            max_slots: Some(20),      // Default: 20 slots
            max_weight: None,         // Default: no weight limit
            max_stack_size: Some(99), // Default: stack up to 99
            holding_cost_per_slot: 0.0,
            holding_cost_per_weight: 0.0,
        }
    }
}

/// Per-operation input for the inventory mechanic.
///
/// This type is constructed for each inventory operation.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::{InventoryInput, InventoryOperation, ItemStack};
///
/// // Add 5 health potions
/// let input = InventoryInput {
///     operation: InventoryOperation::Add {
///         stack: ItemStack::new(101, 5),
///         weight_per_item: 0.5,
///     },
///     elapsed_time: 0,
/// };
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct InventoryInput {
    /// The operation to perform
    pub operation: InventoryOperation,
    /// Time units elapsed (for holding cost calculation)
    pub elapsed_time: u32,
}

impl Default for InventoryInput {
    fn default() -> Self {
        Self {
            operation: InventoryOperation::Add {
                stack: ItemStack::new(0, 0),
                weight_per_item: 0.0,
            },
            elapsed_time: 0,
        }
    }
}

/// Inventory operations.
///
/// Defines the possible operations that can be performed on an inventory.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InventoryOperation {
    /// Add items to the inventory
    Add {
        /// The item stack to add
        stack: ItemStack,
        /// Weight per individual item
        weight_per_item: Weight,
    },

    /// Remove items from the inventory
    Remove {
        /// The item stack to remove
        stack: ItemStack,
    },

    /// Transfer items (used for validation, actual transfer handled by game logic)
    Transfer {
        /// The item stack to transfer
        stack: ItemStack,
    },
}

/// Per-entity mutable state for the inventory mechanic.
///
/// This type is stored as a component on each entity.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::{InventoryState, ItemStack};
///
/// let mut state = InventoryState::new();
/// assert_eq!(state.stacks.len(), 0);
/// assert_eq!(state.total_weight, 0.0);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct InventoryState {
    /// Item stacks currently held
    pub stacks: Vec<ItemStack>,
    /// Total weight of all items
    pub total_weight: Weight,
    /// Number of occupied slots
    pub occupied_slots: usize,
}

impl InventoryState {
    /// Create a new empty inventory state.
    pub fn new() -> Self {
        Self {
            stacks: Vec::new(),
            total_weight: 0.0,
            occupied_slots: 0,
        }
    }

    /// Find a stack by item ID.
    pub fn find_stack(&self, item_id: ItemId) -> Option<&ItemStack> {
        self.stacks.iter().find(|s| s.item_id == item_id)
    }

    /// Find a mutable stack by item ID.
    pub fn find_stack_mut(&mut self, item_id: ItemId) -> Option<&mut ItemStack> {
        self.stacks.iter_mut().find(|s| s.item_id == item_id)
    }

    /// Get the total quantity of a specific item.
    pub fn quantity_of(&self, item_id: ItemId) -> Quantity {
        self.find_stack(item_id).map(|s| s.quantity).unwrap_or(0)
    }
}

impl Default for InventoryState {
    fn default() -> Self {
        Self::new()
    }
}

/// Events emitted by the inventory mechanic.
///
/// These events communicate state changes to the game world without
/// coupling the mechanic to any specific engine.
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::{InventoryEvent, ItemStack};
///
/// // Check if item was added
/// match (InventoryEvent::ItemAdded { stack: ItemStack::new(1, 5) }) {
///     InventoryEvent::ItemAdded { stack } => {
///         println!("Added {} of item {}", stack.quantity, stack.item_id);
///     }
///     _ => {}
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum InventoryEvent {
    /// Item stack was added to inventory
    ItemAdded {
        /// The stack that was added
        stack: ItemStack,
    },

    /// Item stack was removed from inventory
    ItemRemoved {
        /// The stack that was removed
        stack: ItemStack,
    },

    /// Inventory reached maximum capacity (slots)
    CapacityReached {
        /// Current number of occupied slots
        occupied_slots: usize,
        /// Maximum allowed slots
        max_slots: usize,
    },

    /// Inventory reached maximum weight
    WeightLimitReached {
        /// Current total weight
        current_weight: Weight,
        /// Maximum allowed weight
        max_weight: Weight,
    },

    /// Operation rejected due to insufficient capacity
    OperationRejected {
        /// The operation that was rejected
        operation: InventoryOperation,
        /// Reason for rejection
        reason: RejectionReason,
    },

    /// Holding cost applied
    HoldingCostApplied {
        /// Total cost for this time period
        cost: f32,
    },
}

/// Reasons why an inventory operation might be rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RejectionReason {
    /// Not enough slots available
    InsufficientSlots,
    /// Weight limit would be exceeded
    WeightLimitExceeded,
    /// Item not found (for removal)
    ItemNotFound,
    /// Not enough quantity (for removal)
    InsufficientQuantity,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_item_stack_creation() {
        let stack = ItemStack::new(42, 10);
        assert_eq!(stack.item_id, 42);
        assert_eq!(stack.quantity, 10);
    }

    #[test]
    fn test_inventory_config_default() {
        let config = InventoryConfig::default();
        assert_eq!(config.max_slots, Some(20));
        assert_eq!(config.max_weight, None);
        assert_eq!(config.holding_cost_per_slot, 0.0);
    }

    #[test]
    fn test_inventory_state_new() {
        let state = InventoryState::new();
        assert_eq!(state.stacks.len(), 0);
        assert_eq!(state.total_weight, 0.0);
        assert_eq!(state.occupied_slots, 0);
    }

    #[test]
    fn test_inventory_state_find_stack() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 5));
        state.stacks.push(ItemStack::new(2, 3));

        let stack = state.find_stack(1);
        assert!(stack.is_some());
        assert_eq!(stack.unwrap().quantity, 5);

        let missing = state.find_stack(99);
        assert!(missing.is_none());
    }

    #[test]
    fn test_inventory_state_quantity_of() {
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 10));

        assert_eq!(state.quantity_of(1), 10);
        assert_eq!(state.quantity_of(2), 0);
    }
}
