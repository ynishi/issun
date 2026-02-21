//! The core ExchangeMechanic implementation.

use std::marker::PhantomData;

use crate::mechanics::{EventEmitter, Mechanic};

use super::policies::{ExecutionPolicy, ValuationPolicy};
use super::strategies::{FairTradeExecution, SimpleValuation};
use super::types::{ExchangeConfig, ExchangeEvent, ExchangeInput, ExchangeState};

/// The core exchange mechanic that composes valuation and execution policies.
///
/// # Type Parameters
///
/// - `V`: Valuation policy (calculates fair value and fees)
/// - `E`: Execution policy (determines if trade should execute)
///
/// # Examples
///
/// ```
/// use issun_core::mechanics::exchange::prelude::*;
/// use issun_core::mechanics::{Mechanic, EventEmitter};
///
/// // Use default policies
/// type SimpleExchange = ExchangeMechanic;
///
/// // Or customize
/// type MarketExchange = ExchangeMechanic<MarketAdjustedValuation, UrgentExecution>;
///
/// // Create config and state
/// let config = ExchangeConfig::default();
/// let mut state = ExchangeState::default();
///
/// // Prepare input
/// let input = ExchangeInput {
///     offered_value: 100.0,
///     requested_value: 95.0,
///     market_liquidity: 0.7,
///     urgency: 0.3,
/// };
///
/// // Event collector
/// # struct VecEmitter(Vec<ExchangeEvent>);
/// # impl EventEmitter<ExchangeEvent> for VecEmitter {
/// #     fn emit(&mut self, event: ExchangeEvent) { self.0.push(event); }
/// # }
/// let mut emitter = VecEmitter(vec![]);
///
/// // Execute exchange
/// SimpleExchange::step(&config, &mut state, input, &mut emitter);
/// ```
pub struct ExchangeMechanic<
    V: ValuationPolicy = SimpleValuation,
    E: ExecutionPolicy = FairTradeExecution,
> {
    _marker: PhantomData<(V, E)>,
}

