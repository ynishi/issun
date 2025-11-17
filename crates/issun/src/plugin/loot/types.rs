//! Loot system types
//!
//! Generic types for implementing loot and drop systems in games.

use serde::{Deserialize, Serialize};

/// Standard rarity tiers for loot items
///
/// Provides a common 5-tier rarity system with drop weights.
/// Games can use this directly or implement custom rarity systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    /// Get drop weight for weighted random selection
    ///
    /// Higher values = more common drops
    pub fn drop_weight(&self) -> f32 {
        match self {
            Rarity::Common => 50.0,
            Rarity::Uncommon => 25.0,
            Rarity::Rare => 15.0,
            Rarity::Epic => 7.0,
            Rarity::Legendary => 3.0,
        }
    }

    /// Get display color (for TUI rendering)
    pub fn color(&self) -> &'static str {
        match self {
            Rarity::Common => "white",
            Rarity::Uncommon => "green",
            Rarity::Rare => "blue",
            Rarity::Epic => "purple",
            Rarity::Legendary => "yellow",
        }
    }

    /// Get all rarity tiers in order
    pub fn all() -> [Rarity; 5] {
        [
            Rarity::Common,
            Rarity::Uncommon,
            Rarity::Rare,
            Rarity::Epic,
            Rarity::Legendary,
        ]
    }
}

impl Default for Rarity {
    fn default() -> Self {
        Rarity::Common
    }
}

/// Configuration for drop rate calculations
#[derive(Debug, Clone)]
pub struct DropConfig {
    /// Base drop rate (0.0 - 1.0)
    pub base_rate: f32,
    /// Multiplier applied to base rate (e.g., from buffs)
    pub multiplier: f32,
}

impl DropConfig {
    /// Create new drop config
    pub fn new(base_rate: f32, multiplier: f32) -> Self {
        Self {
            base_rate,
            multiplier,
        }
    }

    /// Get final drop rate (capped at 1.0)
    pub fn final_rate(&self) -> f32 {
        (self.base_rate * self.multiplier).min(1.0)
    }
}

impl Default for DropConfig {
    fn default() -> Self {
        Self {
            base_rate: 0.3,
            multiplier: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_weights() {
        assert_eq!(Rarity::Common.drop_weight(), 50.0);
        assert_eq!(Rarity::Legendary.drop_weight(), 3.0);
    }

    #[test]
    fn test_drop_config() {
        let config = DropConfig::new(0.3, 2.0);
        assert_eq!(config.final_rate(), 0.6);

        // Test cap at 1.0
        let config_high = DropConfig::new(0.8, 2.0);
        assert_eq!(config_high.final_rate(), 1.0);
    }

    #[test]
    fn test_rarity_all() {
        let all = Rarity::all();
        assert_eq!(all.len(), 5);
        assert_eq!(all[0], Rarity::Common);
        assert_eq!(all[4], Rarity::Legendary);
    }
}
