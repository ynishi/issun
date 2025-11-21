//! Inventory system configuration (ReadOnly)

use crate::resources::Resource;
use serde::{Deserialize, Serialize};

/// Configuration for inventory system (ReadOnly)
///
/// This is a config loaded at startup and does not change during gameplay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryConfig {
    /// Default inventory capacity (0 = unlimited)
    pub default_capacity: usize,

    /// Whether to allow stacking of identical items
    pub allow_stacking: bool,

    /// Maximum stack size for stackable items (0 = unlimited)
    pub max_stack_size: u32,
}

impl Resource for InventoryConfig {}

impl Default for InventoryConfig {
    fn default() -> Self {
        Self {
            default_capacity: 0, // Unlimited by default
            allow_stacking: true,
            max_stack_size: 99,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = InventoryConfig::default();
        assert_eq!(config.default_capacity, 0);
        assert!(config.allow_stacking);
        assert_eq!(config.max_stack_size, 99);
    }

    #[test]
    fn test_custom_config() {
        let config = InventoryConfig {
            default_capacity: 20,
            allow_stacking: false,
            max_stack_size: 1,
        };
        assert_eq!(config.default_capacity, 20);
        assert!(!config.allow_stacking);
        assert_eq!(config.max_stack_size, 1);
    }
}
