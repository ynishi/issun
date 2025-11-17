//! Rarity system extensions for junk-bot-game
//!
//! Re-exports issun::Rarity and adds game-specific UI display methods.

pub use issun::prelude::Rarity;
use ratatui::style::Color;

/// Game-specific extensions for Rarity
pub trait RarityExt {
    /// Get the display name of the rarity
    fn display_name(&self) -> &'static str;

    /// Get the color for UI display (ratatui::style::Color)
    fn ui_color(&self) -> Color;

    /// Get symbol/emoji for display
    fn ui_symbol(&self) -> &'static str;
}

impl RarityExt for Rarity {
    fn display_name(&self) -> &'static str {
        match self {
            Rarity::Common => "Common",
            Rarity::Uncommon => "Uncommon",
            Rarity::Rare => "Rare",
            Rarity::Epic => "Epic",
            Rarity::Legendary => "Legendary",
        }
    }

    fn ui_color(&self) -> Color {
        match self {
            Rarity::Common => Color::Gray,
            Rarity::Uncommon => Color::Green,
            Rarity::Rare => Color::Blue,
            Rarity::Epic => Color::Magenta,
            Rarity::Legendary => Color::Yellow,
        }
    }

    fn ui_symbol(&self) -> &'static str {
        match self {
            Rarity::Common => "○",
            Rarity::Uncommon => "◆",
            Rarity::Rare => "★",
            Rarity::Epic => "✦",
            Rarity::Legendary => "❖",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rarity_ext() {
        assert_eq!(Rarity::Common.display_name(), "Common");
        assert_eq!(Rarity::Legendary.ui_symbol(), "❖");
    }

    #[test]
    fn test_drop_weight() {
        // Test issun::Rarity's drop_weight method
        assert!(Rarity::Common.drop_weight() > Rarity::Legendary.drop_weight());
    }
}
