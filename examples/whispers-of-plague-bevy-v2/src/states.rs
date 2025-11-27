use bevy::prelude::*;

/// Game scene state
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum GameScene {
    #[default]
    Title,
    Game,
    Result,
}
