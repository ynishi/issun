//! Type definitions for macroeconomy mechanic.

use std::collections::HashMap;

/// Unique identifier for resources
pub type ResourceId = String;

/// Unique identifier for items
pub type ItemId = String;

// ============================================================================
// Config: EconomicParameters
// ============================================================================

/// Configuration parameters for economic calculations
#[derive(Debug, Clone)]
pub struct EconomicParameters {
    /// Inflation calculation parameters
    pub inflation_model: InflationModelConfig,

    /// Market sentiment thresholds
    pub sentiment_thresholds: SentimentThresholds,

    /// Business cycle detection parameters
    pub cycle_detection: CycleDetectionConfig,

    /// Currency supply rules
    pub currency_supply: CurrencySupplyConfig,
}

impl Default for EconomicParameters {
    fn default() -> Self {
        Self {
            inflation_model: InflationModelConfig::default(),
            sentiment_thresholds: SentimentThresholds::default(),
            cycle_detection: CycleDetectionConfig::default(),
            currency_supply: CurrencySupplyConfig::default(),
        }
    }
}

/// Inflation model configuration
#[derive(Debug, Clone)]
pub struct InflationModelConfig {
    /// Target inflation rate (e.g., 0.02 = 2%)
    pub target_rate: f32,

    /// Smoothing factor for rolling calculation (0.0-1.0)
    pub smoothing_alpha: f32,

    /// Base period for price comparison (ticks)
    pub base_period: u32,
}

impl Default for InflationModelConfig {
    fn default() -> Self {
        Self {
            target_rate: 0.02,
            smoothing_alpha: 0.3,
            base_period: 100,
        }
    }
}

/// Market sentiment thresholds
#[derive(Debug, Clone)]
pub struct SentimentThresholds {
    /// Below this = Bearish (< 0.3)
    pub bearish_threshold: f32,

    /// Above this = Bullish (> 0.7)
    pub bullish_threshold: f32,

    /// Volatility threshold for "uncertain" (> 0.5)
    pub volatility_threshold: f32,
}

impl Default for SentimentThresholds {
    fn default() -> Self {
        Self {
            bearish_threshold: 0.3,
            bullish_threshold: 0.7,
            volatility_threshold: 0.5,
        }
    }
}

/// Business cycle detection configuration
#[derive(Debug, Clone)]
pub struct CycleDetectionConfig {
    /// Minimum duration to confirm cycle phase (ticks)
    pub min_phase_duration: u32,

    /// GDP growth threshold for expansion (> 0.03)
    pub expansion_threshold: f32,

    /// GDP contraction threshold for recession (< -0.01)
    pub recession_threshold: f32,
}

impl Default for CycleDetectionConfig {
    fn default() -> Self {
        Self {
            min_phase_duration: 50,
            expansion_threshold: 0.03,
            recession_threshold: -0.01,
        }
    }
}

/// Currency supply configuration
#[derive(Debug, Clone)]
pub struct CurrencySupplyConfig {
    /// Initial money supply
    pub initial_supply: f64,

    /// Maximum allowed supply
    pub max_supply: f64,

    /// Inflation-driven supply growth rate
    pub growth_rate: f32,
}

impl Default for CurrencySupplyConfig {
    fn default() -> Self {
        Self {
            initial_supply: 100_000.0,
            max_supply: 10_000_000.0,
            growth_rate: 0.02,
        }
    }
}

// ============================================================================
// State: EconomicIndicators
// ============================================================================

/// Economic indicators state (output of the mechanic)
#[derive(Debug, Clone)]
pub struct EconomicIndicators {
    /// Current inflation rate (e.g., 0.02 = 2%)
    pub inflation_rate: f32,

    /// Market sentiment (-1.0 to 1.0: bearish to bullish)
    pub market_sentiment: f32,

    /// Volatility index (0.0 to 1.0+)
    pub volatility: f32,

    /// Current business cycle phase
    pub cycle_phase: CyclePhase,

    /// Total money supply
    pub money_supply: f64,

    /// Aggregate production index (normalized)
    pub production_index: f32,

