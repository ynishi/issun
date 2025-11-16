//! Game context - persistent data across scenes
//!
//! This data survives scene transitions and should be saved/loaded

use serde::{Deserialize, Serialize};
use super::entities::Player;

/// Persistent game data (survives scene transitions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameContext {
    pub player: Player,
    pub score: u32,
    pub floor: u32,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            player: Player::new("Hero"),
            score: 0,
            floor: 1,
        }
    }
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
}
