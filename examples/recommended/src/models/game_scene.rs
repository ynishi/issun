//! Game scene enum - distinct game states
//!
//! Scenes represent different phases of the game (Title, Combat, etc.)
//! Scene data is discarded on transition

use issun::prelude::*;
use issun::Scene; // Import derive macro
use super::scenes::*;

/// Game scene enum (< 10 scenes recommended for enum pattern)
#[derive(Scene)]
pub enum GameScene {
    Title(TitleSceneData),
    Combat(CombatSceneData),
    Result(ResultSceneData),
}
