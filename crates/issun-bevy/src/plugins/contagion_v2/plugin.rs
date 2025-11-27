//! Contagion V2 plugin definition

use bevy::prelude::*;
use issun_core::mechanics::contagion::prelude::*;

use super::systems::*;
use crate::IssunSet;

/// Contagion V2 plugin - Policy-Based Design integration
///
/// This plugin demonstrates how to use issun-core's contagion mechanic
/// with Bevy ECS using static dispatch and zero-cost abstraction.
#[derive(Default)]
pub struct ContagionV2Plugin {
    pub base_rate: f32,
}

impl ContagionV2Plugin {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_base_rate(mut self, base_rate: f32) -> Self {
        self.base_rate = base_rate;
        self
    }
}

impl Plugin for ContagionV2Plugin {
    fn build(&self, app: &mut App) {
        // Resources
        app.insert_resource(ContagionConfigResource::new(self.base_rate));
        app.insert_resource(ContagionRng::default());

        // Messages - using issun-core's ContagionEvent
        app.add_message::<ContagionEventWrapper>();

        // Component registration
        // Note: We can't register ContagionState<M> (generic) with Bevy's reflection
        // system, but we can register the concrete types
        app.register_type::<ContagionConfigResource>()
            .register_type::<ContagionInputParams>()
            .register_type::<ContagionRng>();

        // Systems - one system per mechanic type
        app.add_systems(
            Update,
            (
                contagion_step_system::<SimpleVirus>.in_set(IssunSet::Logic),
                contagion_step_system::<ExplosiveVirus>.in_set(IssunSet::Logic),
                contagion_step_system::<ZombieVirus>.in_set(IssunSet::Logic),
            ),
        );
    }
}

// Re-export components for easier access in systems
pub use super::components::*;

/// RNG resource for contagion calculations
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct ContagionRng {
    #[reflect(ignore)]
    rng: fastrand::Rng,
}

impl Default for ContagionRng {
    fn default() -> Self {
        Self {
            rng: fastrand::Rng::new(),
        }
    }
}

impl ContagionRng {
    pub fn gen_f32(&mut self) -> f32 {
        self.rng.f32()
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: fastrand::Rng::with_seed(seed),
        }
    }
}

/// Message wrapper for issun-core's ContagionEvent (Bevy 0.17+)
#[derive(bevy::ecs::message::Message, Clone, Debug)]
pub struct ContagionEventWrapper {
    pub entity: Entity,
    pub event: ContagionEvent,
}
