//! Policy management plugin for strategy and simulation games
//!
//! This plugin provides generic policy/card/buff management with:
//! - Generic effects system (no hard-coded game mechanics)
//! - Flexible aggregation strategies (Multiply, Add, Max, Min)
//! - Hook-based customization for game-specific logic
//! - Event-driven architecture for network replication
//! - Single-active OR multi-active policy modes
//!
//! # Example
//!
//! ```ignore
//! use issun::plugin::policy::{PolicyPlugin, PolicyRegistry, Policy, AggregationStrategy};
//! use issun::builder::GameBuilder;
//! use std::collections::HashMap;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create game with policy plugin
//! let game = GameBuilder::new()
//!     .add_plugin(PolicyPlugin::new())
//!     .build()
//!     .await?;
//!
//! // Add policies to the registry
//! let mut registry = game.resources.get_mut::<PolicyRegistry>().await.unwrap();
//!
//! let policy = Policy::new(
//!     "investor_friendly",
//!     "Investor-Friendly Policy",
//!     "Increases dividend demands but improves investment efficiency"
//! )
//! .add_effect("dividend_multiplier", 1.2)
//! .add_effect("investment_bonus", 1.3);
//!
//! registry.add(policy);
//!
//! // Activate a policy
//! registry.activate(&"investor_friendly".into()).unwrap();
//!
//! // Get aggregated effects
//! let income_mult = registry.get_effect("dividend_multiplier");
//! println!("Income multiplier: {}", income_mult);
//! # Ok(())
//! # }
//! ```

mod events;
mod hook;
mod plugin;
mod registry;
mod system;
mod types;

// Public exports
pub use events::*;
pub use hook::{DefaultPolicyHook, PolicyHook};
pub use plugin::PolicyPlugin;
pub use registry::{PolicyConfig, PolicyError, PolicyRegistry};
pub use types::{AggregationStrategy, Policy, PolicyId};
