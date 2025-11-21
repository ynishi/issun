//! Territory management plugin for strategy games

mod events;
mod hook;
mod plugin;
mod service;
mod state;
mod system;
mod territories;
mod types;

pub use events::{
    TerritoryControlChangeRequested, TerritoryControlChangedEvent, TerritoryDevelopedEvent,
    TerritoryDevelopmentRequested, TerritoryEffectsUpdatedEvent,
};
pub use hook::{DefaultTerritoryHook, TerritoryHook};
pub use plugin::TerritoryPlugin;
pub use service::TerritoryService;
pub use state::TerritoryState;
pub use system::TerritorySystem;
pub use territories::Territories;
pub use types::{
    ControlChanged, Developed, Territory, TerritoryEffects, TerritoryError, TerritoryId,
};
