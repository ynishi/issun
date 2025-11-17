use crate::models::entities::BuffCard;
use crate::models::{GameContext, GameScene, proceed_to_next_floor};
use issun::prelude::SceneTransition;
use issun::ui::InputEvent;
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

    pub fn handle_input(
        &mut self,
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
                // Select card and apply buff
                self.select_current();
                if let Some(card) = self.get_selected_card() {
                    ctx.apply_buff_card(card);
                }
                // Proceed to next floor after selecting a card
                proceed_to_next_floor(ctx)
            }
            InputEvent::Cancel => {
                // Skip card selection, proceed to next floor
                proceed_to_next_floor(ctx)
            }
            _ => SceneTransition::Stay
        }
    }
}
