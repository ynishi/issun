//! Pure logic service for market calculations
//!
//! This service provides stateless functions for price calculation, event application,
//! trend detection, and equilibrium drift.

use super::config::MarketConfig;
use super::types::{MarketData, MarketEvent, MarketEventType, MarketTrend};

/// Market service (stateless, pure functions)
///
/// All methods are pure functions with no side effects, making them easy to test.
#[derive(Debug, Clone, Copy, Default)]
pub struct MarketService;

impl MarketService {
    /// Calculate price based on supply and demand
    ///
    /// # Formula
    ///
    /// ```text
    /// price_factor = (demand / supply) ^ elasticity
    /// new_price = base_price * price_factor
    /// ```
    ///
    /// # Arguments
    ///
    /// * `data` - Current market data
    /// * `config` - Market configuration
    ///
    /// # Returns
    ///
    /// New calculated price (clamped to min/max bounds)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let data = MarketData { base_price: 100.0, demand: 0.8, supply: 0.4, .. };
    /// let config = MarketConfig::default();
    /// let price = MarketService::calculate_price(&data, &config);
    /// // High demand (0.8) + Low supply (0.4) = Higher price
    /// ```
    pub fn calculate_price(data: &MarketData, config: &MarketConfig) -> f32 {
        let demand = data.demand.clamp(0.01, 1.0);
        let supply = data.supply.clamp(0.01, 1.0);

        // Price increases when demand > supply
        // Price decreases when supply > demand
        let demand_supply_ratio = demand / supply;

        // Apply elasticity (how responsive price is to supply/demand)
        let avg_elasticity = (config.demand_elasticity + config.supply_elasticity) / 2.0;
        let price_factor = demand_supply_ratio.powf(avg_elasticity);

        // Calculate new price
        let new_price = data.base_price * price_factor;

        // Clamp to bounds
        let min_price = data.base_price * config.min_price_multiplier;
        let max_price = data.base_price * config.max_price_multiplier;

        new_price.clamp(min_price, max_price)
    }

    /// Apply market event to demand/supply
    ///
    /// # Arguments
    ///
    /// * `data` - Current market data
    /// * `event` - Market event
    /// * `config` - Market configuration
    ///
    /// # Returns
    ///
    /// Tuple of (new_demand, new_supply)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let event = MarketEvent::demand_shock(vec!["water".to_string()], 0.5);
    /// let (demand, supply) = MarketService::apply_event(&data, &event, &config);
    /// // demand increased by 0.5 * event_impact_coefficient
    /// ```
    pub fn apply_event(
        data: &MarketData,
        event: &MarketEvent,
        config: &MarketConfig,
    ) -> (f32, f32) {
        let mut demand = data.demand;
        let mut supply = data.supply;

        let impact = event.magnitude * config.event_impact_coefficient;

        match &event.event_type {
            MarketEventType::DemandShock => {
                // Positive magnitude = demand increase
                demand = (demand + impact).clamp(0.0, 1.0);
            }
            MarketEventType::SupplyShock => {
                // Positive magnitude = supply increase
                supply = (supply + impact).clamp(0.0, 1.0);
            }
            MarketEventType::Rumor {
                sentiment,
                credibility,
            } => {
                // Positive rumor = demand increase
                // Negative rumor = demand decrease
                let rumor_impact = sentiment * credibility * config.event_impact_coefficient;
                demand = (demand + rumor_impact).clamp(0.0, 1.0);
            }
            MarketEventType::Scarcity => {
                // Low supply
                supply = (supply - impact).clamp(0.0, 1.0);
            }
            MarketEventType::Abundance => {
                // High supply
                supply = (supply + impact).clamp(0.0, 1.0);
            }
            MarketEventType::Custom { .. } => {
                // Custom events handled by hook
            }
        }

        (demand, supply)
    }

