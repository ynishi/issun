//! Action points plugin for turn-based games

mod resources;
mod systems;

pub use resources::ActionPoints;
pub use systems::{ActionAutoAdvanceSystem, ActionResetSystem};
