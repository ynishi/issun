//! Garden resource - holds the simulation state

use crate::event_log::EventLog;
use crate::hooks::{GardenEntropyHook, GardenGenerationHook};
use crate::models::PlantSpecies;
use hecs::Entity;
use issun::plugin::entropy::{
    Durability, EntropyConfig, EntropyStateECS, EntropySystemECS, EnvironmentalExposure,
    MaterialType,
};
use issun::plugin::generation::{
    Generation, GenerationConfig, GenerationEnvironment, GenerationStateECS, GenerationSystemECS,
    GenerationType,
};
use std::sync::Arc;

/// Garden simulation state
pub struct Garden {
    pub generation_system: GenerationSystemECS,
    pub generation_state: GenerationStateECS,
    pub generation_config: GenerationConfig,

    pub entropy_system: EntropySystemECS,
    pub entropy_state: EntropyStateECS,
    pub entropy_config: EntropyConfig,

    pub plants: Vec<(Entity, PlantSpecies)>,
}

impl Garden {
    pub fn new() -> Self {
        let generation_hook = Arc::new(GardenGenerationHook);
        let entropy_hook = Arc::new(GardenEntropyHook);

        Self {
            generation_system: GenerationSystemECS::new(generation_hook),
            generation_state: GenerationStateECS::new(),
            generation_config: GenerationConfig::default(),

            entropy_system: EntropySystemECS::new(entropy_hook),
            entropy_state: EntropyStateECS::new(),
            entropy_config: EntropyConfig::default(),

            plants: Vec::new(),
        }
    }

    pub fn plant_seed(&mut self, species: PlantSpecies) {
        // Create generation component (growth)
        let generation = Generation::new(
            species.max_growth(),
            species.growth_rate(),
            GenerationType::Organic,
        );

        // Create environment (optimal conditions for now)
        let environment = GenerationEnvironment::with_values(
            22.0, // temperature
            0.8,  // fertility
            1.0,  // resources (water)
            0.9,  // light
        );

        // Spawn in generation system
        let gen_entity = self.generation_state.spawn_entity(generation, environment);

        // Create durability component (health/decay)
        let durability = Durability::new(
            species.max_durability(),
            species.decay_rate(),
            MaterialType::Organic,
        );

        // Create environmental exposure
        let exposure = EnvironmentalExposure {
            humidity: 0.5,
            pollution: 0.0,
            temperature: 22.0,
            sunlight_exposure: 0.9,
        };

        // Spawn in entropy system
        let _ent_entity = self.entropy_state.spawn_entity(durability, exposure);

        // Store plant info
        self.plants.push((gen_entity, species));
    }

    pub async fn update_tick(&mut self, delta_time: f32, event_log: &mut EventLog) {
        // Track previous states for event detection
        let mut prev_states: Vec<(bool, bool)> = Vec::new();
        for (entity, _species) in &self.plants {
            let was_ready = if let Ok(gen) = self.generation_state.world.get::<&Generation>(*entity)
            {
                gen.progress_ratio() >= 1.0
            } else {
                false
            };

            let was_dying = if let Ok(dur) = self.entropy_state.world.get::<&Durability>(*entity) {
                dur.current / dur.max < 0.2
            } else {
                false
            };

            prev_states.push((was_ready, was_dying));
        }

        // Update generation (plants grow)
        self.generation_system
            .update_generation(
                &mut self.generation_state,
                &self.generation_config,
                delta_time,
            )
            .await;

        // Update entropy (plants decay)
        self.entropy_system
            .update_decay(&mut self.entropy_state, &self.entropy_config, delta_time)
            .await;

        // Detect and log events
        for (idx, (entity, species)) in self.plants.iter().enumerate() {
            let (was_ready, was_dying) = prev_states[idx];

            // Check if plant just became ready to harvest
            if let Ok(gen) = self.generation_state.world.get::<&Generation>(*entity) {
                let is_ready = gen.progress_ratio() >= 1.0;
                if is_ready && !was_ready {
                    event_log.log(format!("🌟 {} is ready to harvest!", species.name()));
                }
            }

            // Check if plant is dying
            if let Ok(dur) = self.entropy_state.world.get::<&Durability>(*entity) {
                let ratio = dur.current / dur.max;
                let is_dying = ratio < 0.2;
                if is_dying && !was_dying {
                    event_log.log(format!(
                        "⚠️ {} is dying (health: {:.1}%)",
                        species.name(),
                        ratio * 100.0
                    ));
                }
            }
        }

        // Cleanup
        self.generation_system
            .cleanup_completed(&mut self.generation_state);
        self.entropy_system
            .cleanup_destroyed(&mut self.entropy_state);
    }
}

impl Default for Garden {
    fn default() -> Self {
        Self::new()
    }
}
