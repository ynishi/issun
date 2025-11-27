use bevy::prelude::*;
use issun_core::mechanics::diplomacy::{DiplomacyConfig, DiplomacyInput};
use issun_core::mechanics::Mechanic;
use std::marker::PhantomData;

use crate::IssunSet;

use super::systems::{log_negotiation_events, negotiation_system};
use super::types::{
    DiplomacyConfigResource, DiplomaticStance, NegotiationRequested, NegotiationResult, Negotiator,
};

/// Diplomacy plugin using issun-core's policy-based design.
pub struct DiplomacyPlugin<M>
where
    M: Mechanic<
        Config = DiplomacyConfig,
        State = issun_core::mechanics::diplomacy::DiplomacyState,
        Input = DiplomacyInput,
        Event = issun_core::mechanics::diplomacy::DiplomacyEvent,
    >,
{
    pub config: DiplomacyConfig,
    _phantom: PhantomData<M>,
}

impl<M> Default for DiplomacyPlugin<M>
where
    M: Mechanic<
        Config = DiplomacyConfig,
        State = issun_core::mechanics::diplomacy::DiplomacyState,
        Input = DiplomacyInput,
        Event = issun_core::mechanics::diplomacy::DiplomacyEvent,
    >,
{
    fn default() -> Self {
        Self {
            config: DiplomacyConfig::default(),
            _phantom: PhantomData,
        }
    }
}

impl<M> Plugin for DiplomacyPlugin<M>
where
    M: Mechanic<
            Config = DiplomacyConfig,
            State = issun_core::mechanics::diplomacy::DiplomacyState,
            Input = DiplomacyInput,
            Event = issun_core::mechanics::diplomacy::DiplomacyEvent,
        > + Send
        + Sync
        + 'static,
{
    fn build(&self, app: &mut App) {
        app.insert_resource(DiplomacyConfigResource::new(self.config.clone()));

        app.register_type::<DiplomacyConfigResource>();
        app.register_type::<Negotiator>();
        app.register_type::<DiplomaticStance>();

        app.add_message::<NegotiationRequested>();
        app.add_message::<NegotiationResult>();

        app.add_systems(
            Update,
            (negotiation_system::<M>, log_negotiation_events).in_set(IssunSet::Logic),
        );

        info!(
            "DiplomacyPlugin initialized with mechanic: {}",
            std::any::type_name::<M>()
        );
    }
}
