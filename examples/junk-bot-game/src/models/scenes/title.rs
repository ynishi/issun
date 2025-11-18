//! Title scene data

use crate::models::{scenes::CombatSceneData, GameContext, GameScene};
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
        let mut ctx = resources
            .get_mut::<GameContext>()
            .await
            .expect("GameContext resource not registered");
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
                        // Start game - initialize dungeon
                        ctx.start_dungeon();

                        // Get first room from dungeon
                        if let Some(dungeon) = ctx.get_dungeon() {
                            if let Some(room) = dungeon.get_current_room() {
                                SceneTransition::Switch(GameScene::Combat(
                                    CombatSceneData::from_room(room.clone()),
                                ))
                            } else {
                                SceneTransition::Stay
                            }
                        } else {
                            SceneTransition::Stay
                        }
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
