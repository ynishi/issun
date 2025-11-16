//! Rarity system for items and loot

use serde::{Deserialize, Serialize};
use ratatui::style::Color;

/// Rarity levels for items, cards, and loot
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
    Legendary,
}

impl Rarity {
    /// Get the display name of the rarity
    pub fn name(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
        }
    }

    /// Get the color for UI display
    pub fn color(&self) -> Color {
        match self {
            Rarity::Common => Color::Gray,
            Rarity::Uncommon => Color::Green,
            Rarity::Rare => Color::Blue,
            Rarity::Epic => Color::Magenta,
            Rarity::Legendary => Color::Yellow,
        }
    }

    /// Get the drop weight (higher = more common)
    pub fn drop_weight(&self) -> f32 {
        match self {
            Rarity::Common => 50.0,
            Rarity::Uncommon => 30.0,
            Rarity::Rare => 15.0,
            Rarity::Epic => 4.0,
            Rarity::Legendary => 1.0,
        }
    }

    /// Get symbol/emoji for display
    pub fn symbol(&self) -> &'static str {
        match self {
            Rarity::Common => "○",
            Rarity::Uncommon => "◆",
            Rarity::Rare => "★",
            Rarity::Epic => "✦",
            Rarity::Legendary => "❖",
        }
    }
}

impl Default for Rarity {
    fn default() -> Self {
        Rarity::Common
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_ordering() {
        assert!(Rarity::Common < Rarity::Uncommon);
        assert!(Rarity::Rare < Rarity::Epic);
        assert!(Rarity::Epic < Rarity::Legendary);
    }

    #[test]
    fn test_drop_weight() {
        assert!(Rarity::Common.drop_weight() > Rarity::Legendary.drop_weight());
    }
}
