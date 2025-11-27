//! Contagion V2 systems - using issun-core's Mechanic trait

use bevy::{
    ecs::message::{MessageReader, MessageWriter},
    prelude::*,
};
use issun_core::mechanics::contagion::prelude::*;
use issun_core::mechanics::{EventEmitter, Mechanic};

use super::components::*;
use super::plugin::{ContagionConfigResource, ContagionEventWrapper, ContagionRng};

/// Generic contagion step system for any mechanic type
///
/// This system:
/// 1. Queries all entities with ContagionState<M> and ContagionInputParams
/// 2. For each entity, calls Mechanic::step() with current state and input
/// 3. Emits events through Bevy's message system
pub fn contagion_step_system<M>(
    config: Res<ContagionConfigResource>,
    mut rng: ResMut<ContagionRng>,
    mut query: Query<(Entity, &mut ContagionState<M>, &ContagionInputParams)>,
    mut message_writer: MessageWriter<ContagionEventWrapper>,
) where
    M: Mechanic<
            Config = ContagionConfig,
            State = SimpleSeverity,
            Input = ContagionInput,
            Event = ContagionEvent,
        > + Send
        + Sync
        + 'static,
{
    for (entity, mut state, params) in query.iter_mut() {
        // Generate random value for this frame
        let rng_value = rng.gen_f32();

        // Convert params to input
        let input = params.to_input(rng_value);

        // Create message emitter that captures entity and writer
        let mut emitter = BevyMessageEmitter {
            entity,
            writer: &mut message_writer,
        };

        // Call issun-core's mechanic step
        M::step(&config.config, &mut state.state, input, &mut emitter);
    }
}

/// Message emitter adapter for Bevy's message system
struct BevyMessageEmitter<'a, 'b> {
    entity: Entity,
    writer: &'a mut MessageWriter<'b, ContagionEventWrapper>,
}

impl<'a, 'b> EventEmitter<ContagionEvent> for BevyMessageEmitter<'a, 'b> {
    fn emit(&mut self, event: ContagionEvent) {
        self.writer.write(ContagionEventWrapper {
            entity: self.entity,
            event,
        });
    }
}

// ==================== Message Logging System ====================

/// Example system to log contagion messages
pub fn log_contagion_events(mut messages: MessageReader<ContagionEventWrapper>) {
    for wrapper in messages.read() {
        match wrapper.event {
            ContagionEvent::Infected => {
                info!("Entity {:?} became infected!", wrapper.entity);
            }
            ContagionEvent::Progressed { new_severity } => {
                info!(
                    "Entity {:?} infection progressed to severity {}",
                    wrapper.entity, new_severity
                );
            }
        }
    }
}
