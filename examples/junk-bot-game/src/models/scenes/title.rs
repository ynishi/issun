//! Title scene data

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleSceneData {
    pub selected_index: usize,
}

impl TitleSceneData {
    pub fn new() -> Self {
        Self { selected_index: 0 }
    }
}

impl Default for TitleSceneData {
    fn default() -> Self {
        Self::new()
    }
}
