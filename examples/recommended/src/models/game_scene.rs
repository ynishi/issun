//! Game scene enum - distinct game states
//!
//! Scenes represent different phases of the game (Title, Combat, etc.)
//! Scene data is discarded on transition

use super::game_context::GameContext; // Needed for Scene derive macro
use super::scenes::*;
use issun::Scene;
use serde::{Deserialize, Serialize}; // Scene derive macro

/// Game scene enum (< 10 scenes recommended for enum pattern)
#[derive(Debug, Clone, Serialize, Deserialize, Scene)]
#[scene(
    context = "GameContext",
    initial = "Title(TitleSceneData::new())",
    handler_params = "input: ::issun::ui::InputEvent"
)]
pub enum GameScene {
    Title(TitleSceneData),
    Ping(PingSceneData),
    Pong(PongSceneData),
}
