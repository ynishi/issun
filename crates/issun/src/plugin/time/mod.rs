//! Time management plugin for turn-based and day-based games
//!
//! This plugin provides:
//! - `GameClock`: Resource tracking current day and action points
//! - `DayPassedEvent`: Event published when a new day begins
//! - `ActionConsumedEvent`: Event published when an action is consumed
//! - `BuiltInTimePlugin`: Plugin for registering time management components
//!
//! # Architecture
//!
//! The time plugin follows a minimalist design:
//! - **No automatic time progression**: Systems must explicitly advance time
//! - **Event-driven**: Systems publish events to notify others of time changes
//! - **Configurable**: Initial day and actions per day can be customized
//!
//! # Usage Example
//!
//! ```ignore
//! use issun::builder::GameBuilder;
//! use issun::plugin::time::{BuiltInTimePlugin, TimeConfig, GameClock, DayPassedEvent};
//! use issun::context::ResourceContext;
//! use issun::event::EventBus;
//!
//! // Register the plugin
//! let game = GameBuilder::new()
//!     .with_plugin(BuiltInTimePlugin::new(TimeConfig {
//!         initial_day: 1,
//!         actions_per_day: 5,
//!     }))?
//!     .build()
//!     .await?;
//!
//! // In a system, advance time
//! async fn advance_time_system(resources: &mut ResourceContext) {
//!     let config = resources.get::<TimeConfig>().await.unwrap();
//!     let mut clock = resources.get_mut::<GameClock>().await.unwrap();
//!     let new_day = clock.advance_day(config.actions_per_day);
//!     drop(clock); // Release lock
//!
//!     // Publish event to notify other systems
//!     let mut bus = resources.get_mut::<EventBus>().await.unwrap();
//!     bus.publish(DayPassedEvent { day: new_day });
//! }
//!
//! // In another system, react to day changes
//! async fn settlement_system(resources: &mut ResourceContext) {
//!     let mut bus = resources.get_mut::<EventBus>().await.unwrap();
//!     let reader = bus.reader::<DayPassedEvent>();
//!
//!     for event in reader.iter() {
//!         println!("Running settlement for day {}", event.day);
//!         // ... settlement logic
//!     }
//! }
//! ```

mod config;
mod events;
mod plugin;
mod resources;

pub use config::TimeConfig;
pub use events::{ActionConsumedEvent, DayPassedEvent};
pub use plugin::BuiltInTimePlugin;
pub use resources::GameClock;