impl<V, E> Mechanic for ExchangeMechanic<V, E>
where
    V: ValuationPolicy,
    E: ExecutionPolicy,
{
    type Config = ExchangeConfig;
    type State = ExchangeState;
    type Input = ExchangeInput;
    type Event = ExchangeEvent;
    type Execution = crate::mechanics::ParallelSafe;

    fn step(
        config: &Self::Config,
        state: &mut Self::State,
        input: Self::Input,
        emitter: &mut impl EventEmitter<Self::Event>,
    ) {
        // 1. Emit proposal event
        emitter.emit(ExchangeEvent::TradeProposed {
            offered: input.offered_value,
            requested: input.requested_value,
        });

        // 2. Check if trade should be executed (Policy E)
        let execution_result = E::should_execute(
            input.offered_value,
            input.requested_value,
            input.urgency,
            state.reputation,
            state.is_locked,
            config,
        );

        if let Err(reason) = execution_result {
            emitter.emit(ExchangeEvent::TradeRejected { reason });

            // Update reputation on rejection
            let rep_change =
                E::calculate_reputation_change(input.offered_value, input.requested_value, false);
            state.decrease_reputation(rep_change.abs());

            emitter.emit(ExchangeEvent::ReputationChanged {
                delta: -rep_change.abs(),
                new_value: state.reputation,
            });

            return;
        }

        // 3. Calculate fair value (Policy V)
        let fair_value = V::calculate_fair_value(
            input.offered_value,
            input.requested_value,
            input.market_liquidity,
            state.reputation,
            config,
        );

        // If fair value is zero, reject trade
        if fair_value <= 0.0 {
            emitter.emit(ExchangeEvent::TradeRejected {
                reason: super::types::RejectionReason::UnfairTrade,
            });

            let rep_change =
                E::calculate_reputation_change(input.offered_value, input.requested_value, false);
            state.decrease_reputation(rep_change.abs());

            emitter.emit(ExchangeEvent::ReputationChanged {
                delta: -rep_change.abs(),
                new_value: state.reputation,
            });

            return;
        }

        // 4. Calculate fee
        let fee = V::calculate_fee(fair_value, config);
        let final_value = fair_value - fee;

        // 5. Execute trade
        state.total_trades += 1;

        emitter.emit(ExchangeEvent::TradeAccepted {
            fair_value: final_value,
            fee,
        });

        // 6. Update reputation on success
        let rep_change =
            E::calculate_reputation_change(input.offered_value, input.requested_value, true);

        if rep_change > 0.0 {
            state.increase_reputation(rep_change);
        } else {
            state.decrease_reputation(rep_change.abs());
        }

        emitter.emit(ExchangeEvent::ReputationChanged {
            delta: rep_change,
            new_value: state.reputation,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanics::exchange::strategies::{MarketAdjustedValuation, UrgentExecution};
    use crate::mechanics::exchange::types::RejectionReason;

    struct VecEmitter(Vec<ExchangeEvent>);
    impl EventEmitter<ExchangeEvent> for VecEmitter {
        fn emit(&mut self, event: ExchangeEvent) {
            self.0.push(event);
        }
    }

    type SimpleExchange = ExchangeMechanic;
    type MarketExchange = ExchangeMechanic<MarketAdjustedValuation, UrgentExecution>;

    #[test]
    fn test_simple_exchange_success() {
        let config = ExchangeConfig::default();
        let mut state = ExchangeState::default();
        let input = ExchangeInput {
            offered_value: 100.0,
            requested_value: 100.0,
            market_liquidity: 0.5,
            urgency: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);

        SimpleExchange::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.total_trades, 1);
        assert!(state.reputation > 0.5); // Increased from default 0.5
        assert!(emitter
            .0
            .iter()
            .any(|e| matches!(e, ExchangeEvent::TradeAccepted { .. })));
    }

    #[test]
    fn test_simple_exchange_unfair_rejection() {
        let config = ExchangeConfig {
            fairness_threshold: 0.8,
            ..Default::default()
        };

        let mut state = ExchangeState::default();
        let input = ExchangeInput {
            offered_value: 50.0,
            requested_value: 100.0, // Ratio 0.5 < 0.8
            market_liquidity: 0.5,
            urgency: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);

        SimpleExchange::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.total_trades, 0);
        assert!(state.reputation < 0.5); // Decreased due to rejection
        assert!(emitter.0.iter().any(|e| matches!(
            e,
            ExchangeEvent::TradeRejected {
                reason: RejectionReason::UnfairTrade
            }
        )));
    }

    #[test]
    fn test_simple_exchange_locked() {
        let config = ExchangeConfig::default();
        let mut state = ExchangeState {
            is_locked: true,
            ..Default::default()
        };

        let input = ExchangeInput {
            offered_value: 100.0,
            requested_value: 100.0,
            market_liquidity: 0.5,
            urgency: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);

        SimpleExchange::step(&config, &mut state, input, &mut emitter);

        assert_eq!(state.total_trades, 0);
        assert!(emitter.0.iter().any(|e| matches!(
            e,
            ExchangeEvent::TradeRejected {
                reason: RejectionReason::EntityLocked
            }
        )));
    }

    #[test]
    fn test_market_exchange_liquidity_bonus() {
        let config = ExchangeConfig::default();
        let mut state = ExchangeState::new(1.0); // Max reputation
        let input = ExchangeInput {
            offered_value: 100.0,
            requested_value: 100.0,
            market_liquidity: 1.0, // Max liquidity
            urgency: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);

        MarketExchange::step(&config, &mut state, input, &mut emitter);

        // Should get fair_value > 100 due to bonuses
        if let Some(ExchangeEvent::TradeAccepted { fair_value, .. }) = emitter
            .0
            .iter()
            .find(|e| matches!(e, ExchangeEvent::TradeAccepted { .. }))
        {
            assert!(*fair_value > 100.0); // Bonus applied
        } else {
            panic!("Expected TradeAccepted event");
        }
    }

    #[test]
    fn test_market_exchange_urgent_unfair_trade() {
        let config = ExchangeConfig {
            fairness_threshold: 0.5,
            ..Default::default()
        };

        let mut state = ExchangeState::new(0.8); // High reputation
        let input = ExchangeInput {
            offered_value: 60.0,
            requested_value: 100.0, // Ratio 0.6 (would normally fail)
            market_liquidity: 0.5,
            urgency: 0.9, // High urgency allows unfair trade
        };

        let mut emitter = VecEmitter(vec![]);

        MarketExchange::step(&config, &mut state, input, &mut emitter);

        // Should succeed due to high urgency
        assert_eq!(state.total_trades, 1);
        // But reputation should decrease for unfair trade
        assert!(state.reputation < 0.8);
    }

    #[test]
    fn test_transaction_fee() {
        let config = ExchangeConfig {
            transaction_fee_rate: 0.05, // 5% fee
            ..Default::default()
        };

        let mut state = ExchangeState::default();
        let input = ExchangeInput {
            offered_value: 100.0,
            requested_value: 100.0,
            market_liquidity: 0.0,
            urgency: 0.0,
        };

        let mut emitter = VecEmitter(vec![]);

        SimpleExchange::step(&config, &mut state, input, &mut emitter);

        if let Some(ExchangeEvent::TradeAccepted { fair_value, fee }) = emitter
            .0
            .iter()
            .find(|e| matches!(e, ExchangeEvent::TradeAccepted { .. }))
        {
            assert_eq!(*fee, 5.0); // 5% of 100
            assert_eq!(*fair_value, 95.0); // 100 - 5
        } else {
            panic!("Expected TradeAccepted event");
        }
    }
}
