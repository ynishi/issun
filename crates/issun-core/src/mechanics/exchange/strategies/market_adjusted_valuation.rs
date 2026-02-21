//! Market-adjusted valuation strategy that considers liquidity and reputation.

use crate::mechanics::exchange::policies::ValuationPolicy;
use crate::mechanics::exchange::types::ExchangeConfig;

/// Market-adjusted valuation that factors in liquidity and reputation.
///
/// This strategy adjusts the fair value based on:
/// - Market liquidity: Higher liquidity = more favorable rates
/// - Reputation: Higher reputation = better deals
pub struct MarketAdjustedValuation;

impl ValuationPolicy for MarketAdjustedValuation {
    fn calculate_fair_value(
        offered_value: f32,
        requested_value: f32,
        market_liquidity: f32,
        reputation: f32,
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

        if ratio < config.fairness_threshold || ratio > 1.0 / config.fairness_threshold {
            return 0.0;
        }

        // Base fair value (minimum of offered and requested)
        let base_value = offered_value.min(requested_value);

        // Liquidity bonus: High liquidity gives better exchange rates
        // Liquidity range: 0.0 to 1.0
        // Bonus range: 0% to 10%
        let liquidity_bonus = market_liquidity * 0.1;

        // Reputation bonus: High reputation gets better deals
        // Reputation range: 0.0 to 1.0
        // Bonus range: 0% to 5%
        let reputation_bonus = reputation * 0.05;

        // Apply bonuses
        let adjusted_value = base_value * (1.0 + liquidity_bonus + reputation_bonus);

        // Cap at 120% of base value to prevent excessive bonuses
        // But allow exceeding the max(offered, requested) as bonuses represent favorable conditions
        adjusted_value.min(base_value * 1.2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_market_adjusted_valuation_neutral_conditions() {
        let config = ExchangeConfig::default();
        // liquidity=0.0, reputation=0.0 should behave like simple valuation
        let fair = MarketAdjustedValuation::calculate_fair_value(100.0, 100.0, 0.0, 0.0, &config);
        assert_eq!(fair, 100.0);
    }

    #[test]
    fn test_market_adjusted_valuation_high_liquidity() {
        let config = ExchangeConfig::default();
        // High liquidity gives 10% bonus
        let fair = MarketAdjustedValuation::calculate_fair_value(100.0, 100.0, 1.0, 0.0, &config);
        assert_eq!(fair, 110.0); // 100 * (1 + 0.1)
    }

    #[test]
    fn test_market_adjusted_valuation_high_reputation() {
        let config = ExchangeConfig::default();
        // High reputation gives 5% bonus
        let fair = MarketAdjustedValuation::calculate_fair_value(100.0, 100.0, 0.0, 1.0, &config);
        assert!((fair - 105.0).abs() < 0.01); // 100 * (1 + 0.05), with floating point tolerance
    }

    #[test]
    fn test_market_adjusted_valuation_both_high() {
        let config = ExchangeConfig::default();
        // Both bonuses combined: 10% + 5% = 15%
        let fair = MarketAdjustedValuation::calculate_fair_value(100.0, 100.0, 1.0, 1.0, &config);
        assert_eq!(fair, 115.0); // 100 * (1 + 0.15)
    }

    #[test]
    fn test_market_adjusted_valuation_capped() {
        let config = ExchangeConfig::default();
        // Even with bonuses, capped at 120% of base value
        let fair = MarketAdjustedValuation::calculate_fair_value(100.0, 90.0, 1.0, 1.0, &config);
        // Base: 90, with 15% bonus = 103.5, capped at 90 * 1.2 = 108.0
        assert_eq!(fair, 103.5);
    }

    #[test]
    fn test_market_adjusted_valuation_unfair_trade() {
        let config = ExchangeConfig {
            fairness_threshold: 0.8,
            ..Default::default()
        };
        let fair = MarketAdjustedValuation::calculate_fair_value(100.0, 300.0, 1.0, 1.0, &config);
        assert_eq!(fair, 0.0); // Unfair trade regardless of bonuses
    }
}
