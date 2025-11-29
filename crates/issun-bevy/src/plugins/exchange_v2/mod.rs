//! Exchange Plugin V2 - Policy-Based Design
//!
//! This plugin integrates issun-core's policy-based exchange mechanic with Bevy's ECS.

pub mod plugin;
pub mod systems;
pub mod types;

pub use plugin::ExchangePluginV2;
pub use types::{
    MarketLiquidity, OfferedValue, RequestedValue, TradeCompleted, TradeRequested, Trader,
};
