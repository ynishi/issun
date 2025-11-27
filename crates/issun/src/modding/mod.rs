//! MOD System for ISSUN
//!
//! Provides core abstractions for dynamic content loading.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────┐
//! │  issun (Core Engine)        │
//! │  ├── ModLoader trait        │  ← Interface
//! │  ├── PluginControl          │  ← Control API
//! │  └── ModSystemPlugin        │  ← Integration
//! └─────────────────────────────┘
//!          ▲         ▲
//!          │         │
//!   ┌──────┘         └──────┐
//!   │                       │
//! ┌─┴──────────────┐  ┌─────┴─────────────┐
//! │ issun-mod-rhai │  │ issun-mod-wasm    │  ← Backends
//! │ (RhaiLoader)   │  │ (WasmLoader)      │
//! └────────────────┘  └───────────────────┘
//! ```
//!
//! # Usage
//!
//! ```ignore
//! use issun::prelude::*;
//!
//! let game = GameBuilder::new()
//!     .with_plugin(ModSystemPlugin::new().with_loader(RhaiLoader::new()))?
//!     .build()
//!     .await?;
//! ```

pub mod control;
pub mod error;
pub mod event_system;
pub mod events;
pub mod loader;
pub mod plugin;

#[cfg(test)]
mod tests;

pub use control::{PluginAction, PluginControl};
pub use error::{ModError, ModResult};
pub use event_system::ModEventSystem;
pub use events::{
    DynamicEvent, ModLoadFailedEvent, ModLoadRequested, ModLoadedEvent, ModUnloadRequested,
    ModUnloadedEvent, PluginControlRequested, PluginDisabledEvent, PluginEnabledEvent,
    PluginHookTriggeredEvent, PluginParameterChangedEvent,
};
pub use loader::{ModBackend, ModHandle, ModLoader, ModMetadata};
pub use plugin::{ModLoaderState, ModSystemConfig, ModSystemPlugin};

// Backend loaders are NOT re-exported from issun core to avoid circular dependencies.
// Users should import them directly from their respective crates:
// - use issun_mod_rhai::RhaiLoader;
// - use issun_mod_wasm::WasmLoader;
