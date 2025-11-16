//! Game scene enum - distinct game states
//!
//! Scenes represent different phases of the game (Title, Combat, etc.)
//! Scene data is discarded on transition

use super::scenes::*;
use super::game_context::GameContext; // Needed for Scene derive macro
use serde::{Deserialize, Serialize};
use issun::Scene; // Scene derive macro

/// Game scene enum (< 10 scenes recommended for enum pattern)
#[derive(Debug, Clone, Serialize, Deserialize, Scene)]
#[scene(
    context = "GameContext",
    initial = "Title(TitleSceneData::new())",
    handler_params = "ctx: &mut GameContext, input: ::issun::ui::InputEvent"
)]
pub enum GameScene {
    Title(TitleSceneData),
    RoomSelection(RoomSelectionSceneData),
    Combat(CombatSceneData),
    DropCollection(DropCollectionSceneData),
    CardSelection(CardSelectionSceneData),
    Floor4Choice(Floor4ChoiceSceneData),
    Result(ResultSceneData),
}
