//! Contagion plugin definition

use bevy::prelude::*;

use crate::IssunSet;
use super::{
    components::*,
    events::*,
    resources::*,
    systems::*,
};

/// Contagion propagation plugin with infection state machine
pub struct ContagionPlugin {
    pub config: ContagionConfig,
    pub rng_seed: Option<u64>,
}

impl Default for ContagionPlugin {
    fn default() -> Self {
        Self {
            config: ContagionConfig::default(),
            rng_seed: None,
        }
    }
}

impl ContagionPlugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: ContagionConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.rng_seed = Some(seed);
        self
    }
}

impl Plugin for ContagionPlugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(self.config.clone());
        app.insert_resource(NodeRegistry::default());
        app.insert_resource(EdgeRegistry::default());
        app.insert_resource(NodeInfectionIndex::default());
        app.insert_resource(TurnCounter::default());

        // RNG
        let rng = if let Some(seed) = self.rng_seed {
            ContagionRng::with_seed(seed)
        } else {
            ContagionRng::default()
        };
        app.insert_resource(rng);

        // Messages
        app.add_message::<ContagionSpawnRequested>()
            .add_message::<PropagationStepRequested>()
            .add_message::<TurnAdvancedMessage>()
            .add_message::<CredibilityDecayRequested>()
            .add_message::<ContagionSpawnedEvent>()
            .add_message::<ContagionSpreadEvent>()
            .add_message::<InfectionStateChangedEvent>()
            .add_message::<ReinfectionOccurredEvent>()
            .add_message::<ContagionRemovedEvent>()
            .add_message::<PropagationStepCompletedEvent>();

        // Component registration
        app.register_type::<ContagionNode>()
            .register_type::<PropagationEdge>()
            .register_type::<NodeType>()
            .register_type::<Contagion>()
            .register_type::<ContagionInfection>()
            .register_type::<InfectionState>()
            .register_type::<ContagionDuration>()
            .register_type::<ContagionContent>()
            .register_type::<DiseaseLevel>()
            .register_type::<TrendDirection>()
            .register_type::<InfectionStateType>()
            .register_type::<RemovalReason>()
            .register_type::<ContagionConfig>()
            .register_type::<TimeMode>()
            .register_type::<DurationConfig>()
            .register_type::<NodeRegistry>()
            .register_type::<EdgeRegistry>()
            .register_type::<NodeInfectionIndex>()
            .register_type::<TurnCounter>()
            .register_type::<ContagionRng>();

        // Systems
        app.add_systems(
            Update,
            (
                handle_contagion_spawn.in_set(IssunSet::Logic),
                progress_infection_states_continuous.in_set(IssunSet::Logic),
                progress_infection_states_turn_based.in_set(IssunSet::Logic),
                handle_propagation_step.in_set(IssunSet::Logic),
                handle_credibility_decay.in_set(IssunSet::PostLogic),
            ),
        );
    }
}
