//! Systems for reputation_v2 plugin.
//!
//! These systems adapt Bevy's ECS to issun-core's pure reputation logic.

use bevy::{
    ecs::message::{MessageReader, MessageWriter},
    prelude::*,
};
use issun_core::mechanics::reputation::{ReputationConfig, ReputationInput};
use issun_core::mechanics::Mechanic;

use super::types::{
    BevyEventEmitter, ReputationChangeRequested, ReputationConfigResource, ReputationEventWrapper,
    ReputationValue,
};

/// System: Handle reputation change requests using the generic Mechanic.
///
/// This system:
/// 1. Reads ReputationChangeRequested messages
/// 2. Queries entity's ReputationValue component
/// 3. Constructs ReputationInput from message data
/// 4. Calls Mechanic::step (issun-core)
/// 5. Updates entity's ReputationValue component
/// 6. Emits ReputationEvent messages
///
/// # Generic Parameters
///
/// - `M`: The reputation mechanic to use (e.g., `ReputationMechanic<LinearChange, NoDecay, HardClamp>`)
///
/// # Safety
///
/// This system validates that the target entity exists before processing.
/// Invalid entities are skipped with a warning.
pub fn reputation_system<M>(
    mut requests: MessageReader<ReputationChangeRequested>,
    config: Res<ReputationConfigResource>,
    mut reputation_query: Query<&mut ReputationValue>,
    mut reputation_events: MessageWriter<ReputationEventWrapper>,
) where
    M: Mechanic<
            Config = ReputationConfig,
            State = issun_core::mechanics::reputation::ReputationState,
            Input = ReputationInput,
            Event = issun_core::mechanics::reputation::ReputationEvent,
        > + Send
        + Sync
        + 'static,
{
    for request in requests.read() {
        // Validate entity exists
        let Ok(mut reputation) = reputation_query.get_mut(request.entity) else {
            warn!(
                "ReputationChangeRequested: entity {:?} does not exist or has no ReputationValue component",
                request.entity
            );
            continue;
        };

        // 1. Convert ReputationValue to ReputationState
        let mut state = reputation.to_reputation_state();

        // 2. Construct ReputationInput from message data
        let input = ReputationInput {
            delta: request.delta,
            elapsed_time: request.elapsed_time,
        };

        // 3. Create event emitter (wraps Bevy's MessageWriter)
        let mut emitter = BevyEventEmitter::new(request.entity, &mut reputation_events);

        // 4. Call issun-core's pure reputation logic
        M::step(&config.config, &mut state, input, &mut emitter);

        // 5. Update ReputationValue component from modified state
        reputation.from_reputation_state(&state);
    }
}

/// System: Log reputation events for debugging.
///
/// This system listens to ReputationEventWrappers and logs them.
/// In a real game, you might use this to trigger UI updates or achievements.
pub fn log_reputation_events(mut reputation_events: MessageReader<ReputationEventWrapper>) {
    for wrapper in reputation_events.read() {
        match wrapper.event {
            issun_core::mechanics::reputation::ReputationEvent::ValueChanged {
                old_value,
                new_value,
            } => {
                info!(
                    "Reputation changed for {:?}: {:.1} → {:.1}",
                    wrapper.entity, old_value, new_value
                );
            }
            issun_core::mechanics::reputation::ReputationEvent::ReachedMinimum { min_value } => {
                info!(
                    "Reputation reached minimum for {:?}: {:.1}",
                    wrapper.entity, min_value
                );
            }
            issun_core::mechanics::reputation::ReputationEvent::ReachedMaximum { max_value } => {
                info!(
                    "Reputation reached maximum for {:?}: {:.1}",
                    wrapper.entity, max_value
                );
            }
            issun_core::mechanics::reputation::ReputationEvent::Clamped {
                attempted_value,
                clamped_value,
            } => {
                debug!(
                    "Reputation clamped for {:?}: {:.1} → {:.1}",
                    wrapper.entity, attempted_value, clamped_value
                );
            }
        }
    }
}
