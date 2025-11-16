//! Result scene data

use serde::{Deserialize, Serialize};
use crate::models::{GameContext, GameScene, scenes::TitleSceneData};
use issun::prelude::SceneTransition;
use issun::ui::InputEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSceneData {
    pub victory: bool,
    pub final_score: u32,
    pub message: String,
}

impl ResultSceneData {
    pub fn new(victory: bool, score: u32) -> Self {
        let message = if victory {
            format!("Victory! Final score: {}", score)
        } else {
            format!("Game Over. Final score: {}", score)
        };

        Self {
            victory,
            final_score: score,
            message,
        }
    }

    pub fn handle_input(
        self,
        ctx: &mut GameContext,
        input: InputEvent,
    ) -> (GameScene, SceneTransition) {
        match input {
            InputEvent::Select | InputEvent::Char(' ') => {
                // Return to title and reset context
                *ctx = GameContext::new();
                (GameScene::Title(TitleSceneData::new()), SceneTransition::Stay)
            }
            _ => (GameScene::Result(self), SceneTransition::Stay)
        }
    }
}
