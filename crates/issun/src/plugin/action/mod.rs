//! Action points plugin for turn-based games

mod plugin;
mod resources;
mod systems;

pub use plugin::{ActionConfig, ActionPlugin};
pub use resources::ActionPoints;
pub use systems::{ActionAutoAdvanceSystem, ActionResetSystem};
