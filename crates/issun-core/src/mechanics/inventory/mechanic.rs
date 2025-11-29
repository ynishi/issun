//! Inventory mechanic implementation.
//!
//! This module provides the main `InventoryMechanic` type, which composes
//! capacity, stacking, and cost policies into a complete inventory system.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic, ParallelSafe};

use super::policies::{CapacityPolicy, CostPolicy, StackingPolicy};
use super::strategies::{AlwaysStack, FixedSlotCapacity, NoCost};
use super::types::*;

/// Generic inventory mechanic.
///
/// This type composes three orthogonal policy dimensions:
/// - **CapacityPolicy** (C): How to evaluate capacity constraints
/// - **StackingPolicy** (S): How to organize items in stacks
/// - **CostPolicy** (K): How to calculate holding costs
///
/// # Type Parameters
///
/// - `C`: Capacity evaluation policy (default: `FixedSlotCapacity`)
/// - `S`: Stacking behavior policy (default: `AlwaysStack`)
/// - `K`: Cost calculation policy (default: `NoCost`)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::inventory::prelude::*;
/// use issun_core::mechanics::Mechanic;
///
/// // RPG inventory with slot limits and stacking
/// type RPGInventory = InventoryMechanic<
///     FixedSlotCapacity,
///     AlwaysStack,
///     NoCost,
/// >;
/// ```
pub struct InventoryMechanic<
    C: CapacityPolicy = FixedSlotCapacity,
    S: StackingPolicy = AlwaysStack,
    K: CostPolicy = NoCost,
> {
    _marker: PhantomData<(C, S, K)>,
}

