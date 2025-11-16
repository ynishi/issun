//! Result scene data

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultSceneData {
    pub victory: bool,
    pub final_score: u32,
    pub message: String,
}

impl ResultSceneData {
    pub fn new(victory: bool, score: u32) -> Self {
        let message = if victory {
            format!("Victory! Final score: {}", score)
        } else {
            format!("Game Over. Final score: {}", score)
        };

        Self {
            victory,
            final_score: score,
            message,
        }
    }
}
