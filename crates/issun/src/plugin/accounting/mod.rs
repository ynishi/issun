//! Accounting management plugin for budget and settlement
//!
//! This plugin provides:
//! - `Currency`: Safe currency type with saturation arithmetic
//! - `BudgetLedger`: Resource tracking multiple financial pools
//! - `AccountingService`: Service for pure financial calculations
//! - `AccountingSystem`: System for periodic settlement orchestration
//! - `AccountingPlugin`: Plugin for registering accounting components
//!
//! # Architecture
//!
//! The accounting plugin follows a settlement-based design:
//! - **Event-driven**: Reacts to settlement requests and time events
//! - **Configurable**: Settlement period and calculation logic customizable via hooks
//! - **Composable**: Service/System separation allows flexible reuse
//!
//! # Usage Example
//!
//! ```ignore
//! use issun::builder::GameBuilder;
//! use issun::plugin::accounting::{AccountingPlugin, AccountingConfig, Currency};
//! use issun::plugin::time::TimePlugin;
//! use issun::context::ResourceContext;
//!
//! // Register the plugins
//! let game = GameBuilder::new()
//!     .with_plugin(TimePlugin::default())?
//!     .with_plugin(AccountingPlugin::new()
//!         .with_hook(MyAccountingHook)
//!     )?
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
//!     // Accounting system will automatically run settlements when requested
//! }
//! ```

mod config;
mod events;
mod hook;
mod plugin;
mod resources;
mod service;
mod state;
mod system;
mod types;

pub use config::AccountingConfig;
pub use events::*;
pub use hook::{AccountingHook, DefaultAccountingHook};
pub use plugin::AccountingPlugin;
pub use resources::{BudgetChannel, BudgetLedger};
pub use service::AccountingService;
pub use state::AccountingState;
pub use system::AccountingSystem;
pub use types::Currency;
