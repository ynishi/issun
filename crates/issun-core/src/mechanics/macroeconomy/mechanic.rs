//! Core mechanic implementation for macroeconomy.

use super::policies::EconomicPolicy;
use super::types::{
    EconomicEvent, EconomicIndicators, EconomicParameters, EconomicSnapshot, SentimentDirection,
    ShockType,
};
use crate::mechanics::{EventEmitter, Mechanic, Transactional};
use std::marker::PhantomData;

/// Macroeconomy mechanic with policy-based design
///
/// Generic over `P: EconomicPolicy` to support different economic models.
pub struct MacroeconomyMechanic<P: EconomicPolicy> {
    _marker: PhantomData<P>,
}

impl<P: EconomicPolicy> Mechanic for MacroeconomyMechanic<P> {
    type Config = EconomicParameters;
    type State = EconomicIndicators;
    type Input = EconomicSnapshot;
    type Event = EconomicEvent;
    type Execution = Transactional;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // Store old values for event emission
        let old_inflation = state.inflation_rate;
        let old_sentiment = state.market_sentiment;
        let old_cycle_phase = state.cycle_phase;

        // 1. Calculate new inflation rate
        let new_inflation = P::calculate_inflation(config, state, &input);
        state.inflation_rate = new_inflation;

        // Emit inflation change event if significant
        if (new_inflation - old_inflation).abs() > 0.001 {
            emitter.emit(EconomicEvent::InflationChanged {
                old_rate: old_inflation,
                new_rate: new_inflation,
                delta: new_inflation - old_inflation,
            });
        }

        // 2. Calculate market sentiment
        let new_sentiment = P::calculate_sentiment(config, state, &input);
        state.market_sentiment = new_sentiment;

        // Emit sentiment shift event if significant
        if (new_sentiment - old_sentiment).abs() > 0.1 {
            let direction = if new_sentiment > old_sentiment {
                SentimentDirection::MoreBullish
            } else if new_sentiment < old_sentiment {
                SentimentDirection::MoreBearish
            } else {
                SentimentDirection::Neutral
            };

            emitter.emit(EconomicEvent::SentimentShifted {
                old_sentiment,
                new_sentiment,
                direction,
            });
        }

        // 3. Calculate volatility
        let new_volatility = P::calculate_volatility(config, state, &input);
        state.volatility = new_volatility;

        // 4. Detect cycle phase
        let new_cycle_phase = P::detect_cycle_phase(config, state, &input);

        // Emit cycle phase change event
        if new_cycle_phase != old_cycle_phase {
            emitter.emit(EconomicEvent::CyclePhaseChanged {
                from: old_cycle_phase,
                to: new_cycle_phase,
            });
        }

        state.cycle_phase = new_cycle_phase;

        // 5. Calculate resource scarcity
        let scarcity_indices = P::calculate_scarcity(config, &input);

        // Emit scarcity alerts for high scarcity resources
        for (resource_id, scarcity_index) in &scarcity_indices {
            if *scarcity_index > 0.8 {
                emitter.emit(EconomicEvent::ScarcityAlert {
                    resource: resource_id.clone(),
                    scarcity_index: *scarcity_index,
                });
            }
        }

        state.scarcity_indices = scarcity_indices;

        // 6. Update money supply (simple inflation-based growth)
        let supply_growth = config.currency_supply.growth_rate * state.inflation_rate;
        state.money_supply = (state.money_supply * (1.0 + supply_growth as f64))
            .min(config.currency_supply.max_supply);

        // 7. Update production index
        state.production_index = input.production_output as f32;

        // 8. Detect economic shocks
        Self::detect_shocks(state, &input, emitter);

        // 9. Update last update tick
        state.last_update = input.current_tick;
    }
}

