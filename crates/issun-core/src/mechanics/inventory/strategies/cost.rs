//! Cost policy strategies.
//!
//! Provides concrete implementations of the CostPolicy trait.

use crate::mechanics::inventory::policies::CostPolicy;
use crate::mechanics::inventory::types::*;

/// No holding cost strategy.
///
/// Items can be held indefinitely without any cost.
///
/// # Use Cases
///
/// - Most standard RPGs
/// - Games without inventory management mechanics
/// - Abstract inventories (quest items)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::NoCost;
/// use issun_core::mechanics::inventory::policies::CostPolicy;
/// use issun_core::mechanics::inventory::{InventoryConfig, InventoryState};
///
/// let config = InventoryConfig::default();
/// let state = InventoryState::new();
///
/// let cost = NoCost::calculate_cost(&state, &config, 1000);
/// assert_eq!(cost, 0.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoCost;

impl CostPolicy for NoCost {
    fn calculate_cost(
        _state: &InventoryState,
        _config: &InventoryConfig,
        _elapsed_time: u32,
    ) -> f32 {
        0.0 // No cost
    }
}

/// Slot-based holding cost strategy.
///
/// Cost is proportional to the number of occupied slots.
/// Useful for warehouse/storage mechanics.
///
/// # Formula
///
/// ```text
/// cost = occupied_slots * cost_per_slot * elapsed_time
/// ```
///
/// # Use Cases
///
/// - Warehouse management games
/// - Storage rental systems
/// - Inventory management with fees
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::SlotBasedCost;
/// use issun_core::mechanics::inventory::policies::CostPolicy;
/// use issun_core::mechanics::inventory::{InventoryConfig, InventoryState, ItemStack};
///
/// let config = InventoryConfig {
///     holding_cost_per_slot: 10.0,
///     ..Default::default()
/// };
///
/// let mut state = InventoryState::new();
/// state.stacks.push(ItemStack::new(1, 5));
/// state.stacks.push(ItemStack::new(2, 3));
/// state.occupied_slots = 2;
///
/// // 2 slots * 10.0 cost/slot * 3 time units = 60.0
/// let cost = SlotBasedCost::calculate_cost(&state, &config, 3);
/// assert_eq!(cost, 60.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotBasedCost;

impl CostPolicy for SlotBasedCost {
    fn calculate_cost(state: &InventoryState, config: &InventoryConfig, elapsed_time: u32) -> f32 {
        state.occupied_slots as f32 * config.holding_cost_per_slot * elapsed_time as f32
    }
}

/// Weight-based holding cost strategy.
///
/// Cost is proportional to the total weight carried.
/// Useful for logistics/transport simulations.
///
/// # Formula
///
/// ```text
/// cost = total_weight * cost_per_weight * elapsed_time
/// ```
///
/// # Use Cases
///
/// - Logistics simulations
/// - Shipping/cargo management
/// - Weight-based storage fees
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::strategies::WeightBasedCost;
/// use issun_core::mechanics::inventory::policies::CostPolicy;
/// use issun_core::mechanics::inventory::{InventoryConfig, InventoryState};
///
/// let config = InventoryConfig {
///     holding_cost_per_weight: 0.5,
///     ..Default::default()
/// };
///
/// let mut state = InventoryState::new();
/// state.total_weight = 100.0;
///
/// // 100.0 weight * 0.5 cost/weight * 2 time units = 100.0
/// let cost = WeightBasedCost::calculate_cost(&state, &config, 2);
/// assert_eq!(cost, 100.0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightBasedCost;

impl CostPolicy for WeightBasedCost {
    fn calculate_cost(state: &InventoryState, config: &InventoryConfig, elapsed_time: u32) -> f32 {
        state.total_weight * config.holding_cost_per_weight * elapsed_time as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_config() -> InventoryConfig {
        InventoryConfig {
            max_slots: Some(20),
            max_weight: Some(100.0),
            max_stack_size: Some(99),
            holding_cost_per_slot: 5.0,
            holding_cost_per_weight: 0.1,
        }
    }

    // NoCost tests
    #[test]
    fn test_no_cost_always_zero() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.occupied_slots = 10;
        state.total_weight = 50.0;

        let cost = NoCost::calculate_cost(&state, &config, 100);
        assert_eq!(cost, 0.0);
    }

    // SlotBasedCost tests
    #[test]
    fn test_slot_based_cost_calculation() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.occupied_slots = 5;

        // 5 slots * 5.0 cost/slot * 2 time = 50.0
        let cost = SlotBasedCost::calculate_cost(&state, &config, 2);
        assert_eq!(cost, 50.0);
    }

    #[test]
    fn test_slot_based_cost_zero_slots() {
        let config = default_config();
        let state = InventoryState::new();

        let cost = SlotBasedCost::calculate_cost(&state, &config, 10);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_slot_based_cost_zero_time() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.occupied_slots = 10;

        let cost = SlotBasedCost::calculate_cost(&state, &config, 0);
        assert_eq!(cost, 0.0);
    }

    // WeightBasedCost tests
    #[test]
    fn test_weight_based_cost_calculation() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.total_weight = 100.0;

        // 100.0 weight * 0.1 cost/weight * 3 time = 30.0
        let cost = WeightBasedCost::calculate_cost(&state, &config, 3);
        assert_eq!(cost, 30.0);
    }

    #[test]
    fn test_weight_based_cost_zero_weight() {
        let config = default_config();
        let state = InventoryState::new();

        let cost = WeightBasedCost::calculate_cost(&state, &config, 10);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_weight_based_cost_zero_time() {
        let config = default_config();
        let mut state = InventoryState::new();
        state.total_weight = 50.0;

        let cost = WeightBasedCost::calculate_cost(&state, &config, 0);
        assert_eq!(cost, 0.0);
    }
}
