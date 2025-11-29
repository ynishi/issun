//! Bevy-specific types and adapters for exchange_v2 plugin.

use bevy::{ecs::message::MessageWriter, prelude::*};
use issun_core::mechanics::exchange::{ExchangeConfig, ExchangeEvent, ExchangeState};
use issun_core::mechanics::EventEmitter;

/// Exchange configuration resource - wraps issun-core's ExchangeConfig
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
#[derive(Default)]
pub struct ExchangeConfigResource {
    #[reflect(ignore)]
    pub config: ExchangeConfig,
}


impl ExchangeConfigResource {
    pub fn new(config: ExchangeConfig) -> Self {
        Self { config }
    }
}

/// Message wrapper for issun-core's ExchangeEvent
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct ExchangeEventWrapper {
    pub entity: Entity,
    pub event: ExchangeEvent,
}

/// Bevy adapter: Wraps Bevy's MessageWriter to implement EventEmitter.
pub struct BevyExchangeEventEmitter<'a, 'b> {
    entity: Entity,
    writer: &'a mut MessageWriter<'b, ExchangeEventWrapper>,
}

impl<'a, 'b> BevyExchangeEventEmitter<'a, 'b> {
    pub fn new(entity: Entity, writer: &'a mut MessageWriter<'b, ExchangeEventWrapper>) -> Self {
        Self { entity, writer }
    }
}

impl<'a, 'b> EventEmitter<ExchangeEvent> for BevyExchangeEventEmitter<'a, 'b> {
    fn emit(&mut self, event: ExchangeEvent) {
        self.writer.write(ExchangeEventWrapper {
            entity: self.entity,
            event,
        });
    }
}

/// Component: Trader state (wraps issun-core's ExchangeState).
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[derive(Default)]
pub struct Trader {
    #[reflect(ignore)]
    pub state: ExchangeState,
}


impl Trader {
    pub fn new(initial_reputation: f32) -> Self {
        Self {
            state: ExchangeState::new(initial_reputation),
        }
    }
}

/// Component: Offered value for trade.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct OfferedValue {
    pub value: f32,
}

/// Component: Requested value for trade.
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct RequestedValue {
    pub value: f32,
}

/// Component: Market liquidity (0.0 to 1.0).
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct MarketLiquidity {
    pub value: f32,
}

impl Default for MarketLiquidity {
    fn default() -> Self {
        Self { value: 0.5 }
    }
}

/// Message: Request a trade.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct TradeRequested {
    pub trader: Entity,
    pub offered_value: f32,
    pub requested_value: f32,
    pub urgency: f32,
}

/// Message: Trade was completed.
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct TradeCompleted {
    pub trader: Entity,
    pub fair_value: f32,
    pub fee: f32,
    pub success: bool,
}
