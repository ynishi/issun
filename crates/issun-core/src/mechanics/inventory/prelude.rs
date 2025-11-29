//! Convenience re-exports for the inventory mechanic.
//!
//! This module provides a convenient way to import all commonly used types
//! and traits for working with the inventory mechanic.
//!
//! # Examples
//!
//! ```
//! use issun_core::mechanics::inventory::prelude::*;
//!
//! // Now you have access to:
//! // - InventoryMechanic
//! // - All strategies (FixedSlotCapacity, AlwaysStack, etc.)
//! // - All presets (BasicInventory, WarehouseInventory, etc.)
//! // - Core types (InventoryConfig, InventoryState, etc.)
//! ```

// Core mechanic
pub use super::mechanic::InventoryMechanic;

// Policies
pub use super::policies::{CapacityPolicy, CostPolicy, StackingPolicy};

// Strategies
pub use super::strategies::{
    AlwaysStack, FixedSlotCapacity, LimitedStack, NeverStack, NoCost, SlotBasedCost,
    UnlimitedCapacity, WeightBasedCapacity, WeightBasedCost,
};

// Presets
pub use super::presets::{
    BasicInventory, LimitedStackInventory, TransportInventory, UniqueItemInventory,
    UnlimitedInventory, VaultInventory, WarehouseInventory, WeightLimitedInventory,
};

// Core types
pub use super::types::{
    InventoryConfig, InventoryEvent, InventoryInput, InventoryOperation, InventoryState, ItemId,
    ItemStack, Quantity, RejectionReason, Weight,
};
