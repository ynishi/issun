//! Game context - persistent data across scenes
//!
//! Keeps lightweight data for the scaffold (player + ping/pong log).

use super::entities::Player;
use serde::{Deserialize, Serialize};

/// Persistent game data (survives scene transitions)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameContext {
    pub player: Player,
    pub ping_pong_log: Vec<String>,
}

impl GameContext {
    pub fn new() -> Self {
        Self {
            player: Player::new("Hero"),
            ping_pong_log: Vec::new(),
        }
    }
}

impl Default for GameContext {
    fn default() -> Self {
        Self::new()
    }
}
