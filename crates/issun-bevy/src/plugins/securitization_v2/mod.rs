//! Securitization Plugin V2 - Policy-Based Design
//!
//! This plugin integrates issun-core's policy-based securitization mechanic with Bevy's ECS.

use bevy::prelude::*;
use issun_core::mechanics::securitization::{SecuritizationConfig, SecuritizationInput, SecuritizationAction};
use issun_core::mechanics::Mechanic;
use std::marker::PhantomData;

/// Securitization configuration resource
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct SecuritizationConfigResource {
    #[reflect(ignore)]
    pub config: SecuritizationConfig,
}

impl Default for SecuritizationConfigResource {
    fn default() -> Self {
        Self {
            config: SecuritizationConfig::default(),
        }
    }
}

/// Component: Securitization pool state
#[derive(Component, Reflect, Clone)]
#[reflect(Component)]
pub struct SecuritizationPool {
    #[reflect(ignore)]
    pub state: issun_core::mechanics::securitization::SecuritizationState,
}

impl Default for SecuritizationPool {
    fn default() -> Self {
        Self {
            state: issun_core::mechanics::securitization::SecuritizationState::default(),
        }
    }
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
