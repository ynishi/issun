//! Game scene enum - distinct game states
//!
//! Scenes represent different phases of the game (Title, Combat, etc.)
//! Scene data is discarded on transition

use issun::prelude::*;
use issun::Scene; // Import derive macro
use super::scenes::*;
use serde::{Deserialize, Serialize};

/// Game scene enum (< 10 scenes recommended for enum pattern)
#[derive(Debug, Clone, Serialize, Deserialize, Scene)]
pub enum GameScene {
    Title(TitleSceneData),
    RoomSelection(RoomSelectionSceneData),
    Combat(CombatSceneData),
    DropCollection(DropCollectionSceneData),
    CardSelection(CardSelectionSceneData),
    Floor4Choice(Floor4ChoiceSceneData),
    Result(ResultSceneData),
}
