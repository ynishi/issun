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

    pub async fn handle_input(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        let mut ctx = resources
            .get_mut::<GameContext>()
            .await
            .expect("GameContext resource not registered");
        match input {
            InputEvent::Select | InputEvent::Char(' ') => {
                // Return to title and reset context
                *ctx = GameContext::new();
                drop(ctx);
                SceneTransition::Switch(GameScene::Title(TitleSceneData::new()))
            }
            _ => SceneTransition::Stay,
        }
    }
}
