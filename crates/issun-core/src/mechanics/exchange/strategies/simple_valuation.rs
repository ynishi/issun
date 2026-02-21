//! Simple valuation strategy that directly compares offered and requested values.

use crate::mechanics::exchange::policies::ValuationPolicy;
use crate::mechanics::exchange::types::ExchangeConfig;

/// Simple valuation that returns the minimum of offered and requested values.
///
/// This strategy represents a basic fair trade where both parties get equal value.
/// If the offered value is less than requested, the fair value is the offered amount.
/// If the offered value is more than requested, the fair value is the requested amount.
pub struct SimpleValuation;

impl ValuationPolicy for SimpleValuation {
    fn calculate_fair_value(
        offered_value: f32,
        requested_value: f32,
        _market_liquidity: f32,
        _reputation: f32,
        config: &ExchangeConfig,
    ) -> f32 {
        // Check minimum threshold
        if offered_value < config.minimum_value_threshold {
            return 0.0;
        }

        // Check fairness threshold
        let ratio = if requested_value > 0.0 {
            offered_value / requested_value
        } else {
            0.0
        };

        // If ratio is too far from 1.0, trade is unfair
        if ratio < config.fairness_threshold || ratio > 1.0 / config.fairness_threshold {
            return 0.0;
        }

        // Return the minimum (conservative valuation)
        offered_value.min(requested_value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_valuation_equal_values() {
        let config = ExchangeConfig::default();
        let fair = SimpleValuation::calculate_fair_value(100.0, 100.0, 0.5, 0.5, &config);
        assert_eq!(fair, 100.0);
    }

    #[test]
    fn test_simple_valuation_offered_less() {
        let config = ExchangeConfig::default();
        let fair = SimpleValuation::calculate_fair_value(80.0, 100.0, 0.5, 0.5, &config);
        assert_eq!(fair, 80.0); // Returns offered value (minimum)
    }

    #[test]
    fn test_simple_valuation_offered_more() {
        let config = ExchangeConfig::default();
        let fair = SimpleValuation::calculate_fair_value(120.0, 100.0, 0.5, 0.5, &config);
        assert_eq!(fair, 100.0); // Returns requested value (minimum)
    }

    #[test]
    fn test_simple_valuation_below_threshold() {
        let config = ExchangeConfig {
            minimum_value_threshold: 50.0,
            ..Default::default()
        };
        let fair = SimpleValuation::calculate_fair_value(30.0, 30.0, 0.5, 0.5, &config);
        assert_eq!(fair, 0.0); // Below threshold
    }

    #[test]
    fn test_simple_valuation_unfair_ratio() {
        let config = ExchangeConfig {
            fairness_threshold: 0.8, // 0.8x to 1.25x is acceptable
            ..Default::default()
        };
        let fair = SimpleValuation::calculate_fair_value(100.0, 300.0, 0.5, 0.5, &config);
        assert_eq!(fair, 0.0); // Ratio 0.33 is below threshold 0.8
    }

    #[test]
    fn test_simple_valuation_fee() {
        let config = ExchangeConfig {
            transaction_fee_rate: 0.05, // 5% fee
            ..Default::default()
        };
        let fair = SimpleValuation::calculate_fair_value(100.0, 100.0, 0.5, 0.5, &config);
        let fee = SimpleValuation::calculate_fee(fair, &config);
        assert_eq!(fee, 5.0);
    }
}
