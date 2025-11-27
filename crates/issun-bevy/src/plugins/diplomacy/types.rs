use bevy::prelude::*;
use issun_core::mechanics::diplomacy::{
    ArgumentType, DiplomacyConfig, DiplomacyEvent, DiplomacyState,
};

/// Resource wrapper for DiplomacyConfig
#[derive(Resource, Reflect, Default, Clone)]
#[reflect(Resource)]
pub struct DiplomacyConfigResource {
    #[reflect(ignore)]
    pub config: DiplomacyConfig,
}

impl DiplomacyConfigResource {
    pub fn new(config: DiplomacyConfig) -> Self {
        Self { config }
    }
}

/// Component for entities that can negotiate (Actors).
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct Negotiator {
    /// Base persuasion power
    pub persuasion: f32,
}

/// Component for entities that can be negotiated with (Targets).
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DiplomaticStance {
    /// Base resistance to persuasion
    pub resistance: f32,
    /// Current relationship score (-1.0 to 1.0)
    pub relationship: f32,
    /// Patience turns remaining
    pub patience: u32,
    /// Current agreement progress
    pub progress: f32,
}

impl Default for DiplomaticStance {
    fn default() -> Self {
        Self {
            resistance: 10.0,
            relationship: 0.0,
            patience: 5,
            progress: 0.0,
        }
    }
}

impl DiplomaticStance {
    pub fn to_state(&self) -> DiplomacyState {
        let mut state = DiplomacyState::new(self.relationship, self.patience);
        state.agreement_progress = self.progress;
        state
    }

    pub fn from_state(&mut self, state: &DiplomacyState) {
        self.progress = state.agreement_progress;
        self.patience = state.patience;
        // Relationship might change based on events, but state holds the snapshot used for calc
    }
}

/// Message: Request a negotiation attempt.
#[derive(bevy::ecs::message::Message, Debug, Clone)]
pub struct NegotiationRequested {
    pub initiator: Entity,
    pub target: Entity,
    pub argument_type: ArgumentType,
}

/// Message: Result of a negotiation step.
#[derive(bevy::ecs::message::Message, Debug, Clone, PartialEq)]
pub struct NegotiationResult {
    pub initiator: Entity,
    pub target: Entity,
    pub event: DiplomacyEvent,
}

/// Helper to emit Bevy messages from the core mechanic.
pub struct BevyDiplomacyEmitter<'a, 'b> {
    initiator: Entity,
    target: Entity,
    writer: &'a mut bevy::ecs::message::MessageWriter<'b, NegotiationResult>,
}

impl<'a, 'b> BevyDiplomacyEmitter<'a, 'b> {
    pub fn new(
        initiator: Entity,
        target: Entity,
        writer: &'a mut bevy::ecs::message::MessageWriter<'b, NegotiationResult>,
    ) -> Self {
        Self {
            initiator,
            target,
            writer,
        }
    }
}

impl<'a, 'b> issun_core::mechanics::EventEmitter<DiplomacyEvent> for BevyDiplomacyEmitter<'a, 'b> {
    fn emit(&mut self, event: DiplomacyEvent) {
        self.writer.write(NegotiationResult {
            initiator: self.initiator,
            target: self.target,
            event,
        });
    }
}
