//! Territory management plugin for strategy games

mod definitions;
mod events;
mod hook;
mod plugin;
mod registry;
mod service;
mod state;
mod system;
mod types;

pub use definitions::TerritoryDefinitions;
pub use events::{
    TerritoryControlChangeRequested, TerritoryControlChangedEvent, TerritoryDevelopedEvent,
    TerritoryDevelopmentRequested, TerritoryEffectsUpdatedEvent,
};
pub use hook::{DefaultTerritoryHook, TerritoryHook};
pub use plugin::TerritoryPlugin;
pub use registry::TerritoryRegistry; // TODO: Deprecated, will be removed
pub use service::TerritoryService;
pub use state::TerritoryState;
pub use system::TerritorySystem;
pub use types::{
    ControlChanged, Developed, Territory, TerritoryEffects, TerritoryError, TerritoryId,
};
