//! Territory management plugin for strategy games

mod events;
mod hook;
mod plugin;
mod registry;
mod service;
mod system;
mod types;

pub use events::{
    TerritoryControlChangeRequested, TerritoryControlChangedEvent, TerritoryDevelopedEvent,
    TerritoryDevelopmentRequested, TerritoryEffectsUpdatedEvent,
};
pub use hook::{DefaultTerritoryHook, TerritoryHook};
pub use plugin::TerritoryPlugin;
pub use registry::TerritoryRegistry;
pub use service::TerritoryService;
pub use system::TerritorySystem;
pub use types::{
    ControlChanged, Developed, Territory, TerritoryEffects, TerritoryError, TerritoryId,
};
