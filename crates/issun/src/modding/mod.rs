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

pub mod error;
pub mod loader;
pub mod control;
pub mod plugin;

#[cfg(test)]
mod tests;

pub use error::{ModError, ModResult};
pub use loader::{ModLoader, ModHandle, ModMetadata, ModBackend};
pub use control::{PluginControl, PluginAction};
pub use plugin::{ModSystemPlugin, ModSystemConfig, ModLoaderState};

// Backend loaders are NOT re-exported from issun core to avoid circular dependencies.
// Users should import them directly from their respective crates:
// - use issun_mod_rhai::RhaiLoader;
// - use issun_mod_wasm::WasmLoader;
