//! Preset inventory configurations.
//!
//! This module provides ready-to-use inventory mechanic configurations
//! for common use cases. Each preset is a type alias that combines
//! specific capacity, stacking, and cost policies.

use super::mechanic::InventoryMechanic;
use super::strategies::*;

/// Basic RPG inventory.
///
/// **Characteristics:**
/// - Fixed slot capacity (defined in config)
/// - Items stack automatically
/// - No holding cost
///
/// **Use Cases:**
/// - Traditional RPG inventories
/// - Action-adventure games
/// - Most standard inventory systems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
/// use issun_core::mechanics::Mechanic;
///
/// type MyInventory = BasicInventory;
///
/// let config = InventoryConfig {
///     max_slots: Some(20),
///     ..Default::default()
/// };
/// ```
pub type BasicInventory = InventoryMechanic<FixedSlotCapacity, AlwaysStack, NoCost>;

/// Equipment inventory (non-stacking).
///
/// **Characteristics:**
/// - Fixed slot capacity
/// - Items never stack (each item is unique)
/// - No holding cost
///
/// **Use Cases:**
/// - Equipment slots (weapon, armor, accessories)
/// - Unique item collections
/// - Card/collectible inventories
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
///
/// type EquipmentSlots = UniqueItemInventory;
/// ```
pub type UniqueItemInventory = InventoryMechanic<FixedSlotCapacity, NeverStack, NoCost>;

/// Weight-limited inventory.
///
/// **Characteristics:**
/// - Weight-based capacity
/// - Items stack automatically
/// - No holding cost
///
/// **Use Cases:**
/// - Survival games (Skyrim, Fallout)
/// - Realistic RPGs
/// - Logistics simulations
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
///
/// type CarryingCapacity = WeightLimitedInventory;
///
/// let config = InventoryConfig {
///     max_weight: Some(100.0),
///     ..Default::default()
/// };
/// ```
pub type WeightLimitedInventory = InventoryMechanic<WeightBasedCapacity, AlwaysStack, NoCost>;

/// Unlimited storage.
///
/// **Characteristics:**
/// - No capacity limits
/// - Items stack automatically
/// - No holding cost
///
/// **Use Cases:**
/// - Creative mode inventories
/// - Quest item storage
/// - Abstract resource pools
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
///
/// type InfiniteStorage = UnlimitedInventory;
/// ```
pub type UnlimitedInventory = InventoryMechanic<UnlimitedCapacity, AlwaysStack, NoCost>;

/// Warehouse with slot-based fees.
///
/// **Characteristics:**
/// - Fixed slot capacity
/// - Items stack automatically
/// - Slot-based holding cost
///
/// **Use Cases:**
/// - Warehouse management games
/// - Storage rental systems
/// - Bank vaults with fees
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
///
/// type PaidStorage = WarehouseInventory;
///
/// let config = InventoryConfig {
///     max_slots: Some(100),
///     holding_cost_per_slot: 10.0,
///     ..Default::default()
/// };
/// ```
pub type WarehouseInventory = InventoryMechanic<FixedSlotCapacity, AlwaysStack, SlotBasedCost>;

/// Cargo transport with weight-based fees.
///
/// **Characteristics:**
/// - Weight-based capacity
/// - Items stack automatically
/// - Weight-based holding cost
///
/// **Use Cases:**
/// - Shipping/cargo games
/// - Transport simulations
/// - Freight management
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
///
/// type CargoHold = TransportInventory;
///
/// let config = InventoryConfig {
///     max_weight: Some(1000.0),
///     holding_cost_per_weight: 0.5,
///     ..Default::default()
/// };
/// ```
pub type TransportInventory = InventoryMechanic<WeightBasedCapacity, AlwaysStack, WeightBasedCost>;

/// Limited stack inventory (Minecraft-style).
///
/// **Characteristics:**
/// - Fixed slot capacity
/// - Limited stack size (e.g., 64 per stack)
/// - No holding cost
///
/// **Use Cases:**
/// - Minecraft-style inventories
/// - Stack-limited resource management
/// - Grid inventories with stack limits
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
///
/// type BlockInventory = LimitedStackInventory;
///
/// let config = InventoryConfig {
///     max_slots: Some(36),
///     max_stack_size: Some(64),
///     ..Default::default()
/// };
/// ```
pub type LimitedStackInventory = InventoryMechanic<FixedSlotCapacity, LimitedStack, NoCost>;

/// Vault storage with comprehensive fees.
///
/// **Characteristics:**
/// - Weight-based capacity
/// - Limited stack size
/// - Weight-based holding cost
///
/// **Use Cases:**
/// - Bank vault simulations
/// - Precious item storage
/// - High-security storage systems
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
///
/// type SecureVault = VaultInventory;
///
/// let config = InventoryConfig {
///     max_weight: Some(500.0),
///     max_stack_size: Some(10),
///     holding_cost_per_weight: 1.0,
///     ..Default::default()
/// };
/// ```
pub type VaultInventory = InventoryMechanic<WeightBasedCapacity, LimitedStack, WeightBasedCost>;
