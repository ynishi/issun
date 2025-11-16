//! Drop collection scene data

use crate::models::entities::LootItem;
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
}
