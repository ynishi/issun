//! Systems for exchange_v2 plugin.

use bevy::prelude::*;
use issun_core::mechanics::exchange::ExchangeInput;
use issun_core::mechanics::Mechanic;

use super::types::{
    BevyExchangeEventEmitter, ExchangeConfigResource, ExchangeEventWrapper, MarketLiquidity,
    TradeCompleted, TradeRequested, Trader,
};

/// System: Process trade requests using the exchange mechanic.
pub fn trade_system<M>(
    config: Res<ExchangeConfigResource>,
    mut traders: Query<(&mut Trader, Option<&MarketLiquidity>)>,
    mut trade_requests: MessageReader<TradeRequested>,
    mut event_writer: MessageWriter<ExchangeEventWrapper>,
    mut trade_completed_writer: MessageWriter<TradeCompleted>,
) where
    M: Mechanic<
        Config = issun_core::mechanics::exchange::ExchangeConfig,
        State = issun_core::mechanics::exchange::ExchangeState,
        Input = ExchangeInput,
        Event = issun_core::mechanics::exchange::ExchangeEvent,
    >,
{
    for request in trade_requests.read() {
        // Get trader component
        let Ok((mut trader, market_liquidity)) = traders.get_mut(request.trader) else {
            warn!("Trade request for non-existent trader: {:?}", request.trader);
            continue;
        };

        // Prepare input
        let input = ExchangeInput {
            offered_value: request.offered_value,
            requested_value: request.requested_value,
            market_liquidity: market_liquidity.map(|ml| ml.value).unwrap_or(0.5),
            urgency: request.urgency,
        };

        // Create event emitter
        let mut emitter = BevyExchangeEventEmitter::new(request.trader, &mut event_writer);

        // Execute mechanic
        M::step(&config.config, &mut trader.state, input, &mut emitter);

        // Check if trade was successful
        let success = trader.state.total_trades > 0;

        // Emit trade completed message
        trade_completed_writer.write(TradeCompleted {
            trader: request.trader,
            fair_value: request.offered_value.min(request.requested_value),
            fee: 0.0, // TODO: Calculate actual fee from events
            success,
        });
    }
}

/// System: Log exchange events.
pub fn log_exchange_events(mut events: MessageReader<ExchangeEventWrapper>) {
    for wrapper in events.read() {
        match &wrapper.event {
            issun_core::mechanics::exchange::ExchangeEvent::TradeProposed {
                offered,
                requested,
            } => {
                debug!(
                    "Trade proposed by {:?}: offered={}, requested={}",
                    wrapper.entity, offered, requested
                );
            }
            issun_core::mechanics::exchange::ExchangeEvent::TradeAccepted {
                fair_value,
                fee,
            } => {
                info!(
                    "Trade accepted for {:?}: fair_value={}, fee={}",
                    wrapper.entity, fair_value, fee
                );
            }
            issun_core::mechanics::exchange::ExchangeEvent::TradeRejected { reason } => {
                warn!(
                    "Trade rejected for {:?}: reason={:?}",
                    wrapper.entity, reason
                );
            }
            issun_core::mechanics::exchange::ExchangeEvent::ReputationChanged {
                delta,
                new_value,
            } => {
                debug!(
                    "Reputation changed for {:?}: delta={}, new={}",
                    wrapper.entity, delta, new_value
                );
            }
        }
    }
}
