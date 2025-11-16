use crate::models::entities::BuffCard;
use serde::{Deserialize, Serialize};

/// Scene data for card selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardSelectionSceneData {
    /// Available cards to choose from (typically 3 cards)
    pub available_cards: Vec<BuffCard>,
    /// Current cursor position
    pub cursor: usize,
    /// Indices of selected cards (max 1 selection)
    pub selected_index: Option<usize>,
}

impl CardSelectionSceneData {
    /// Create new card selection scene with the given cards
    pub fn new(cards: Vec<BuffCard>) -> Self {
        Self {
            available_cards: cards,
            cursor: 0,
            selected_index: None,
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
        if self.cursor < self.available_cards.len().saturating_sub(1) {
            self.cursor += 1;
        }
    }

    /// Select the card at current cursor position
    pub fn select_current(&mut self) {
        self.selected_index = Some(self.cursor);
    }

    /// Get the selected card if any
    pub fn get_selected_card(&self) -> Option<BuffCard> {
        self.selected_index
            .and_then(|idx| self.available_cards.get(idx).cloned())
    }
}
