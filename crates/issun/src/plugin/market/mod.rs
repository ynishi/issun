//! MarketPlugin - Dynamic market economy system
//!
//! Provides a dynamic market economy where all items have real-time prices
//! driven by supply and demand.
//!
//! # Features
//!
//! - **Supply/Demand Price Calculation**: Prices automatically adjust based on market forces
//! - **Market Events**: External shocks (DemandShock, SupplyShock, Rumors, Scarcity, Abundance)
//! - **Trend Detection**: Automatic detection of Rising, Falling, Stable, or Volatile markets
//! - **Hook System**: Extensible with game-specific customization
//! - **Price History**: Track price changes over time for analysis
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::market::*;
//!
//! // Create plugin
//! let market = MarketPlugin::new()
//!     .with_config(
//!         MarketConfig::default()
//!             .with_demand_elasticity(0.7)
//!             .with_supply_elasticity(0.6)
//!     )
//!     .register_item("water", 10.0)
//!     .register_item("ammo", 50.0)
//!     .register_item("medicine", 100.0);
//!
//! // In game loop
//! let system = MarketSystem::new(DefaultMarketHook);
//!
//! // Apply event
//! let rumor = MarketEvent::rumor(vec!["medicine".to_string()], 0.9, 0.8);
//! system.apply_event(rumor, &mut state, &config).await;
//!
//! // Update prices
//! let changes = system.update_prices(&mut state, &config).await;
//! ```

pub mod config;
pub mod events;
pub mod hook;
pub mod plugin;
pub mod service;
pub mod state;
pub mod system;
pub mod types;

pub use config::*;
pub use events::*;
pub use hook::*;
pub use plugin::*;
pub use service::*;
pub use state::*;
pub use system::*;
pub use types::*;
