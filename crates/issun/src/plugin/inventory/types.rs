//! Inventory types and traits

use serde::{Deserialize, Serialize};

/// Item trait for inventory management
///
/// Implement this trait for game items that can be stored in inventories.
///
/// # Example
///
/// ```ignore
/// #[derive(Clone, Debug)]
/// pub struct Weapon {
///     pub name: String,
///     pub attack: i32,
/// }
///
/// impl Item for Weapon {}
/// ```
pub trait Item: Clone + Send + Sync + 'static {}

/// Auto-implement Item for any type that satisfies the bounds
impl<T> Item for T where T: Clone + Send + Sync + 'static {}

/// Unique identifier for an item type
pub type ItemId = String;

/// Unique identifier for an entity (player, NPC, container, etc.)
pub type EntityId = String;

/// Error types for inventory operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryError {
    /// Entity not found
    EntityNotFound,
    /// Item not found in inventory
    ItemNotFound,
    /// Inventory is full
    InventoryFull,
    /// Cannot perform operation (e.g., equip non-equippable item)
    InvalidOperation(String),
}

impl std::fmt::Display for InventoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InventoryError::EntityNotFound => write!(f, "Entity not found"),
            InventoryError::ItemNotFound => write!(f, "Item not found in inventory"),
            InventoryError::InventoryFull => write!(f, "Inventory is full"),
            InventoryError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl std::error::Error for InventoryError {}
