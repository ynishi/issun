//! Resources for macroeconomy plugin

use std::collections::HashMap;

/// Aggregated economic metrics from various plugins
///
/// Plugins write their metrics here, and macroeconomy plugin reads
/// and aggregates them into EconomicSnapshot.
#[derive(Debug, Clone, Default)]
pub struct EconomicMetrics {
    /// Market-related metrics
    pub market: MarketMetrics,

    /// Production-related metrics
    pub production: ProductionMetrics,

    /// Currency-related metrics
    pub currency: CurrencyMetrics,

    /// Resource-related metrics
    pub resources: ResourceMetrics,

    /// Current tick
    pub current_tick: u64,
}

/// Market metrics (from market plugin)
#[derive(Debug, Clone, Default)]
pub struct MarketMetrics {
    /// Total transaction volume
    pub transaction_volume: f64,

    /// Price changes per item (item_id -> price_change)
    pub price_changes: HashMap<String, f32>,
}

/// Production metrics (from generation plugin)
#[derive(Debug, Clone, Default)]
pub struct ProductionMetrics {
    /// Total production output
    pub total_output: f64,
}

/// Currency metrics (from economy + accounting plugins)
#[derive(Debug, Clone, Default)]
pub struct CurrencyMetrics {
    /// Total currency in circulation
    pub total_circulation: f64,
}

/// Resource metrics (from economy plugin)
#[derive(Debug, Clone, Default)]
pub struct ResourceMetrics {
    /// Resource availability (resource_id -> quantity)
    pub availability: HashMap<String, u64>,
}
