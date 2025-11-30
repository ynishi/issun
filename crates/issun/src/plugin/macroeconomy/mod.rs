//! Macroeconomy Plugin - Macro-economic indicators and market context
//!
//! This plugin provides macro-economic context that influences micro-economic
//! mechanics like `exchange` and `market`. It aggregates data from various
//! plugins to calculate indicators like inflation, market sentiment, and
//! business cycle phases.
//!
//! # Features
//!
//! - **Economic Indicators**: Inflation, sentiment, volatility, cycle phase
//! - **Data Aggregation**: Collects metrics from market, accounting, generation plugins
//! - **Event System**: Emits events for indicator changes and economic shocks
//! - **Periodic Updates**: Configurable update frequency for efficiency
//!
//! # Integration
//!
//! ```ignore
//! use issun::builder::GameBuilder;
//! use issun::plugin::macroeconomy::MacroeconomyPlugin;
//!
//! let game = GameBuilder::new()
//!     .with_plugin(MacroeconomyPlugin::default())?
//!     .build()
//!     .await?;
//! ```

pub mod config;
pub mod events;
pub mod plugin;
pub mod resources;
pub mod service;
pub mod state;
pub mod system;

pub use config::MacroeconomyConfig;
pub use events::*;
pub use plugin::MacroeconomyPlugin;
pub use resources::EconomicMetrics;
pub use service::MacroeconomyService;
pub use state::MacroeconomyState;
pub use system::MacroeconomySystem;

// Re-export core types from issun-core
pub use issun_core::mechanics::macroeconomy::{
    CyclePhase, EconomicEvent, EconomicIndicators, EconomicParameters, EconomicSnapshot,
};
