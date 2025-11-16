use crate::models::entities::Floor4Choice;
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
}

impl Default for Floor4ChoiceSceneData {
    fn default() -> Self {
        Self::new()
    }
}
