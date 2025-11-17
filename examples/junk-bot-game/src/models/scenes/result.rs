//! Result scene data

use crate::models::{scenes::TitleSceneData, GameContext, GameScene};
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

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
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
        ctx: &mut GameContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Select | InputEvent::Char(' ') => {
                // Return to title and reset context
                *ctx = GameContext::new();
                SceneTransition::Switch(GameScene::Title(TitleSceneData::new()))
            }
            _ => SceneTransition::Stay
        }
    }
}
