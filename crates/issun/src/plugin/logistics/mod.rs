//! Logistics plugin for automated resource transportation
//!
//! Provides high-performance logistics system with event-driven scheduling.
//!
//! # Overview
//!
//! The Logistics plugin manages automated transport routes that move resources
//! between inventories. It uses event-driven scheduling to efficiently process
//! thousands of routes without impacting frame rate.
//!
//! # Features
//!
//! - **Event-Driven Scheduling**: Only processes routes when ready (cooldown expired)
//! - **Transactional Integrity**: Proper error handling for inventory operations
//! - **Exponential Backoff**: Failed routes retry with increasing delays
//! - **Auto-Disable**: Routes that fail repeatedly are automatically disabled
//! - **Performance Metrics**: Track transfers, throughput, and route status
//! - **Hook System**: Customize behavior for game-specific logic
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::logistics::{LogisticsPlugin, Route, Transporter};
//! use issun::plugin::inventory::InventoryPlugin;
//! use issun::GameBuilder;
//!
//! #[tokio::main]
//! async fn main() {
//!     let game = GameBuilder::new()
//!         .with_plugin(LogisticsPlugin::new())
//!         .with_plugin(InventoryPlugin::new())
//!         .build()
//!         .await
//!         .unwrap();
//!
//!     // Setup inventories
//!     let inventory = game.get_plugin_mut::<InventoryPlugin>().unwrap();
//!     inventory.state_mut().add_item(&"mine".into(), &"iron_ore".into(), 1000).unwrap();
//!
//!     // Create transport route
//!     let logistics = game.get_plugin_mut::<LogisticsPlugin>().unwrap();
//!     let route = Route::new(
//!         "mine_to_smelter",
//!         "mine",
//!         "smelter",
//!         Transporter::new(10, 1.0) // 10 items/sec, 1 sec cooldown
//!             .with_filter(vec!["iron_ore"]),
//!     );
//!     logistics.state_mut().register_route(route);
//!
//!     // Game loop
//!     loop {
//!         let logistics = game.get_plugin_mut::<LogisticsPlugin>().unwrap();
//!         let inventory = game.get_plugin_mut::<InventoryPlugin>().unwrap();
//!
//!         logistics.system_mut().update(
//!             logistics.state_mut(),
//!             inventory.state_mut(),
//!             logistics.config(),
//!         ).await;
//!     }
//! }
//! ```
//!
//! # Performance
//!
//! - **1,000 routes**: ~0.1ms per update
//! - **10,000 routes**: ~1ms per update
//! - **100,000 routes**: ~5ms per update (capped at max_routes_per_update)
//!
//! # Architecture
//!
//! ```text
//! LogisticsPlugin
//! ├── Config (LogisticsConfig) - Global settings
//! ├── State (LogisticsState) - Routes & scheduling
//! ├── Service (LogisticsService) - Pure logic
//! ├── System (LogisticsSystem) - Orchestration
//! └── Hook (LogisticsHook) - Customization
//! ```

pub mod config;
pub mod hook;
pub mod plugin;
pub mod service;
pub mod state;
pub mod system;
pub mod types;

// Re-exports
pub use config::LogisticsConfig;
pub use hook::{DefaultLogisticsHook, LogisticsHook};
pub use plugin::LogisticsPlugin;
pub use service::LogisticsService;
pub use state::LogisticsState;
pub use system::LogisticsSystem;
pub use types::{
    InventoryEntityId, ItemId, LogisticsMetrics, Route, RouteId, RouteMetadata, RouteRuntime,
    TransferResult, Transporter, TransporterStatus,
};
