//! Policy traits for macroeconomy mechanic.

use super::types::{
    CyclePhase, EconomicIndicators, EconomicParameters, EconomicSnapshot, ResourceId,
};
use std::collections::HashMap;

/// Policy for calculating economic indicators
///
/// This trait defines how macroeconomic indicators are derived from
/// the economic snapshot. Different implementations can model different
/// economic theories (Keynesian, Monetarist, Austrian, etc.).
pub trait EconomicPolicy {
    /// Calculate inflation rate from price changes
    ///
    /// # Arguments
    ///
    /// * `config` - Economic parameters
    /// * `state` - Current economic indicators (for smoothing)
    /// * `snapshot` - Current economic data
    ///
    /// # Returns
    ///
    /// New inflation rate (e.g., 0.02 = 2%)
    fn calculate_inflation(
        config: &EconomicParameters,
        state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> f32;

    /// Calculate market sentiment from transaction volume and price trends
    ///
    /// # Arguments
    ///
    /// * `config` - Economic parameters
    /// * `state` - Current economic indicators
    /// * `snapshot` - Current economic data
    ///
    /// # Returns
    ///
    /// Market sentiment (-1.0 to 1.0: bearish to bullish)
    fn calculate_sentiment(
        config: &EconomicParameters,
        state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> f32;

    /// Detect business cycle phase
    ///
    /// # Arguments
    ///
    /// * `config` - Economic parameters
    /// * `state` - Current economic indicators
    /// * `snapshot` - Current economic data
    ///
    /// # Returns
    ///
    /// Current cycle phase
    fn detect_cycle_phase(
        config: &EconomicParameters,
        state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> CyclePhase;

    /// Calculate resource scarcity indices
    ///
    /// # Arguments
    ///
    /// * `config` - Economic parameters
    /// * `snapshot` - Current economic data
    ///
    /// # Returns
    ///
    /// HashMap of resource scarcity indices (0.0 = abundant, 1.0 = scarce)
    fn calculate_scarcity(
        config: &EconomicParameters,
        snapshot: &EconomicSnapshot,
    ) -> HashMap<ResourceId, f32>;

    /// Calculate volatility index
    ///
    /// # Arguments
    ///
    /// * `config` - Economic parameters
    /// * `state` - Current economic indicators
    /// * `snapshot` - Current economic data
    ///
    /// # Returns
    ///
    /// Volatility index (0.0 to 1.0+)
    fn calculate_volatility(
        config: &EconomicParameters,
        state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> f32;
}
