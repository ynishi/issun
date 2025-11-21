use crate::models::entities::Room;
use crate::models::{scenes::TitleSceneData, GameScene};
use issun::prelude::{ResourceContext, SceneTransition, ServiceContext, SystemContext};
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

/// Scene data for room selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoomSelectionSceneData {
    /// Available rooms to choose from (typically 3 rooms)
    pub available_rooms: Vec<Room>,
    /// Current cursor position
    pub cursor: usize,
}

impl RoomSelectionSceneData {
    /// Create new room selection scene with the given rooms
    #[allow(dead_code)]
    pub fn new(rooms: Vec<Room>) -> Self {
        Self {
            available_rooms: rooms,
            cursor: 0,
        }
    }

    /// Move cursor up
    #[allow(dead_code)]
    pub fn cursor_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    /// Move cursor down
    #[allow(dead_code)]
    pub fn cursor_down(&mut self) {
        if self.cursor < self.available_rooms.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    /// Get the selected room
    #[allow(dead_code)]
    pub fn get_selected_room(&self) -> Option<Room> {
        self.available_rooms.get(self.cursor).cloned()
    }

    /// Room selection is no longer used in dungeon mode, redirect to title
    pub async fn handle_input(
        &mut self,
        _services: &ServiceContext,
        _systems: &mut SystemContext,
        _resources: &mut ResourceContext,
        _input: InputEvent,
    ) -> SceneTransition<GameScene> {
        SceneTransition::Switch(GameScene::Title(TitleSceneData::new()))
    }
}
