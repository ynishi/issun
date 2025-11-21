//! Action points plugin for turn-based games

mod events;
mod hook;
mod plugin;
mod resources;
mod systems;

pub use events::{ActionConsumedEvent, ActionsResetEvent};
pub use hook::{ActionHook, DefaultActionHook};
pub use plugin::{ActionConfig, ActionPlugin};
pub use resources::{ActionConsumed, ActionError, ActionPoints};
pub use systems::{ActionResetSystem, ActionSystem};