impl<P: EconomicPolicy> MacroeconomyMechanic<P> {
    /// Detect economic shocks from sudden changes
    fn detect_shocks(
        state: &EconomicIndicators,
        input: &EconomicSnapshot,
        emitter: &mut impl EventEmitter<EconomicEvent>,
    ) {
        // Supply shock detection: High scarcity in multiple resources
        let high_scarcity_count = state
            .scarcity_indices
            .values()
            .filter(|&&s| s > 0.7)
            .count();
        if high_scarcity_count >= 3 {
            emitter.emit(EconomicEvent::Shock {
                shock_type: ShockType::SupplyShock,
                magnitude: high_scarcity_count as f32 / state.scarcity_indices.len() as f32,
            });
        }

        // Demand shock detection: Sudden transaction volume spike
        // (This is simplified; in real implementation we'd track historical volume)
        if input.transaction_volume > 50000.0 {
            emitter.emit(EconomicEvent::Shock {
                shock_type: ShockType::DemandShock,
                magnitude: (input.transaction_volume / 50000.0) as f32,
            });
        }

        // Monetary shock detection: Large inflation spike
        if state.inflation_rate.abs() > 0.1 {
            emitter.emit(EconomicEvent::Shock {
                shock_type: ShockType::MonetaryShock,
                magnitude: state.inflation_rate,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::macroeconomy::strategies::SimpleEconomicPolicy;
    use crate::mechanics::macroeconomy::CyclePhase;
    use std::collections::HashMap;

    type SimpleMacroeconomy = MacroeconomyMechanic<SimpleEconomicPolicy>;

    struct TestEmitter {
        events: Vec<EconomicEvent>,
    }

    impl EventEmitter<EconomicEvent> for TestEmitter {
        fn emit(&mut self, event: EconomicEvent) {
            self.events.push(event);
        }
    }

    #[test]
    fn test_mechanic_step_basic() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();
        let snapshot = EconomicSnapshot {
            transaction_volume: 5000.0,
            price_changes: vec![("bread".to_string(), 0.02), ("water".to_string(), 0.01)]
                .into_iter()
                .collect(),
            production_output: 1000.0,
            currency_circulation: 100_000.0,
            resource_availability: HashMap::new(),
            current_tick: 100,
        };

        let mut emitter = TestEmitter { events: vec![] };

        SimpleMacroeconomy::step(&config, &mut state, snapshot, &mut emitter);

        // State should be updated
        assert!(state.inflation_rate > 0.0);
        assert_ne!(state.market_sentiment, 0.5);
        assert_eq!(state.last_update, 100);
        assert_eq!(state.production_index, 1000.0);

        // Events should be emitted
        assert!(!emitter.events.is_empty());
    }

    #[test]
    fn test_inflation_change_event() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();
        state.inflation_rate = 0.01;

        let snapshot = EconomicSnapshot {
            transaction_volume: 5000.0,
            price_changes: vec![("item".to_string(), 0.05)].into_iter().collect(),
            production_output: 1000.0,
            currency_circulation: 100_000.0,
            resource_availability: HashMap::new(),
            current_tick: 100,
        };

        let mut emitter = TestEmitter { events: vec![] };

        SimpleMacroeconomy::step(&config, &mut state, snapshot, &mut emitter);

        // Should emit inflation changed event
        let inflation_events: Vec<_> = emitter
            .events
            .iter()
            .filter(|e| matches!(e, EconomicEvent::InflationChanged { .. }))
            .collect();

        assert!(!inflation_events.is_empty());
    }

    #[test]
    fn test_cycle_phase_transition() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();
        state.production_index = 1000.0;
        state.cycle_phase = CyclePhase::Expansion;

        // Simulate contraction
        let snapshot = EconomicSnapshot {
            transaction_volume: 5000.0,
            price_changes: HashMap::new(),
            production_output: 950.0, // -5% production
            currency_circulation: 100_000.0,
            resource_availability: HashMap::new(),
            current_tick: 100,
        };

        let mut emitter = TestEmitter { events: vec![] };

        SimpleMacroeconomy::step(&config, &mut state, snapshot, &mut emitter);

        // Should transition to contraction
        assert_eq!(state.cycle_phase, CyclePhase::Contraction);

        // Should emit cycle phase change event
        let cycle_events: Vec<_> = emitter
            .events
            .iter()
            .filter(|e| matches!(e, EconomicEvent::CyclePhaseChanged { .. }))
            .collect();

        assert!(!cycle_events.is_empty());
    }

    #[test]
    fn test_scarcity_alert() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();

        let snapshot = EconomicSnapshot {
            transaction_volume: 5000.0,
            price_changes: HashMap::new(),
            production_output: 1000.0,
            currency_circulation: 100_000.0,
            resource_availability: vec![
                ("rare_resource".to_string(), 50), // Very scarce
            ]
            .into_iter()
            .collect(),
            current_tick: 100,
        };

        let mut emitter = TestEmitter { events: vec![] };

        SimpleMacroeconomy::step(&config, &mut state, snapshot, &mut emitter);

        // Should emit scarcity alert
        let scarcity_events: Vec<_> = emitter
            .events
            .iter()
            .filter(|e| matches!(e, EconomicEvent::ScarcityAlert { .. }))
            .collect();

        assert!(!scarcity_events.is_empty());
    }

    #[test]
    fn test_money_supply_growth() {
        let config = EconomicParameters::default();
        let mut state = EconomicIndicators::default();
        state.money_supply = 100_000.0;
        state.inflation_rate = 0.02;

        let snapshot = EconomicSnapshot::default();
        let mut emitter = TestEmitter { events: vec![] };

        SimpleMacroeconomy::step(&config, &mut state, snapshot, &mut emitter);

        // Money supply should grow based on inflation
        assert!(state.money_supply > 100_000.0);
    }
}
