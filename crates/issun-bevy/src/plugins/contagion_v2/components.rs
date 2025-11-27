//! Component wrappers for issun-core contagion types

use bevy::prelude::*;
use issun_core::mechanics::contagion::prelude::*;
use issun_core::mechanics::Mechanic;
use std::marker::PhantomData;

// ==================== Core Components ====================

/// Contagion state component - wraps issun-core's SimpleSeverity
///
/// Generic over the mechanic type to allow different virus behaviors
/// per entity at compile time.
#[derive(Component)]
pub struct ContagionState<M: Mechanic<State = SimpleSeverity>> {
    pub state: SimpleSeverity,
    _marker: PhantomData<M>,
}

impl<M: Mechanic<State = SimpleSeverity>> Default for ContagionState<M> {
    fn default() -> Self {
        Self {
            state: SimpleSeverity::default(),
            _marker: PhantomData,
        }
    }
}

impl<M: Mechanic<State = SimpleSeverity>> ContagionState<M> {
    pub fn new(severity: u32) -> Self {
        Self {
            state: SimpleSeverity { severity },
            _marker: PhantomData,
        }
    }

    pub fn severity(&self) -> u32 {
        self.state.severity
    }

    pub fn is_infected(&self) -> bool {
        self.state.severity > 0
    }
}

/// Contagion configuration resource - wraps issun-core's ContagionConfig
#[derive(Resource, Clone, Reflect, Default)]
#[reflect(Resource)]
pub struct ContagionConfigResource {
    #[reflect(ignore)]
    pub config: ContagionConfig,
}


impl ContagionConfigResource {
    pub fn new(base_rate: f32) -> Self {
        Self {
            config: ContagionConfig { base_rate },
        }
    }
}

// ==================== Input Components ====================

/// Per-entity input parameters for contagion calculation
#[derive(Component, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct ContagionInputParams {
    /// Population density around this entity (0.0 to 1.0)
    pub density: f32,
    /// Entity's resistance to infection (higher = more resistant)
    pub resistance: u32,
}

impl Default for ContagionInputParams {
    fn default() -> Self {
        Self {
            density: 0.5,
            resistance: 10,
        }
    }
}

impl ContagionInputParams {
    pub fn new(density: f32, resistance: u32) -> Self {
        Self {
            density,
            resistance,
        }
    }

    /// Convert to issun-core's ContagionInput with RNG value
    pub fn to_input(&self, rng: f32) -> ContagionInput {
        ContagionInput {
            density: self.density,
            resistance: self.resistance,
            rng,
        }
    }
}

// ==================== Preset Type Aliases ====================

/// Simple virus with default behavior
pub type SimpleVirusState = ContagionState<SimpleVirus>;

/// Explosive pandemic-style virus
pub type ExplosiveVirusState = ContagionState<ExplosiveVirus>;

/// Zombie apocalypse virus (fast spread + low resistance threshold)
pub type ZombieVirusState = ContagionState<ZombieVirus>;
