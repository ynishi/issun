//! Simple economic policy implementation.

use crate::mechanics::macroeconomy::policies::EconomicPolicy;
use crate::mechanics::macroeconomy::types::{
    CyclePhase, EconomicIndicators, EconomicParameters, EconomicSnapshot, ResourceId,
};
use std::collections::HashMap;

/// Simple economic policy based on straightforward calculations
///
/// This policy implements basic economic formulas:
/// - Inflation: Simple CPI-like calculation from price changes
/// - Sentiment: Based on transaction volume and price trends
/// - Cycle: Compares production growth to thresholds
/// - Scarcity: Based on resource availability vs baseline
pub struct SimpleEconomicPolicy;

impl EconomicPolicy for SimpleEconomicPolicy {
    fn calculate_inflation(
        config: &EconomicParameters,
        state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> f32 {
        if snapshot.price_changes.is_empty() {
            // No price data, use exponential decay toward target
            let alpha = config.inflation_model.smoothing_alpha;
            return state.inflation_rate * (1.0 - alpha)
                + config.inflation_model.target_rate * alpha;
        }

        // Calculate average price change (CPI-like)
        let avg_price_change: f32 =
            snapshot.price_changes.values().sum::<f32>() / snapshot.price_changes.len() as f32;

        // Exponential smoothing with previous inflation rate
        let alpha = config.inflation_model.smoothing_alpha;
        state.inflation_rate * (1.0 - alpha) + avg_price_change * alpha
    }

    fn calculate_sentiment(
        _config: &EconomicParameters,
        _state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> f32 {
        // Sentiment = f(transaction_volume, price_trend)

        // Normalize transaction volume (assume 10000 as "high")
        let volume_factor = (snapshot.transaction_volume / 10000.0).clamp(0.0, 1.0) as f32;

        // Calculate average price trend
        let price_trend = if snapshot.price_changes.is_empty() {
            0.0
        } else {
            snapshot.price_changes.values().sum::<f32>() / snapshot.price_changes.len() as f32
        };

        // Combine factors (volume = activity, price trend = optimism/pessimism)
        // High volume + rising prices = bullish
        // High volume + falling prices = bearish
        // Low volume = neutral
        let sentiment = volume_factor * 0.5 + price_trend.clamp(-1.0, 1.0) * 0.5;

        sentiment.clamp(-1.0, 1.0)
    }

    fn detect_cycle_phase(
        config: &EconomicParameters,
        state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> CyclePhase {
        // Calculate production growth rate
        let growth_rate = if state.production_index > 0.0 {
            (snapshot.production_output as f32 - state.production_index) / state.production_index
        } else {
            0.0
        };

        // Detect phase based on growth thresholds
        if growth_rate > config.cycle_detection.expansion_threshold {
            // Strong growth -> Expansion or Peak
            match state.cycle_phase {
                CyclePhase::Expansion => {
                    // Check if we should transition to Peak
                    if growth_rate > config.cycle_detection.expansion_threshold * 2.0 {
                        CyclePhase::Peak
                    } else {
                        CyclePhase::Expansion
                    }
                }
                _ => CyclePhase::Expansion,
            }
        } else if growth_rate < config.cycle_detection.recession_threshold {
            // Negative growth -> Contraction or Trough
            match state.cycle_phase {
                CyclePhase::Contraction => {
                    // Check if we should transition to Trough
                    if growth_rate < config.cycle_detection.recession_threshold * 2.0 {
                        CyclePhase::Trough
                    } else {
                        CyclePhase::Contraction
                    }
                }
                _ => CyclePhase::Contraction,
            }
        } else {
            // Moderate growth/decline -> maintain current phase
            state.cycle_phase
        }
    }

    fn calculate_scarcity(
        _config: &EconomicParameters,
        snapshot: &EconomicSnapshot,
    ) -> HashMap<ResourceId, f32> {
        snapshot
            .resource_availability
            .iter()
            .map(|(id, qty)| {
                // Scarcity = 1 - (availability / baseline)
                // Baseline = 1000 units (arbitrary reference)
                let baseline = 1000.0;
                let scarcity = 1.0 - (*qty as f32 / baseline).min(1.0);
                (id.clone(), scarcity.clamp(0.0, 1.0))
            })
            .collect()
    }

    fn calculate_volatility(
        _config: &EconomicParameters,
        state: &EconomicIndicators,
        snapshot: &EconomicSnapshot,
    ) -> f32 {
        if snapshot.price_changes.is_empty() {
            return state.volatility * 0.9; // Decay if no data
        }

        // Calculate price change standard deviation
        let avg_change: f32 =
            snapshot.price_changes.values().sum::<f32>() / snapshot.price_changes.len() as f32;

        let variance: f32 = snapshot
            .price_changes
            .values()
            .map(|&change| (change - avg_change).powi(2))
            .sum::<f32>()
            / snapshot.price_changes.len() as f32;

        let std_dev = variance.sqrt();

        // Normalize to 0-1 range (assuming typical std_dev < 0.1)
        (std_dev / 0.1).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_snapshot(
        transaction_volume: f64,
        price_changes: Vec<(String, f32)>,
        production_output: f64,
    ) -> EconomicSnapshot {
        EconomicSnapshot {
            transaction_volume,
            price_changes: price_changes.into_iter().collect(),
            production_output,
            currency_circulation: 100_000.0,
            resource_availability: HashMap::new(),
            current_tick: 100,
        }
    }

    #[test]
    fn test_calculate_inflation_with_prices() {
        let config = EconomicParameters::default();
        let state = EconomicIndicators::default();
        let snapshot = create_snapshot(
            5000.0,
            vec![
                ("bread".to_string(), 0.02),
                ("water".to_string(), 0.03),
                ("metal".to_string(), 0.01),
            ],
            1000.0,
        );

        let inflation = SimpleEconomicPolicy::calculate_inflation(&config, &state, &snapshot);

        // Average price change = (0.02 + 0.03 + 0.01) / 3 = 0.02
        // With smoothing: 0.0 * 0.7 + 0.02 * 0.3 = 0.006
        assert!((inflation - 0.006).abs() < 0.001);
    }

    #[test]
    fn test_calculate_inflation_no_prices() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();
        state.inflation_rate = 0.05;
        let snapshot = create_snapshot(5000.0, vec![], 1000.0);

        let inflation = SimpleEconomicPolicy::calculate_inflation(&config, &state, &snapshot);

        // Decay toward target: 0.05 * 0.7 + 0.02 * 0.3 = 0.041
        assert!((inflation - 0.041).abs() < 0.001);
    }

    #[test]
    fn test_calculate_sentiment_bullish() {
        let config = EconomicParameters::default();
        let state = EconomicIndicators::default();
        let snapshot = create_snapshot(
            10000.0, // High volume
            vec![("item1".to_string(), 0.1), ("item2".to_string(), 0.2)], // Rising prices
            1000.0,
        );

        let sentiment = SimpleEconomicPolicy::calculate_sentiment(&config, &state, &snapshot);

        // volume_factor = 1.0, price_trend = 0.15
        // sentiment = 1.0 * 0.5 + 0.15 * 0.5 = 0.575
        assert!(sentiment > 0.5);
        assert!(sentiment <= 1.0);
    }

    #[test]
    fn test_calculate_sentiment_bearish() {
        let config = EconomicParameters::default();
        let state = EconomicIndicators::default();
        let snapshot = create_snapshot(
            8000.0,
            vec![("item1".to_string(), -0.1), ("item2".to_string(), -0.2)], // Falling prices
            1000.0,
        );

        let sentiment = SimpleEconomicPolicy::calculate_sentiment(&config, &state, &snapshot);

        // volume_factor = 0.8, price_trend = -0.15
        // sentiment = 0.8 * 0.5 + (-0.15) * 0.5 = 0.325
        assert!(sentiment < 0.5);
    }

    #[test]
    fn test_detect_cycle_phase_expansion() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();
        state.production_index = 1000.0;
        state.cycle_phase = CyclePhase::Expansion;

        let snapshot = create_snapshot(5000.0, vec![], 1050.0); // 5% growth

        let phase = SimpleEconomicPolicy::detect_cycle_phase(&config, &state, &snapshot);

        assert_eq!(phase, CyclePhase::Expansion);
    }

    #[test]
    fn test_detect_cycle_phase_contraction() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();
        state.production_index = 1000.0;
        state.cycle_phase = CyclePhase::Expansion;

        let snapshot = create_snapshot(5000.0, vec![], 980.0); // -2% growth

        let phase = SimpleEconomicPolicy::detect_cycle_phase(&config, &state, &snapshot);

        assert_eq!(phase, CyclePhase::Contraction);
    }

    #[test]
    fn test_calculate_scarcity() {
        let config = EconomicParameters::default();
        let mut snapshot = create_snapshot(5000.0, vec![], 1000.0);
        snapshot.resource_availability = vec![
            ("wood".to_string(), 500),   // 50% of baseline -> scarcity 0.5
            ("stone".to_string(), 1000),  // 100% of baseline -> scarcity 0.0
            ("rare_ore".to_string(), 100), // 10% of baseline -> scarcity 0.9
        ]
        .into_iter()
        .collect();

        let scarcity = SimpleEconomicPolicy::calculate_scarcity(&config, &snapshot);

        assert_eq!(scarcity.len(), 3);
        assert!((scarcity["wood"] - 0.5).abs() < 0.01);
        assert!((scarcity["stone"] - 0.0).abs() < 0.01);
        assert!((scarcity["rare_ore"] - 0.9).abs() < 0.01);
    }

    #[test]
    fn test_calculate_volatility() {
        let config = EconomicParameters::default();
        let state = EconomicIndicators::default();

        // High volatility scenario
        let snapshot = create_snapshot(
            5000.0,
            vec![
                ("item1".to_string(), 0.1),
                ("item2".to_string(), -0.1),
                ("item3".to_string(), 0.15),
                ("item4".to_string(), -0.15),
            ],
            1000.0,
        );

        let volatility = SimpleEconomicPolicy::calculate_volatility(&config, &state, &snapshot);

        assert!(volatility > 0.5);
    }
}
