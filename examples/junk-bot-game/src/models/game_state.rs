//! Game state - Combines GameContext with current GameScene
//!
//! This structure represents the complete game state using ISSUN's Scene system

use super::{GameContext, GameScene};
use issun::prelude::*;
use serde::{Deserialize, Serialize};

/// GameState - Combines GameContext (shared data) with current GameScene
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current scene with scene-specific data
    pub scene: GameScene,

    /// Shared context accessible from all scenes
    pub ctx: GameContext,

    /// Should quit flag
    pub should_quit: bool,
}

impl GameState {
    pub fn new() -> Self {
        use super::scenes::TitleSceneData;
        Self {
            scene: GameScene::Title(TitleSceneData::new()),
            ctx: GameContext::new(),
            should_quit: false,
        }
    }

    /// Check if player is alive
    pub fn is_player_alive(&self) -> bool {
        self.ctx.player.is_alive()
    }
}

impl Default for GameState {
    fn default() -> Self {
        Self::new()
    }
}
