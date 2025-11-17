use crate::models::entities::Floor4Choice;
use crate::models::{
    scenes::{CombatSceneData, TitleSceneData},
    GameContext,
    GameScene,
};
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

/// Scene data for Floor 4 choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Floor4ChoiceSceneData {
    /// Current cursor position
    pub cursor: usize,
}

impl Floor4ChoiceSceneData {
    /// Create new Floor 4 choice scene
    pub fn new() -> Self {
        Self {
            cursor: 0,
        }
    }

    /// Move cursor up
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor down
    pub fn cursor_down(&mut self) {
        if self.cursor < 2 {
            self.cursor += 1;
        }
    }

    /// Get selected choice
    pub fn get_selected_choice(&self) -> Floor4Choice {
        match self.cursor {
            0 => Floor4Choice::Easy,
            1 => Floor4Choice::Normal,
            2 => Floor4Choice::Hard,
            _ => Floor4Choice::Normal,
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
            InputEvent::Up => {
                self.cursor_up();
                SceneTransition::Stay
            }
            InputEvent::Down => {
                self.cursor_down();
                SceneTransition::Stay
            }
            InputEvent::Select => {
                // Apply floor 4 choice
                let choice = self.get_selected_choice();
                if let Some(dungeon) = ctx.get_dungeon_mut() {
                    dungeon.set_floor4_choice(choice);
                    // Get the room and start combat
                    if let Some(room) = dungeon.get_current_room() {
                        return SceneTransition::Switch(GameScene::Combat(CombatSceneData::from_room(room.clone())));
                    }
                }
                SceneTransition::Stay
            }
            InputEvent::Cancel => {
                // Go back to title
                SceneTransition::Switch(GameScene::Title(TitleSceneData::new()))
            }
            _ => SceneTransition::Stay
        }
    }
}

impl Default for Floor4ChoiceSceneData {
    fn default() -> Self {
        Self::new()
    }
}
