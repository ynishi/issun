//! Core types for economy plugin

use serde::{Deserialize, Serialize};

// Re-export Currency from economy plugin
pub use crate::plugin::economy::Currency;

/// Budget channel for tracking expenses
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum BudgetChannel {
    Cash,
    Research,
    Operations,
    Reserve,
    Innovation,
    Security,
}
