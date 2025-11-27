use bevy::{
    ecs::message::{MessageReader, MessageWriter},
    prelude::*,
};
use issun_core::mechanics::diplomacy::{DiplomacyConfig, DiplomacyInput};
use issun_core::mechanics::Mechanic;

use super::types::{
    BevyDiplomacyEmitter, DiplomacyConfigResource, DiplomaticStance, NegotiationRequested,
    NegotiationResult, Negotiator,
};

/// System: Handle negotiation requests using the generic Mechanic.
pub fn negotiation_system<M>(
    mut requests: MessageReader<NegotiationRequested>,
    config: Res<DiplomacyConfigResource>,
    initiator_query: Query<&Negotiator>,
    mut target_query: Query<&mut DiplomaticStance>,
    mut result_writer: MessageWriter<NegotiationResult>,
) where
    M: Mechanic<
            Config = DiplomacyConfig,
            State = issun_core::mechanics::diplomacy::DiplomacyState,
            Input = DiplomacyInput,
            Event = issun_core::mechanics::diplomacy::DiplomacyEvent,
        > + Send
        + Sync
        + 'static,
{
    for request in requests.read() {
        // Validate initiator
        let Ok(negotiator) = initiator_query.get(request.initiator) else {
            warn!(
                "NegotiationRequested: Initiator {:?} invalid",
                request.initiator
            );
            continue;
        };

        // Validate target
        let Ok(mut stance) = target_query.get_mut(request.target) else {
            warn!("NegotiationRequested: Target {:?} invalid", request.target);
            continue;
        };

        // 1. Convert Component to State
        let mut state = stance.to_state();

        // 2. Construct Input
        let input = DiplomacyInput {
            argument_strength: negotiator.persuasion,
            argument_type: request.argument_type,
            target_resistance: stance.resistance,
        };

        // 3. Create Emitter
        let mut emitter =
            BevyDiplomacyEmitter::new(request.initiator, request.target, &mut result_writer);

        // 4. Run Mechanic
        M::step(&config.config, &mut state, input, &mut emitter);

        // 5. Update Component
        stance.from_state(&state);
    }
}

/// System: Log negotiation results.
pub fn log_negotiation_events(mut events: MessageReader<NegotiationResult>) {
    for result in events.read() {
        info!(
            "Negotiation {:?} -> {:?}: {:?}",
            result.initiator, result.target, result.event
        );
    }
}
