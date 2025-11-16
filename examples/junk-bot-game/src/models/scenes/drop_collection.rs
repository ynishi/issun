//! Drop collection scene data

use crate::models::entities::{LootItem, generate_random_cards};
use crate::models::{GameContext, GameScene, scenes::CardSelectionSceneData};
use issun::prelude::SceneTransition;
use issun::ui::InputEvent;
use serde::{Deserialize, Serialize};

/// Scene data for collecting dropped items after combat
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DropCollectionSceneData {
    /// Items dropped by defeated enemies
    pub drops: Vec<LootItem>,
    /// Index of currently selected item
    pub selected_index: usize,
}

impl DropCollectionSceneData {
    pub fn new(drops: Vec<LootItem>) -> Self {
        Self {
            drops,
            selected_index: 0,
        }
    }

    /// Check if there are any drops to collect
    pub fn has_drops(&self) -> bool {
        !self.drops.is_empty()
    }

    /// Move selection up
    pub fn move_up(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    /// Move selection down
    pub fn move_down(&mut self) {
        if self.selected_index < self.drops.len().saturating_sub(1) {
            self.selected_index += 1;
        }
    }

    /// Get currently selected item
    pub fn selected_item(&self) -> Option<&LootItem> {
        self.drops.get(self.selected_index)
    }

    /// Take the selected item (removes it from drops)
    pub fn take_selected(&mut self) -> Option<LootItem> {
        if self.selected_index < self.drops.len() {
            let item = self.drops.remove(self.selected_index);
            // Adjust selection if needed
            if self.selected_index >= self.drops.len() && !self.drops.is_empty() {
                self.selected_index = self.drops.len() - 1;
            }
            Some(item)
        } else {
            None
        }
    }

    pub fn handle_input(
        mut self,
        ctx: &mut GameContext,
        input: InputEvent,
    ) -> (GameScene, SceneTransition) {
        match input {
            InputEvent::Up => {
                self.move_up();
                (GameScene::DropCollection(self), SceneTransition::Stay)
            }
            InputEvent::Down => {
                self.move_down();
                (GameScene::DropCollection(self), SceneTransition::Stay)
            }
            InputEvent::Select => {
                // Take selected item
                if let Some(item) = self.take_selected() {
                    ctx.apply_loot_item(&item);
                }

                // If no more items, transition to card selection
                if !self.has_drops() {
                    let cards = generate_random_cards(3);
                    (GameScene::CardSelection(CardSelectionSceneData::new(cards)), SceneTransition::Stay)
                } else {
                    (GameScene::DropCollection(self), SceneTransition::Stay)
                }
            }
            InputEvent::Char(' ') => {
                // Take all items
                while let Some(item) = self.take_selected() {
                    ctx.apply_loot_item(&item);
                }
                // Transition to card selection after taking all
                let cards = generate_random_cards(3);
                (GameScene::CardSelection(CardSelectionSceneData::new(cards)), SceneTransition::Stay)
            }
            InputEvent::Cancel => {
                // Skip all items, transition to card selection
                let cards = generate_random_cards(3);
                (GameScene::CardSelection(CardSelectionSceneData::new(cards)), SceneTransition::Stay)
            }
            _ => (GameScene::DropCollection(self), SceneTransition::Stay)
        }
    }
}