    /// Detect market trend from price history
    ///
    /// # Algorithm
    ///
    /// Compares recent average price to older average price.
    /// If difference exceeds threshold, trend is detected.
    ///
    /// # Arguments
    ///
    /// * `data` - Market data with price history
    /// * `config` - Configuration with sensitivity threshold
    ///
    /// # Returns
    ///
    /// Detected market trend
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let data = MarketData {
    ///     price_history: vec![10.0, 11.0, 12.0, 13.0, 14.0],
    ///     ..
    /// };
    /// let trend = MarketService::detect_trend(&data, &config);
    /// // trend == MarketTrend::Rising
    /// ```
    pub fn detect_trend(data: &MarketData, config: &MarketConfig) -> MarketTrend {
        if data.price_history.len() < 4 {
            return MarketTrend::Stable;
        }

        let history: Vec<f32> = data.price_history.iter().copied().collect();
        let mid = history.len() / 2;

        // Compare first half average to second half average
        let old_avg: f32 = history[..mid].iter().sum::<f32>() / mid as f32;
        let new_avg: f32 = history[mid..].iter().sum::<f32>() / (history.len() - mid) as f32;

        let change_ratio = if old_avg > 0.0 {
            (new_avg - old_avg) / old_avg
        } else {
            0.0
        };

        // Calculate volatility (standard deviation)
        let mean = history.iter().sum::<f32>() / history.len() as f32;
        let variance: f32 = history
            .iter()
            .map(|&price| (price - mean).powi(2))
            .sum::<f32>()
            / history.len() as f32;
        let std_dev = variance.sqrt();
        let volatility = if mean > 0.0 { std_dev / mean } else { 0.0 };

        // Detect trend
        if volatility > config.trend_sensitivity * 3.0 {
            MarketTrend::Volatile
        } else if change_ratio > config.trend_sensitivity {
            MarketTrend::Rising
        } else if change_ratio < -config.trend_sensitivity {
            MarketTrend::Falling
        } else {
            MarketTrend::Stable
        }
    }

    /// Calculate natural demand/supply drift (returns to equilibrium)
    ///
    /// # Formula
    ///
    /// Demand and supply slowly drift toward 0.5 (equilibrium)
    ///
    /// # Arguments
    ///
    /// * `data` - Current market data
    /// * `drift_rate` - How fast to return to equilibrium (0.0-1.0)
    ///
    /// # Returns
    ///
    /// Tuple of (new_demand, new_supply)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let data = MarketData { demand: 0.8, supply: 0.3, .. };
    /// let (demand, supply) = MarketService::calculate_equilibrium_drift(&data, 0.1);
    /// // demand moves 10% closer to 0.5
    /// // supply moves 10% closer to 0.5
    /// ```
    pub fn calculate_equilibrium_drift(data: &MarketData, drift_rate: f32) -> (f32, f32) {
        let equilibrium = 0.5;

        let demand_drift = data.demand + (equilibrium - data.demand) * drift_rate;
        let supply_drift = data.supply + (equilibrium - data.supply) * drift_rate;

        (
            demand_drift.clamp(0.0, 1.0),
            supply_drift.clamp(0.0, 1.0),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    fn create_test_data(base_price: f32, demand: f32, supply: f32) -> MarketData {
        MarketData {
            item_id: "test".to_string(),
            base_price,
            current_price: base_price,
            demand,
            supply,
            price_history: VecDeque::new(),
            trend: MarketTrend::Stable,
            volatility: 0.1,
        }
    }

    fn create_config() -> MarketConfig {
        MarketConfig::default()
    }

    #[test]
    fn test_calculate_price_equilibrium() {
        let data = create_test_data(100.0, 0.5, 0.5);
        let config = create_config();

        let price = MarketService::calculate_price(&data, &config);

        // At equilibrium (demand == supply), price should be close to base
        assert!((price - 100.0).abs() < 1.0);
    }

    #[test]
    fn test_calculate_price_high_demand() {
        let data = create_test_data(100.0, 0.9, 0.3);
        let config = create_config();

        let price = MarketService::calculate_price(&data, &config);

        // High demand (0.9) and low supply (0.3) should increase price
        assert!(price > 100.0);
    }

    #[test]
    fn test_calculate_price_high_supply() {
        let data = create_test_data(100.0, 0.3, 0.9);
        let config = create_config();

        let price = MarketService::calculate_price(&data, &config);

        // Low demand (0.3) and high supply (0.9) should decrease price
        assert!(price < 100.0);
    }

    #[test]
    fn test_calculate_price_bounds() {
        let data = create_test_data(100.0, 1.0, 0.01);
        let config = create_config();

        let price = MarketService::calculate_price(&data, &config);

        // Price should be clamped to max_price_multiplier (10.0)
        assert!(price <= 100.0 * config.max_price_multiplier);
    }

    #[test]
    fn test_apply_event_demand_shock() {
        let data = create_test_data(100.0, 0.5, 0.5);
        let event = MarketEvent::demand_shock(vec!["test".to_string()], 0.4);
        let config = create_config();

        let (demand, supply) = MarketService::apply_event(&data, &event, &config);

        // Demand should increase by 0.4 * event_impact_coefficient (0.3) = 0.12
        // 0.5 + 0.12 = 0.62
        assert!((demand - 0.62).abs() < 0.01);
        assert_eq!(supply, 0.5); // Supply unchanged
    }

    #[test]
    fn test_apply_event_supply_shock() {
        let data = create_test_data(100.0, 0.5, 0.5);
        let event = MarketEvent::supply_shock(vec!["test".to_string()], 0.4);
        let config = create_config();

        let (demand, supply) = MarketService::apply_event(&data, &event, &config);

        assert_eq!(demand, 0.5); // Demand unchanged
        // Supply should increase by 0.4 * 0.3 = 0.12
        assert!((supply - 0.62).abs() < 0.01);
    }

    #[test]
    fn test_apply_event_rumor_positive() {
        let data = create_test_data(100.0, 0.5, 0.5);
        let event = MarketEvent::rumor(vec!["test".to_string()], 0.8, 0.9);
        let config = create_config();

        let (demand, _supply) = MarketService::apply_event(&data, &event, &config);

        // Positive rumor should increase demand
        // 0.8 * 0.9 * 0.3 = 0.216
        // 0.5 + 0.216 = 0.716
        assert!(demand > 0.5);
    }

    #[test]
    fn test_apply_event_rumor_negative() {
        let data = create_test_data(100.0, 0.5, 0.5);
        let event = MarketEvent::rumor(vec!["test".to_string()], -0.8, 0.9);
        let config = create_config();

        let (demand, _supply) = MarketService::apply_event(&data, &event, &config);

        // Negative rumor should decrease demand
        assert!(demand < 0.5);
    }

    #[test]
    fn test_apply_event_scarcity() {
        let data = create_test_data(100.0, 0.5, 0.5);
        let event = MarketEvent {
            event_type: MarketEventType::Scarcity,
            affected_items: vec!["test".to_string()],
            magnitude: 0.4,
            duration: 1,
        };
        let config = create_config();

        let (demand, supply) = MarketService::apply_event(&data, &event, &config);

        assert_eq!(demand, 0.5); // Demand unchanged
        // Scarcity decreases supply: 0.5 - 0.4*0.3 = 0.38
        assert!(supply < 0.5);
    }

    #[test]
    fn test_detect_trend_rising() {
        let mut data = create_test_data(100.0, 0.5, 0.5);
        data.price_history = vec![10.0, 11.0, 12.0, 13.0, 14.0, 15.0]
            .into_iter()
            .collect();

        let config = create_config();
        let trend = MarketService::detect_trend(&data, &config);

        assert_eq!(trend, MarketTrend::Rising);
    }

    #[test]
    fn test_detect_trend_falling() {
        let mut data = create_test_data(100.0, 0.5, 0.5);
        data.price_history = vec![15.0, 14.0, 13.0, 12.0, 11.0, 10.0]
            .into_iter()
            .collect();

        let config = create_config();
        let trend = MarketService::detect_trend(&data, &config);

        assert_eq!(trend, MarketTrend::Falling);
    }

    #[test]
    fn test_detect_trend_stable() {
        let mut data = create_test_data(100.0, 0.5, 0.5);
        data.price_history = vec![10.0, 10.1, 9.9, 10.0, 10.1, 9.9]
            .into_iter()
            .collect();

        let config = create_config();
        let trend = MarketService::detect_trend(&data, &config);

        assert_eq!(trend, MarketTrend::Stable);
    }

    #[test]
    fn test_detect_trend_volatile() {
        let mut data = create_test_data(100.0, 0.5, 0.5);
        data.price_history = vec![10.0, 20.0, 5.0, 25.0, 8.0, 30.0]
            .into_iter()
            .collect();

        let config = create_config();
        let trend = MarketService::detect_trend(&data, &config);

        assert_eq!(trend, MarketTrend::Volatile);
    }

    #[test]
    fn test_detect_trend_insufficient_data() {
        let mut data = create_test_data(100.0, 0.5, 0.5);
        data.price_history = vec![10.0, 11.0].into_iter().collect();

        let config = create_config();
        let trend = MarketService::detect_trend(&data, &config);

        // Insufficient data (< 4) should return Stable
        assert_eq!(trend, MarketTrend::Stable);
    }

    #[test]
    fn test_calculate_equilibrium_drift() {
        let data = create_test_data(100.0, 0.8, 0.3);
        let drift_rate = 0.1;

        let (demand, supply) = MarketService::calculate_equilibrium_drift(&data, drift_rate);

        // Demand drifts toward 0.5: 0.8 + (0.5 - 0.8) * 0.1 = 0.77
        assert!((demand - 0.77).abs() < 0.01);

        // Supply drifts toward 0.5: 0.3 + (0.5 - 0.3) * 0.1 = 0.32
        assert!((supply - 0.32).abs() < 0.01);
    }

    #[test]
    fn test_calculate_equilibrium_drift_at_equilibrium() {
        let data = create_test_data(100.0, 0.5, 0.5);
        let drift_rate = 0.1;

        let (demand, supply) = MarketService::calculate_equilibrium_drift(&data, drift_rate);

        // Already at equilibrium, should stay at 0.5
        assert_eq!(demand, 0.5);
        assert_eq!(supply, 0.5);
    }

    #[test]
    fn test_calculate_equilibrium_drift_clamping() {
        let data = create_test_data(100.0, 0.0, 1.0);
        let drift_rate = 10.0; // Extreme drift rate

        let (demand, supply) = MarketService::calculate_equilibrium_drift(&data, drift_rate);

        // Should clamp to valid range
        assert!(demand >= 0.0 && demand <= 1.0);
        assert!(supply >= 0.0 && supply <= 1.0);
    }
}
