//! Title scene data

use serde::{Deserialize, Serialize};
use crate::models::{GameContext, GameScene, scenes::CombatSceneData};
use issun::prelude::SceneTransition;
use issun::ui::InputEvent;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleSceneData {
    pub selected_index: usize,
}

impl TitleSceneData {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }

    pub fn handle_input(
        &mut self,
        ctx: &mut GameContext,
        input: InputEvent,
    ) -> SceneTransition<GameScene> {
        match input {
            InputEvent::Cancel => {
                SceneTransition::Quit
            }
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
                                SceneTransition::Switch(GameScene::Combat(CombatSceneData::from_room(room.clone())))
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
                    _ => SceneTransition::Stay
                }
            }
            _ => SceneTransition::Stay
        }
    }
}

impl Default for TitleSceneData {
    fn default() -> Self {
        Self::new()
    }
}
