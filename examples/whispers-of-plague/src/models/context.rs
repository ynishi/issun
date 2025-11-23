use super::resources::GameMode;
use serde::{Deserialize, Serialize};

/// Game-wide context resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlagueGameContext {
    pub turn: u32,
    pub max_turns: u32,
    pub mode: GameMode,
}

impl PlagueGameContext {
    pub fn new() -> Self {
        Self {
            turn: 0,
            max_turns: 10,
            mode: GameMode::Plague,
        }
    }
}

impl Default for PlagueGameContext {
    fn default() -> Self {
        Self::new()
    }
}
