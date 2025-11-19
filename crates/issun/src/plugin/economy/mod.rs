//! Economy management plugin for settlement-based games
//!
//! This plugin provides:
//! - `Currency`: Safe currency type with saturation arithmetic
//! - `BudgetLedger`: Resource tracking multiple financial pools
//! - `PolicyDeck`: Resource for active policies
//! - `EconomyService`: Service for pure financial calculations
//! - `EconomySystem`: System for periodic settlement orchestration
//! - `BuiltInEconomyPlugin`: Plugin for registering economy components
//!
//! # Architecture
//!
//! The economy plugin follows a settlement-based design:
//! - **Event-driven**: Reacts to `DayPassedEvent` from the Time plugin
//! - **Configurable**: Settlement period and dividend rates can be customized
//! - **Composable**: Service/System separation allows flexible reuse
//!
//! # Usage Example
//!
//! ```ignore
//! use issun::builder::GameBuilder;
//! use issun::plugin::economy::{BuiltInEconomyPlugin, EconomyConfig, Currency};
//! use issun::plugin::time::BuiltInTimePlugin;
//! use issun::context::ResourceContext;
//!
//! // Register the plugins
//! let game = GameBuilder::new()
//!     .with_plugin(BuiltInTimePlugin::default())?
//!     .with_plugin(BuiltInEconomyPlugin::new(EconomyConfig {
//!         settlement_period_days: 7,
//!         dividend_base: 200,
//!         dividend_rate: 0.04,
//!     }))?
//!     .build()
//!     .await?;
//!
//! // In your game loop
//! async fn game_loop(resources: &mut ResourceContext) {
//!     // Access budget ledger
//!     let mut ledger = resources.get_mut::<BudgetLedger>().await.unwrap();
//!     ledger.cash = ledger.cash.saturating_add(Currency::new(500));
//!     drop(ledger);
//!
//!     // Economy system will automatically run settlements when day passes
//! }
//! ```
//!
//! # Extending the Plugin
//!
//! To add custom settlement logic:
//!
//! 1. Create a custom system that listens to `DayPassedEvent`
//! 2. Use `EconomyService` for calculations
//! 3. Modify `BudgetLedger` through `ResourceContext`

mod config;
mod plugin;
mod resources;
mod service;
mod settlement;
mod system;
mod types;

pub use config::EconomyConfig;
pub use plugin::BuiltInEconomyPlugin;
pub use resources::{BudgetChannel, BudgetLedger, PolicyDeck};
pub use service::EconomyService;
pub use settlement::{DefaultSettlementSystem, SettlementSystem};
pub use system::EconomySystem;
pub use types::Currency;
