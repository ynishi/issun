//! ExchangePluginV2: Policy-Based Exchange System for Bevy.

use bevy::prelude::*;
use issun_core::mechanics::exchange::{ExchangeConfig, ExchangeInput};
use issun_core::mechanics::{ExecutionHint, Mechanic};
use std::marker::PhantomData;

use super::systems::{log_exchange_events, trade_system};
use super::types::{
    ExchangeConfigResource, ExchangeEventWrapper, MarketLiquidity, OfferedValue, RequestedValue,
    TradeCompleted, TradeRequested, Trader,
};

/// Exchange plugin using issun-core's policy-based design.
pub struct ExchangePluginV2<M>
where
    M: Mechanic<
        Config = ExchangeConfig,
        State = issun_core::mechanics::exchange::ExchangeState,
        Input = ExchangeInput,
        Event = issun_core::mechanics::exchange::ExchangeEvent,
    >,
{
    pub config: ExchangeConfig,
    _phantom: PhantomData<M>,
}

impl<M> Default for ExchangePluginV2<M>
where
    M: Mechanic<
        Config = ExchangeConfig,
        State = issun_core::mechanics::exchange::ExchangeState,
        Input = ExchangeInput,
        Event = issun_core::mechanics::exchange::ExchangeEvent,
    >,
{
    fn default() -> Self {
        Self {
            config: ExchangeConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<M> ExchangePluginV2<M>
where
    M: Mechanic<
        Config = ExchangeConfig,
        State = issun_core::mechanics::exchange::ExchangeState,
        Input = ExchangeInput,
        Event = issun_core::mechanics::exchange::ExchangeEvent,
    >,
{
    pub fn with_config(config: ExchangeConfig) -> Self {
        Self {
            config,
            _phantom: PhantomData,
        }
    }
}

impl<M> Plugin for ExchangePluginV2<M>
where
    M: Mechanic<
            Config = ExchangeConfig,
            State = issun_core::mechanics::exchange::ExchangeState,
            Input = ExchangeInput,
            Event = issun_core::mechanics::exchange::ExchangeEvent,
        > + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(ExchangeConfigResource::new(self.config.clone()));

        app.register_type::<ExchangeConfigResource>();
        app.register_type::<Trader>();
        app.register_type::<OfferedValue>();
        app.register_type::<RequestedValue>();
        app.register_type::<MarketLiquidity>();

        app.add_message::<TradeRequested>();
        app.add_message::<TradeCompleted>();
        app.add_message::<ExchangeEventWrapper>();

        if M::Execution::PARALLEL_SAFE {
            app.add_systems(Update, (trade_system::<M>, log_exchange_events));
            info!(
                "ExchangePluginV2 initialized with mechanic: {} (parallel-safe)",
                std::any::type_name::<M>()
            );
        } else {
            app.add_systems(Update, trade_system::<M>);
            app.add_systems(Update, log_exchange_events);
            info!(
                "ExchangePluginV2 initialized with mechanic: {} (sequential)",
                std::any::type_name::<M>()
            );
        }
    }
}
