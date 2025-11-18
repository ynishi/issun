//! Title scene data

use crate::models::{scenes::PingSceneData, GameContext, GameScene};
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleSceneData {
    pub selected_index: usize,
}

impl TitleSceneData {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }

    pub async fn handle_input(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        resources: &mut ResourceContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Cancel => SceneTransition::Quit,
            InputEvent::Up => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Down => {
                if self.selected_index < 1 {
                    self.selected_index += 1;
                }
                SceneTransition::Stay
            }
            InputEvent::Select => {
                match self.selected_index {
                    0 => {
                        let mut player_hp = 150;
                        if let Some(mut ctx) = resources.try_get_mut::<GameContext>() {
                            ctx.ping_pong_log.clear();
                            player_hp = ctx.player.hp;
                        }
                        SceneTransition::Switch(GameScene::Ping(PingSceneData::initial(player_hp)))
                    }
                    1 => {
                        // Quit
                        SceneTransition::Quit
                    }
                    _ => SceneTransition::Stay,
                }
            }
            _ => SceneTransition::Stay,
        }
    }
}

impl Default for TitleSceneData {
    fn default() -> Self {
        Self::new()
    }
}