impl<C, S, K> Mechanic for InventoryMechanic<C, S, K>
where
    C: CapacityPolicy,
    S: StackingPolicy,
    K: CostPolicy,
{
    type Config = InventoryConfig;
    type State = InventoryState;
    type Input = InventoryInput;
    type Event = InventoryEvent;
    type Execution = ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // Process the operation
        match input.operation {
            InventoryOperation::Add {
                stack,
                weight_per_item,
            } => {
                // Check capacity
                match C::can_add(state, &stack, weight_per_item, config) {
                    Ok(()) => {
                        // Add to inventory using stacking policy
                        S::add_to_inventory(state, stack, weight_per_item);

                        // Emit success event
                        emitter.emit(InventoryEvent::ItemAdded { stack });
                    }
                    Err(reason) => {
                        // Emit rejection event
                        emitter.emit(InventoryEvent::OperationRejected {
                            operation: input.operation,
                            reason,
                        });

                        // Also emit specific capacity events
                        match reason {
                            RejectionReason::InsufficientSlots => {
                                if let Some(max_slots) = config.max_slots {
                                    emitter.emit(InventoryEvent::CapacityReached {
                                        occupied_slots: state.occupied_slots,
                                        max_slots,
                                    });
                                }
                            }
                            RejectionReason::WeightLimitExceeded => {
                                if let Some(max_weight) = config.max_weight {
                                    emitter.emit(InventoryEvent::WeightLimitReached {
                                        current_weight: state.total_weight,
                                        max_weight,
                                    });
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }

            InventoryOperation::Remove { stack } => {
                // Check if removal is possible
                match C::can_remove(state, &stack) {
                    Ok(()) => {
                        // Remove from inventory
                        S::remove_from_inventory(state, stack);

                        // Emit success event
                        emitter.emit(InventoryEvent::ItemRemoved { stack });
                    }
                    Err(reason) => {
                        // Emit rejection event
                        emitter.emit(InventoryEvent::OperationRejected {
                            operation: input.operation,
                            reason,
                        });
                    }
                }
            }

            InventoryOperation::Transfer { stack } => {
                // Transfer validation only (actual transfer handled by game logic)
                match C::can_remove(state, &stack) {
                    Ok(()) => {
                        // Valid transfer - game logic will handle the actual movement
                    }
                    Err(reason) => {
                        emitter.emit(InventoryEvent::OperationRejected {
                            operation: input.operation,
                            reason,
                        });
                    }
                }
            }
        }

        // Calculate and apply holding cost if time elapsed
        if input.elapsed_time > 0 {
            let cost = K::calculate_cost(state, config, input.elapsed_time);
            if cost > 0.0 {
                emitter.emit(InventoryEvent::HoldingCostApplied { cost });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::inventory::strategies::*;

    // Test event collector
    struct TestEmitter {
        events: Vec<InventoryEvent>,
    }

    impl TestEmitter {
        fn new() -> Self {
            Self { events: Vec::new() }
        }
    }

    impl EventEmitter<InventoryEvent> for TestEmitter {
        fn emit(&mut self, event: InventoryEvent) {
            self.events.push(event);
        }
    }

    type BasicInventory = InventoryMechanic<FixedSlotCapacity, AlwaysStack, NoCost>;

    #[test]
    fn test_add_item_success() {
        let config = InventoryConfig::default();
        let mut state = InventoryState::new();
        let mut emitter = TestEmitter::new();

        let input = InventoryInput {
            operation: InventoryOperation::Add {
                stack: ItemStack::new(1, 10),
                weight_per_item: 1.0,
            },
            elapsed_time: 0,
        };

        BasicInventory::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.stacks[0].quantity, 10);
        assert_eq!(emitter.events.len(), 1);
        assert!(matches!(
            emitter.events[0],
            InventoryEvent::ItemAdded { .. }
        ));
    }

    #[test]
    fn test_add_item_stacking() {
        let config = InventoryConfig::default();
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 5));
        state.occupied_slots = 1;
        let mut emitter = TestEmitter::new();

        let input = InventoryInput {
            operation: InventoryOperation::Add {
                stack: ItemStack::new(1, 3),
                weight_per_item: 1.0,
            },
            elapsed_time: 0,
        };

        BasicInventory::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.stacks[0].quantity, 8); // 5 + 3
    }

    #[test]
    fn test_add_item_capacity_exceeded() {
        let config = InventoryConfig {
            max_slots: Some(1),
            ..Default::default()
        };
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 10));
        state.occupied_slots = 1;
        let mut emitter = TestEmitter::new();

        let input = InventoryInput {
            operation: InventoryOperation::Add {
                stack: ItemStack::new(2, 5), // Different item
                weight_per_item: 1.0,
            },
            elapsed_time: 0,
        };

        BasicInventory::step(&config, &mut state, input, &mut emitter);

        // Should be rejected
        assert_eq!(state.stacks.len(), 1);
        assert!(matches!(
            emitter.events.iter().find(|e| matches!(e, InventoryEvent::OperationRejected { .. })),
            Some(_)
        ));
    }

    #[test]
    fn test_remove_item_success() {
        let config = InventoryConfig::default();
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 10));
        state.occupied_slots = 1;
        let mut emitter = TestEmitter::new();

        let input = InventoryInput {
            operation: InventoryOperation::Remove {
                stack: ItemStack::new(1, 5),
            },
            elapsed_time: 0,
        };

        BasicInventory::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.stacks.len(), 1);
        assert_eq!(state.stacks[0].quantity, 5); // 10 - 5
        assert!(matches!(
            emitter.events[0],
            InventoryEvent::ItemRemoved { .. }
        ));
    }

    #[test]
    fn test_remove_item_not_found() {
        let config = InventoryConfig::default();
        let mut state = InventoryState::new();
        let mut emitter = TestEmitter::new();

        let input = InventoryInput {
            operation: InventoryOperation::Remove {
                stack: ItemStack::new(99, 1),
            },
            elapsed_time: 0,
        };

        BasicInventory::step(&config, &mut state, input, &mut emitter);

        assert!(matches!(
            emitter.events[0],
            InventoryEvent::OperationRejected { .. }
        ));
    }

    #[test]
    fn test_holding_cost() {
        type CostlyInventory = InventoryMechanic<FixedSlotCapacity, AlwaysStack, SlotBasedCost>;

        let config = InventoryConfig {
            holding_cost_per_slot: 10.0,
            ..Default::default()
        };
        let mut state = InventoryState::new();
        state.stacks.push(ItemStack::new(1, 5));
        state.occupied_slots = 1;
        let mut emitter = TestEmitter::new();

        let input = InventoryInput {
            operation: InventoryOperation::Add {
                stack: ItemStack::new(2, 3),
                weight_per_item: 1.0,
            },
            elapsed_time: 5,
        };

        CostlyInventory::step(&config, &mut state, input, &mut emitter);

        // Should have cost event (2 slots * 10.0 * 5 time = 100.0)
        let cost_event = emitter
            .events
            .iter()
            .find(|e| matches!(e, InventoryEvent::HoldingCostApplied { .. }));
        assert!(cost_event.is_some());
    }

    #[test]
    fn test_weight_based_capacity() {
        type WeightInventory =
            InventoryMechanic<WeightBasedCapacity, AlwaysStack, NoCost>;

        let config = InventoryConfig {
            max_weight: Some(100.0),
            ..Default::default()
        };
        let mut state = InventoryState::new();
        state.total_weight = 90.0;
        let mut emitter = TestEmitter::new();

        // Try to add items that would exceed weight limit
        let input = InventoryInput {
            operation: InventoryOperation::Add {
                stack: ItemStack::new(1, 10),
                weight_per_item: 2.0, // 10 * 2.0 = 20.0, total would be 110.0
            },
            elapsed_time: 0,
        };

        WeightInventory::step(&config, &mut state, input, &mut emitter);

        // Should be rejected
        assert!(matches!(
            emitter.events.iter().find(|e| matches!(e, InventoryEvent::OperationRejected { .. })),
            Some(_)
        ));
    }
}
