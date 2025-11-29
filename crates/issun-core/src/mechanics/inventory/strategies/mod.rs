//! Strategy implementations for inventory policies.

pub mod capacity;
pub mod cost;
pub mod stacking;

// Re-export strategies
pub use capacity::{FixedSlotCapacity, UnlimitedCapacity, WeightBasedCapacity};
pub use cost::{NoCost, SlotBasedCost, WeightBasedCost};
pub use stacking::{AlwaysStack, LimitedStack, NeverStack};
