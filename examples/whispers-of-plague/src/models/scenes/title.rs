use crate::models::{GameMode, GameScene, PlagueGameContext};
use issun::auto_pump;
use issun::prelude::*;
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleSceneData {
    pub selected_mode: Option<GameMode>,
}

impl TitleSceneData {
    pub fn new() -> Self {
        Self {
            selected_mode: None,
        }
    }

    #[auto_pump]
    pub async fn handle_input(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Char('1') => {
                self.selected_mode = Some(GameMode::Plague);
                SceneTransition::Stay
            }
            InputEvent::Char('2') => {
                self.selected_mode = Some(GameMode::Savior);
                SceneTransition::Stay
            }
            InputEvent::Select => {
                if let Some(mode) = self.selected_mode {
                    // Set game mode
                    if let Some(mut ctx) = resources.get_mut::<PlagueGameContext>().await {
                        ctx.mode = mode;
                        ctx.turn = 1;
                    }

                    // Initial infection is already seeded in main.rs via ContagionState
                    // No need to manually seed here

                    SceneTransition::Switch(GameScene::Game(super::GameSceneData::new()))
                } else {
                    SceneTransition::Stay
                }
            }
            InputEvent::Cancel | InputEvent::Char('q') => SceneTransition::Quit,
            _ => SceneTransition::Stay,
        }
    }
}

impl Default for TitleSceneData {
    fn default() -> Self {
        Self::new()
    }
}
