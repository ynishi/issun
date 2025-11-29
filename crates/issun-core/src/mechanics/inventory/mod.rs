//! Inventory mechanic - Physical possession management.
//!
//! This module provides a policy-based inventory system for managing
//! physical possession of items. It focuses purely on **who holds what**,
//! not on legal ownership or rights (which would be a separate mechanic).
//!
//! # Core Concept
//!
//! The inventory mechanic models **physical occupation** of items:
//! - Adding/removing items from containers
//! - Capacity constraints (slots, weight)
//! - Stacking behavior
//! - Holding costs over time
//!
//! # Policy Dimensions
//!
//! The inventory mechanic composes three orthogonal policies:
//!
//! 1. **CapacityPolicy**: How capacity is evaluated
//!    - `FixedSlotCapacity`: Limited by number of slots
//!    - `WeightBasedCapacity`: Limited by total weight
//!    - `UnlimitedCapacity`: No limits
//!
//! 2. **StackingPolicy**: How items are organized
//!    - `AlwaysStack`: Items of same type combine
//!    - `NeverStack`: Each item occupies separate slot
//!    - `LimitedStack`: Stack up to configured maximum
//!
//! 3. **CostPolicy**: How holding costs are calculated
//!    - `NoCost`: Free storage
//!    - `SlotBasedCost`: Cost per occupied slot
//!    - `WeightBasedCost`: Cost per unit weight
//!
//! # Examples
//!
//! ## Basic RPG Inventory
//!
//! ```
//! use issun_core::mechanics::inventory::prelude::*;
//! use issun_core::mechanics::{Mechanic, EventEmitter};
//!
//! // Define inventory type
//! type RPGInventory = InventoryMechanic<
//!     FixedSlotCapacity,
//!     AlwaysStack,
//!     NoCost,
//! >;
//!
//! // Or use preset
//! type RPGInventory2 = BasicInventory;
//!
//! // Configure
//! let config = InventoryConfig {
//!     max_slots: Some(20),
//!     max_weight: None,
//!     max_stack_size: Some(99),
//!     holding_cost_per_slot: 0.0,
//!     holding_cost_per_weight: 0.0,
//! };
//!
//! // Create state
//! let mut state = InventoryState::new();
//!
//! // Add items
//! # struct TestEmitter;
//! # impl EventEmitter<InventoryEvent> for TestEmitter {
//! #     fn emit(&mut self, _event: InventoryEvent) {}
//! # }
//! let mut emitter = TestEmitter;
//! let input = InventoryInput {
//!     operation: InventoryOperation::Add {
//!         stack: ItemStack::new(1, 10),
//!         weight_per_item: 1.0,
//!     },
//!     elapsed_time: 0,
//! };
//!
//! RPGInventory::step(&config, &mut state, input, &mut emitter);
//! ```
//!
//! ## Weight-Limited Inventory (Survival Game)
//!
//! ```
//! use issun_core::mechanics::inventory::prelude::*;
//!
//! type SurvivalInventory = WeightLimitedInventory;
//!
//! let config = InventoryConfig {
//!     max_slots: None,  // No slot limit
//!     max_weight: Some(100.0),  // 100kg carry capacity
//!     ..Default::default()
//! };
//! ```
//!
//! ## Warehouse with Storage Fees
//!
//! ```
//! use issun_core::mechanics::inventory::prelude::*;
//!
//! type Warehouse = WarehouseInventory;
//!
//! let config = InventoryConfig {
//!     max_slots: Some(1000),
//!     holding_cost_per_slot: 5.0,  // 5 gold per slot per day
//!     ..Default::default()
//! };
//! ```
//!
//! # Design Philosophy
//!
//! ## Separation of Concerns
//!
//! This mechanic handles **physical possession only**. Related but separate concepts:
//! - **Ownership rights**: Should be handled by a separate Rights/Claim mechanic
//! - **Item properties**: Defined by game logic, not inventory system
//! - **Transfer between entities**: Coordinated by game logic using Add/Remove
//!
//! ## Composition over Inheritance
//!
//! Different inventory systems are created by composing policies, not by
//! subclassing. This provides:
//! - Zero-cost abstraction (compile-time composition)
//! - Type safety (invalid combinations caught by compiler)
//! - Flexibility (any valid combination of policies)

pub mod mechanic;
pub mod policies;
pub mod prelude;
pub mod presets;
pub mod strategies;
pub mod types;

// Re-export core types at module level
pub use mechanic::InventoryMechanic;
pub use types::{
    InventoryConfig, InventoryEvent, InventoryInput, InventoryOperation, InventoryState, ItemId,
    ItemStack, Quantity, RejectionReason, Weight,
};