    /// Resource scarcity indices (per resource type)
    pub scarcity_indices: HashMap<ResourceId, f32>,

    /// Last update tick
    pub last_update: u64,
}

impl Default for EconomicIndicators {
    fn default() -> Self {
        Self {
            inflation_rate: 0.0,
            market_sentiment: 0.5,
            volatility: 0.1,
            cycle_phase: CyclePhase::Expansion,
            money_supply: 100_000.0,
            production_index: 1.0,
            scarcity_indices: HashMap::new(),
            last_update: 0,
        }
    }
}

/// Business cycle phase
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CyclePhase {
    /// Economic expansion phase
    Expansion,
    /// Peak of economic activity
    Peak,
    /// Economic contraction phase
    Contraction,
    /// Trough of economic activity
    Trough,
}

// ============================================================================
// Input: EconomicSnapshot
// ============================================================================

/// Snapshot of economic data from various sources (input to the mechanic)
#[derive(Debug, Clone)]
pub struct EconomicSnapshot {
    /// Aggregate transaction volume (from market/exchange)
    pub transaction_volume: f64,

    /// Average price changes (from market plugin)
    pub price_changes: HashMap<ItemId, f32>,

    /// Total production output (from generation plugin)
    pub production_output: f64,

    /// Total currency in circulation (from accounting/economy)
    pub currency_circulation: f64,

    /// Resource availability (from inventory/economy)
    pub resource_availability: HashMap<ResourceId, u64>,

    /// Current tick
    pub current_tick: u64,
}

impl Default for EconomicSnapshot {
    fn default() -> Self {
        Self {
            transaction_volume: 0.0,
            price_changes: HashMap::new(),
            production_output: 0.0,
            currency_circulation: 100_000.0,
            resource_availability: HashMap::new(),
            current_tick: 0,
        }
    }
}

// ============================================================================
// Event: EconomicEvent
// ============================================================================

/// Events emitted by the macroeconomy mechanic
#[derive(Debug, Clone, PartialEq)]
pub enum EconomicEvent {
    /// Inflation rate changed
    InflationChanged {
        old_rate: f32,
        new_rate: f32,
        delta: f32,
    },

    /// Market sentiment shifted
    SentimentShifted {
        old_sentiment: f32,
        new_sentiment: f32,
        direction: SentimentDirection,
    },

    /// Business cycle phase transition
    CyclePhaseChanged { from: CyclePhase, to: CyclePhase },

    /// Economic shock detected
    Shock {
        shock_type: ShockType,
        magnitude: f32,
    },

    /// Resource scarcity alert
    ScarcityAlert {
        resource: ResourceId,
        scarcity_index: f32,
    },
}

/// Direction of sentiment change
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SentimentDirection {
    /// Sentiment became more bearish
    MoreBearish,
    /// Sentiment became more bullish
    MoreBullish,
    /// No significant change
    Neutral,
}

/// Type of economic shock
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShockType {
    /// Supply shock (scarcity)
    SupplyShock,
    /// Demand shock (sudden demand change)
    DemandShock,
    /// Monetary shock (currency supply change)
    MonetaryShock,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_economic_parameters_default() {
        let params = EconomicParameters::default();
        assert_eq!(params.inflation_model.target_rate, 0.02);
        assert_eq!(params.sentiment_thresholds.bearish_threshold, 0.3);
        assert_eq!(params.cycle_detection.min_phase_duration, 50);
        assert_eq!(params.currency_supply.initial_supply, 100_000.0);
    }

    #[test]
    fn test_economic_indicators_default() {
        let indicators = EconomicIndicators::default();
        assert_eq!(indicators.inflation_rate, 0.0);
        assert_eq!(indicators.market_sentiment, 0.5);
        assert_eq!(indicators.cycle_phase, CyclePhase::Expansion);
        assert_eq!(indicators.last_update, 0);
    }

    #[test]
    fn test_economic_snapshot_default() {
        let snapshot = EconomicSnapshot::default();
        assert_eq!(snapshot.transaction_volume, 0.0);
        assert_eq!(snapshot.production_output, 0.0);
        assert_eq!(snapshot.current_tick, 0);
    }
}
