//! Securitization Plugin V2 - Policy-Based Design
//!
//! This plugin integrates issun-core's policy-based securitization mechanic with Bevy's ECS.

use bevy::prelude::*;
use issun_core::mechanics::securitization::{
    SecuritizationAction, SecuritizationConfig, SecuritizationInput,
};
use issun_core::mechanics::Mechanic;
use std::marker::PhantomData;

/// Securitization configuration resource
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
#[derive(Default)]
pub struct SecuritizationConfigResource {
    #[reflect(ignore)]
    pub config: SecuritizationConfig,
}


/// Component: Securitization pool state
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
#[derive(Default)]
pub struct SecuritizationPool {
    #[reflect(ignore)]
    pub state: issun_core::mechanics::securitization::SecuritizationState,
}


/// Message: Request securitization operation
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct SecuritizationRequested {
    pub pool: Entity,
    pub action: SecuritizationAction,
    pub asset_value: f32,
    pub securities_amount: f32,
    pub risk_factor: f32,
}

/// Securitization plugin
pub struct SecuritizationPluginV2<M>
where
    M: Mechanic<
        Config = SecuritizationConfig,
        State = issun_core::mechanics::securitization::SecuritizationState,
        Input = SecuritizationInput,
        Event = issun_core::mechanics::securitization::SecuritizationEvent,
    >,
{
    pub config: SecuritizationConfig,
    _phantom: PhantomData<M>,
}

impl<M> Default for SecuritizationPluginV2<M>
where
    M: Mechanic<
        Config = SecuritizationConfig,
        State = issun_core::mechanics::securitization::SecuritizationState,
        Input = SecuritizationInput,
        Event = issun_core::mechanics::securitization::SecuritizationEvent,
    >,
{
    fn default() -> Self {
        Self {
            config: SecuritizationConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<M> Plugin for SecuritizationPluginV2<M>
where
    M: Mechanic<
            Config = SecuritizationConfig,
            State = issun_core::mechanics::securitization::SecuritizationState,
            Input = SecuritizationInput,
            Event = issun_core::mechanics::securitization::SecuritizationEvent,
        > + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(SecuritizationConfigResource {
            config: self.config.clone(),
        });

        app.register_type::<SecuritizationConfigResource>();
        app.register_type::<SecuritizationPool>();

        app.add_message::<SecuritizationRequested>();

        info!(
            "SecuritizationPluginV2 initialized with mechanic: {}",
            std::any::type_name::<M>()
        );
    }
}
