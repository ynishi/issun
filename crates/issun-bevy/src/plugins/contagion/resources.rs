//! Resource types for contagion propagation configuration

use bevy::prelude::*;
use rand::SeedableRng;
use std::collections::HashMap;

/// Configuration for contagion propagation
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource)]
pub struct ContagionConfig {
    pub global_propagation_rate: f32,
    pub default_mutation_rate: f32,
    pub lifetime_turns: u64,
    pub min_credibility: f32,
    pub time_mode: TimeMode,
    pub incubation_transmission_rate: f32,
    pub active_transmission_rate: f32,
    pub recovered_transmission_rate: f32,
    pub plain_transmission_rate: f32,
    pub default_incubation_duration: DurationConfig,
    pub default_active_duration: DurationConfig,
    pub default_immunity_duration: DurationConfig,
    pub default_reinfection_enabled: bool,
}

impl Default for ContagionConfig {
    fn default() -> Self {
        Self {
            global_propagation_rate: 0.5,
            default_mutation_rate: 0.1,
            lifetime_turns: 10,
            min_credibility: 0.1,
            time_mode: TimeMode::TurnBased,
            incubation_transmission_rate: 0.2,
            active_transmission_rate: 0.8,
            recovered_transmission_rate: 0.05,
            plain_transmission_rate: 0.0,
            default_incubation_duration: DurationConfig {
                base: 3.0,
                variance: 0.3,
            },
            default_active_duration: DurationConfig {
                base: 7.0,
                variance: 0.2,
            },
            default_immunity_duration: DurationConfig {
                base: 10.0,
                variance: 0.5,
            },
            default_reinfection_enabled: true,
        }
    }
}

impl ContagionConfig {
    pub fn with_propagation_rate(mut self, rate: f32) -> Self {
        self.global_propagation_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn with_mutation_rate(mut self, rate: f32) -> Self {
        self.default_mutation_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn with_time_mode(mut self, mode: TimeMode) -> Self {
        self.time_mode = mode;
        self
    }

    pub fn with_state_transmission_rates(
        mut self,
        incubation: f32,
        active: f32,
        recovered: f32,
    ) -> Self {
        self.incubation_transmission_rate = incubation.clamp(0.0, 1.0);
        self.active_transmission_rate = active.clamp(0.0, 1.0);
        self.recovered_transmission_rate = recovered.clamp(0.0, 1.0);
        self
    }
}

/// Time mode for contagion progression
#[derive(Clone, Copy, Reflect, PartialEq, Debug)]
pub enum TimeMode {
    TurnBased,
    TickBased,
    TimeBased,
}

/// Duration configuration with variance
#[derive(Clone, Copy, Reflect, Debug)]
pub struct DurationConfig {
    pub base: f32,
    pub variance: f32,
}

impl DurationConfig {
    pub fn new(base: f32, variance: f32) -> Self {
        Self { base, variance }
    }
}

/// Node ID → Entity lookup
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct NodeRegistry {
    pub id_to_entity: HashMap<String, Entity>,
}

impl NodeRegistry {
    pub fn register(&mut self, id: impl Into<String>, entity: Entity) {
        self.id_to_entity.insert(id.into(), entity);
    }

    pub fn get(&self, id: &str) -> Option<Entity> {
        self.id_to_entity.get(id).copied()
    }
}

/// Edge ID → Entity lookup
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct EdgeRegistry {
    pub id_to_entity: HashMap<String, Entity>,
}

impl EdgeRegistry {
    pub fn register(&mut self, id: impl Into<String>, entity: Entity) {
        self.id_to_entity.insert(id.into(), entity);
    }

    pub fn get(&self, id: &str) -> Option<Entity> {
        self.id_to_entity.get(id).copied()
    }
}

/// Node → Infections index (performance optimization)
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct NodeInfectionIndex {
    pub node_to_infections: HashMap<Entity, Vec<Entity>>,
}

impl NodeInfectionIndex {
    pub fn add_infection(&mut self, node: Entity, infection: Entity) {
        self.node_to_infections
            .entry(node)
            .or_default()
            .push(infection);
    }

    pub fn remove_infection(&mut self, node: Entity, infection: Entity) {
        if let Some(infections) = self.node_to_infections.get_mut(&node) {
            infections.retain(|&e| e != infection);
        }
    }

    pub fn get_infections(&self, node: Entity) -> &[Entity] {
        self.node_to_infections
            .get(&node)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }
}

/// Deterministic RNG for testing/replay
#[derive(Resource, Clone, Reflect)]
#[reflect(Resource, opaque)]
pub struct ContagionRng {
    pub rng: rand::rngs::StdRng,
}

impl Default for ContagionRng {
    fn default() -> Self {
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(0),
        }
    }
}

impl ContagionRng {
    pub fn with_seed(seed: u64) -> Self {
        Self {
            rng: rand::rngs::StdRng::seed_from_u64(seed),
        }
    }
}

/// Turn counter (Turn-based mode only)
#[derive(Resource, Default, Reflect)]
#[reflect(Resource)]
pub struct TurnCounter {
    pub current_turn: u64,
}

impl TurnCounter {
    pub fn advance(&mut self) {
        self.current_turn += 1;
    }

    pub fn current(&self) -> u64 {
        self.current_turn
    }
}
