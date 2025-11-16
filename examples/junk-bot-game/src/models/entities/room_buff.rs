use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Room buff types that affect combat
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RoomBuff {
    /// Standard room - no special effects
    Normal,
    /// Narrow corridor - Take +30% damage, -1 enemy
    Narrow,
    /// Wide hall - +1 enemy, better positioning
    Wide,
    /// Contaminated zone - Lose 5 HP per turn, 2x rare loot chance
    Contaminated,
}

impl RoomBuff {
    /// Get description of the room buff
    pub fn description(&self) -> &str {
        match self {
            RoomBuff::Normal => "Standard room",
            RoomBuff::Narrow => "Narrow corridor - Take +30% damage, fewer enemies",
            RoomBuff::Wide => "Wide hall - More enemies, better positioning",
            RoomBuff::Contaminated => "Contaminated zone - Lose 5 HP per turn, better loot",
        }
    }

    /// Get damage multiplier for this room
    pub fn damage_multiplier(&self) -> f32 {
        match self {
            RoomBuff::Narrow => 1.3,
            _ => 1.0,
        }
    }

    /// Get enemy count modifier
    pub fn enemy_count_modifier(&self) -> i32 {
        match self {
            RoomBuff::Narrow => -1,
            RoomBuff::Wide => 1,
            _ => 0,
        }
    }

    /// Get loot drop rate multiplier
    pub fn loot_multiplier(&self) -> f32 {
        match self {
            RoomBuff::Contaminated => 2.0,
            _ => 1.0,
        }
    }

    /// Get per-turn damage to party
    pub fn per_turn_damage(&self) -> i32 {
        match self {
            RoomBuff::Contaminated => 5,
            _ => 0,
        }
    }

    /// Get icon for display
    pub fn icon(&self) -> &str {
        match self {
            RoomBuff::Normal => "⚪",
            RoomBuff::Narrow => "🔴",
            RoomBuff::Wide => "🟢",
            RoomBuff::Contaminated => "🟡",
        }
    }

    /// Get color for display
    pub fn color(&self) -> Color {
        match self {
            RoomBuff::Normal => Color::White,
            RoomBuff::Narrow => Color::Red,
            RoomBuff::Wide => Color::Green,
            RoomBuff::Contaminated => Color::Yellow,
        }
    }

    /// Get short name for display
    pub fn name(&self) -> &str {
        match self {
            RoomBuff::Normal => "Standard",
            RoomBuff::Narrow => "Narrow",
            RoomBuff::Wide => "Wide",
            RoomBuff::Contaminated => "Contaminated",
        }
    }
}
